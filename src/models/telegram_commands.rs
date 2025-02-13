use std::fmt;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "retrieve current S&P 500 index data.")]
    Fetch,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Help => write!(f, "Help"),
            Command::Fetch => write!(f, "Fetch"),
        }
    }
}
