use std::{path::PathBuf, sync::Arc};

use log::{error, info};
use serde_json::json;
use teloxide::{
    net::Download,
    prelude::*,
    types::{InlineQueryResultArticle, InputMessageContent, InputMessageContentText},
    utils::command::BotCommands,
};

use crate::{
    ai::{make_summary, transcribe_voice_note},
    commands::{keyboard, BotCommand, BotDialogue, State},
    db::{Database, User, FREE_USAGE_LIMIT_SECONDS},
    DOWNLOAD_DIR,
};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn start(
    bot: Bot,
    dialog: BotDialogue,
    db: Arc<Database>,
    msg: Message,
) -> HandlerResult {
    let user = db.get_user(msg.chat.id.0 as u64).await?;

    let chat = msg.chat.clone();

    if user.is_none() {
        match db.create_user(chat.into()).await {
            Ok(user) => {
                info!("Created user: {:?}", user);
            }
            Err(e) => {
                error!("Failed to create user for chat: {}", msg.chat.id.0);
                return Err(e.into());
            }
        }
    } else {
        bot.send_message(
            msg.chat.id,
            "You are already setup. Type /help to see the usage.",
        )
        .await?;

        return Ok(());
    }

    dialog.update(State::WaitingForTypefullyApiKey).await?;

    bot.send_message(msg.chat.id, "Hey there! To start please provide me your TypeFully API key so we can create drafts for you. Simply go to https://typefully.com, go to settings -> API & Integrations, and create & copy your API key. Then simply send it here in the chat.")
        .await?;

    Ok(())
}

const TYPEFULLY_API_URL: &str = "https://api.typefully.com/v1/";

pub async fn receive_typefully_api_key(
    bot: Bot,
    dialog: BotDialogue,
    db: Arc<Database>,
    msg: Message,
) -> HandlerResult {
    // Get the api key from the message and try to call the typefully api to check if it's valid
    let api_key = msg.text().unwrap_or_default();
    let chat = msg.chat.clone();
    let Some(user) = db.get_user(chat.id.0 as u64).await? else {
        bot.send_message(
            msg.chat.id,
            "Something went wrong. Please try again with /start.",
        )
        .await?;

        return Err(anyhow::anyhow!("User not found").into());
    };

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}{}", TYPEFULLY_API_URL, "notifications/"))
        .header("X-API-KEY", format!("Bearer {}", api_key))
        .send()
        .await?;

    if response.status().is_success() {
        dialog.update(State::Start).await?;

        user.update_key(&db, api_key).await?;

        bot.send_message(msg.chat.id, "Alright, that looks good. Now you can start using the bot. Type /help to see the usage.").await?;
    } else {
        bot.send_message(
            msg.chat.id,
            "API key is invalid. Please provide a valid API key.",
        )
        .await?;
    }
    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let help_text = format!(
        "{}\n\nHow to use:\n1. Use /start to set up your Typefully API key\n2. Send a voice note to the bot\n3. The bot will transcribe it and create a draft in Typefully\n\nNote: You have 5 minutes of free transcription. After that, you'll need to set your own OpenAI API key using /setapikey.",
        BotCommand::descriptions().to_string()
    );

    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}

pub async fn delete_account(bot: Bot, dialog: BotDialogue, msg: Message) -> HandlerResult {
    dialog.update(State::WaitingForDeleteConfirmation).await?;

    bot.send_message(
        msg.chat.id,
        "⚠️ Are you sure you want to delete your account? This action cannot be undone.\n\nType 'DELETE' to confirm or any other message to cancel.",
    )
    .await?;

    Ok(())
}

