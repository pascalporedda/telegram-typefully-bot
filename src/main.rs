use commands::{bot_schema, BotCommand, State};
use log::error;
use std::{path::PathBuf, sync::Arc};

use db::Database;

use teloxide::{
    dispatching::dialogue::InMemStorage, prelude::*, types::BotCommand as TeloxideBotCommand,
    utils::command::BotCommands,
};

mod actions;
mod ai;
mod commands;
mod db;

const DOWNLOAD_DIR: &str = "./voice-notes";
const DATABASE_URL: &str = "sqlite:bot.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting Typefully drafting bot...");

    let bot = Bot::from_env();

    // Set bot commands for autocompletion
    let commands = BotCommand::bot_commands();
    let commands: Vec<TeloxideBotCommand> = commands
        .into_iter()
        .map(|cmd| TeloxideBotCommand::new(cmd.command, cmd.description))
        .collect();
    bot.set_my_commands(commands).await?;

    let db = Arc::new(Database::new(DATABASE_URL).await?);

    if !PathBuf::new().join(DOWNLOAD_DIR).exists() {
        tokio::fs::create_dir(DOWNLOAD_DIR).await.map_err(|e| {
            error!("Could not create download directory: {}", e);
            e
        })?;
    }

    Dispatcher::builder(bot, bot_schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
