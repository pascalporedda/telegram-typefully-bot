use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    macros::BotCommands,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::actions;

pub type BotDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(BotCommands, Clone, PartialEq, Eq, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split",
    separator = " "
)]
pub enum BotCommand {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Set your OpenAI API key (optional, first 5 minutes are free)")]
    SetApiKey,
    #[command(description = "Set or update your Typefully API key")]
    SetTypefullyKey,
    #[command(description = "Check your remaining free usage")]
    Usage,
    #[command(description = "Toggle between AI rewriting and simple formatting")]
    ToggleRewrite,
    #[command(description = "Start using the bot")]
    Start,
    #[command(description = "Delete your account and all data")]
    DeleteAccount,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    WaitingForTypefullyApiKey,
    WaitingForOpenAiApiKey,
    WaitingForDeleteConfirmation,
    // Registered {
    //     user: User,
    // },
    // ReceivedVoiceNote(Voice),
    // Transcribing(String),
}

pub fn bot_schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(case![BotCommand::Help].endpoint(actions::help))
        .branch(case![BotCommand::Start].endpoint(actions::start))
        .branch(case![BotCommand::SetApiKey].endpoint(actions::set_api_key))
        .branch(case![BotCommand::SetTypefullyKey].endpoint(actions::set_typefully_key))
        .branch(case![BotCommand::Usage].endpoint(actions::usage))
        .branch(case![BotCommand::ToggleRewrite].endpoint(actions::toggle_rewrite))
        .branch(case![BotCommand::DeleteAccount].endpoint(actions::delete_account));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(
            case![State::WaitingForTypefullyApiKey].endpoint(actions::receive_typefully_api_key),
        )
        .branch(case![State::WaitingForOpenAiApiKey].endpoint(actions::receive_openai_api_key))
        .branch(
            case![State::WaitingForDeleteConfirmation]
                .endpoint(actions::handle_delete_confirmation),
        )
        .branch(Message::filter_voice().endpoint(actions::handle_voice_note))
        .branch(dptree::endpoint(actions::invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, _, _>()
        .branch(message_handler)
        .branch(Update::filter_inline_query().endpoint(actions::inline_query_handler))
}

pub fn keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Credits", "credits",
    )]])
}