pub async fn handle_delete_confirmation(
    bot: Bot,
    dialog: BotDialogue,
    db: Arc<Database>,
    msg: Message,
) -> HandlerResult {
    let confirmation = msg.text().unwrap_or_default();

    if confirmation == "DELETE" {
        let user = user_extractor(&bot, &db, &msg).await?;
        let total_usage = db.get_total_usage_seconds(user.telegram_id).await?;

        db.mark_user_deleted(user.telegram_id, total_usage).await?;

        dialog.update(State::Start).await?;

        bot.send_message(
            msg.chat.id,
            "Your account has been deleted. All your data has been removed, but your usage statistics are retained to prevent abuse. If you want to use the bot again, you'll need to start fresh with /start.",
        )
        .await?;
    } else {
        dialog.update(State::Start).await?;

        bot.send_message(
            msg.chat.id,
            "Account deletion cancelled. Your account remains active.",
        )
        .await?;
    }

    Ok(())
}

pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

pub async fn inline_query_handler(bot: Bot, q: InlineQuery) -> HandlerResult {
    let choose_debian_version = InlineQueryResultArticle::new(
        "0",
        "Chose debian version",
        InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
    )
    .reply_markup(keyboard());

    bot.answer_inline_query(q.id, vec![choose_debian_version.into()])
        .await?;

    Ok(())
}

pub async fn handle_voice_note(bot: Bot, db: Arc<Database>, msg: Message) -> HandlerResult {
    let user = user_extractor(&bot, &db, &msg).await?;
    let voice_note = msg.voice().unwrap();
    let duration_seconds = voice_note.duration.seconds() as i32;

    // Determine which API key to use
    let has_own_api_key = user.openai_api_key.is_some();
    let api_key = match user.openai_api_key {
        Some(user_api_key) => Ok(user_api_key),
        None => {
            if db.has_free_usage(user.telegram_id).await? {
                std::env::var("OPENAI_API_KEY").map_err(|e| {
                    error!("Failed to get OpenAI API key from env: {}", e);
                    anyhow::anyhow!(e)
                })
            } else {
                bot.send_message(
                    msg.chat.id,
                    "You have exceeded your free usage limit of 5 minutes. Please set your own OpenAI API key using /setapikey to continue using the voice transcription feature.",
                )
                .await?;
                Err(anyhow::anyhow!("User exceeded free usage limit"))
            }
        }
    }?;

    let download_path = PathBuf::new().join(DOWNLOAD_DIR);
    let file = bot.get_file(&voice_note.file.id).await?;
    let file_path = download_path.join(format!("{}.ogg", &file.unique_id));
    let mut download_file = tokio::fs::File::create(&file_path).await?;

    bot.download_file(&file.path, &mut download_file).await?;

    bot.send_message(msg.chat.id, "Processing voice note..")
        .await?;

    let file_path = PathBuf::from(file_path);

    // Use a clone for the transcription since we need the original path for cleanup
    let transcription_path = file_path.clone();
    let result = transcribe_voice_note(transcription_path, api_key.clone()).await;

    // Always try to clean up the file, regardless of transcription result
    if let Err(e) = tokio::fs::remove_file(&file_path).await {
        error!("Failed to clean up voice note file: {}", e);
    }

    match result {
        Ok(transcript) => {
            // Only track usage if using free credits
            if !has_own_api_key {
                db.add_usage(user.telegram_id, duration_seconds).await?;
            }

            bot.send_message(msg.chat.id, "Transcription done.").await?;

            match make_summary(
                user.username.clone(),
                transcript,
                api_key,
                user.rewrite_enabled,
            )
            .await
            {
                Ok(summary) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("This is what we got for you: \n\n{}\n\n", summary),
                    )
                    .await?;

                    let api_key = user.typefully_api_key.unwrap_or_else(|| {
                        panic!("User has no TypeFully API key. Please provide a valid API key.");
                    });

                    let client = reqwest::Client::new();
                    let response = client
                        .post(format!("{}{}", TYPEFULLY_API_URL, "drafts/"))
                        .header("X-API-KEY", format!("Bearer {}", api_key))
                        .json(&json!({
                            "content": summary
                        }))
                        .send()
                        .await?;

                    if !response.status().is_success() {
                        bot.send_message(msg.chat.id, "Failed to create draft in TypeFully. Please check your API key and try again.")
                            .await?;
                    }
                }
                Err(e) => {
                    error!("Error making summary by user {}: {:?}", user.telegram_id, e);
                    bot.send_message(
                        msg.chat.id,
                        "An error occurred while transforming the post.",
                    )
                    .await?;
                }
            }
        }
        Err(e) => {
            error!(
                "Error transcribing voice note by user {}: {:?}",
                user.telegram_id, e
            );
            bot.send_message(
                msg.chat.id,
                "An error occurred while transcribing the voice note.",
            )
            .await?;
        }
    }

    Ok(())
}

