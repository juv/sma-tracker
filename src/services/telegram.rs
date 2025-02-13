use crate::models::telegram_commands::Command;
use teloxide::{prelude::*, utils::command::BotCommands};

pub async fn reply(bot: Bot, msg: Message, cmd: Command) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        Command::Help => Command::descriptions().to_string(),
        Command::Fetch => "axax".to_string(),
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