async fn user_extractor(bot: &Bot, db: &Arc<Database>, msg: &Message) -> anyhow::Result<User> {
    let Some(user) = db.get_user(msg.chat.id.0 as u64).await? else {
        bot.send_message(
            msg.chat.id,
            "Something went wrong. Please try again with /start.",
        )
        .await?;

        return Err(anyhow::anyhow!("User not found").into());
    };

    Ok(user)
}

pub async fn set_api_key(bot: Bot, dialog: BotDialogue, msg: Message) -> HandlerResult {
    dialog.update(State::WaitingForOpenAiApiKey).await?;

    bot.send_message(
        msg.chat.id,
        "Please provide your OpenAI API key. You can get it from https://platform.openai.com/api-keys",
    )
    .await?;

    Ok(())
}

pub async fn receive_openai_api_key(
    bot: Bot,
    dialog: BotDialogue,
    db: Arc<Database>,
    msg: Message,
) -> HandlerResult {
    let api_key = msg.text().unwrap_or_default();
    let user = user_extractor(&bot, &db, &msg).await?;

    // Update the user's OpenAI API key
    user.update_openai_api_key(&db, api_key).await?;

    dialog.update(State::Start).await?;

    bot.send_message(
        msg.chat.id,
        "Your OpenAI API key has been saved. You can now use the voice transcription feature.",
    )
    .await?;

    Ok(())
}

pub async fn usage(bot: Bot, db: Arc<Database>, msg: Message) -> HandlerResult {
    let user = user_extractor(&bot, &db, &msg).await?;
    let total_usage = db.get_total_usage_seconds(user.telegram_id).await?;
    let remaining_seconds = FREE_USAGE_LIMIT_SECONDS - total_usage;

    let message = if user.openai_api_key.is_some() {
        "You are using your own OpenAI API key, so you have unlimited usage.".to_string()
    } else if remaining_seconds <= 0 {
        "You have used up all your free minutes. Please use /setapikey to set your own OpenAI API key to continue using the bot.".to_string()
    } else {
        let minutes = remaining_seconds / 60;
        let seconds = remaining_seconds % 60;
        format!(
            "You have {} minutes and {} seconds of free transcription remaining.",
            minutes, seconds
        )
    };

    bot.send_message(msg.chat.id, message).await?;
    Ok(())
}

pub async fn set_typefully_key(bot: Bot, dialog: BotDialogue, msg: Message) -> HandlerResult {
    dialog.update(State::WaitingForTypefullyApiKey).await?;

    bot.send_message(
        msg.chat.id,
        "Please provide your new Typefully API key. You can get it from https://typefully.com, go to settings -> API & Integrations.",
    )
    .await?;

    Ok(())
}

pub async fn toggle_rewrite(bot: Bot, db: Arc<Database>, msg: Message) -> HandlerResult {
    let user = user_extractor(&bot, &db, &msg).await?;
    let new_value = user.toggle_rewrite(&db).await?;

    let status = if new_value { "enabled" } else { "disabled" };

    bot.send_message(
        msg.chat.id,
        format!(
            "AI rewriting is now {}. When {}, the bot will {}.",
            status,
            status,
            if new_value {
                "enhance and rewrite your voice notes for better social media impact"
            } else {
                "only format your voice notes without changing the content"
            }
        ),
    )
    .await?;

    Ok(())
}
