mod errors;
mod models;
mod services;
mod utils;

use errors::app_error::AppError;
use log::{error, info};
use services::sma_service::fetch_and_compare_sma200;
use std::env;
use std::sync::Arc;
use teloxide::{dptree, Bot};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::{Dispatcher, Requester};
use teloxide::repls::CommandReplExt;
use teloxide::types::{ChatId, Update};
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::logging::init_logger;
use crate::models::telegram_commands::Command;
use crate::services::sma_service::fetch_and_compare_sma200_botless;
use crate::services::telegram;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_logger();

    info!("Starting sma-tracker...");

    let execution_mode = env::var("EXECUTION_MODE").unwrap_or_else(|_| "once".into());
    info!("Execution mode: {}", execution_mode);
    match execution_mode.as_str() {
        "server" => execute_server_mode().await,
        "once" => execute_once().await,
        _ => Err(AppError::UnsupportedExecutionMode)
    }
}

async fn execute_server_mode() -> Result<(), AppError> {
    let bot = Arc::new(initialize_bot().await);

    let telegram_bot_task = tokio::spawn(start_telegram_bot(bot.clone()));

    let cron_job_task = tokio::spawn(start_cron_scheduler(bot.clone()));

    // Wait for either task to complete or fail
    tokio::select! {
        bot_result = telegram_bot_task => {
            // Handle the result of the bot task
            match bot_result {
                Ok(Ok(())) => info!("Telegram bot stopped gracefully."),
                Ok(Err(e)) => {
                    error!("Telegram bot failed: {:?}", e);
                    return Err(e);
                }
                Err(e) => {
                    error!("Telegram bot task panicked: {:?}", e);
                    return Err(AppError::TaskPanicked);
                }
            }
        }
        cron_result = cron_job_task => {
            // Handle the result of the cron task
            match cron_result {
                Ok(Ok(())) => info!("Cron scheduler stopped gracefully."),
                Ok(Err(e)) => {
                    error!("Cron scheduler failed: {:?}", e);
                    return Err(e);
                }
                Err(e) => {
                    error!("Cron scheduler task panicked: {:?}", e);
                    return Err(AppError::TaskPanicked);
                }
            }
        }
    }
    Ok(())
}

async fn start_cron_scheduler(bot: Arc<Bot>) -> Result<(), AppError> {
    let cron_schedule =
        env::var("CRON_SCHEDULE").unwrap_or_else(|_| "0 */15 * * * 1-5".to_string());

    let chat_id =
        env::var("TELOXIDE_CHAT_ID").expect("TELOXIDE_CHAT_ID must be set");

    // Parse the environment variable value into an i64
    let chat_id: i64 = chat_id.parse().unwrap_or_else(|_| {
        panic!("Failed to parse environment variable TELOXIDE_CHAT_ID as i64");
    });

    // Create a scheduler
    let scheduler = JobScheduler::new().await?;
    info!("Executing job scheduler at cron schedule {}", cron_schedule);

    let bot_clone = bot.clone();
    let tracker_job = Job::new(cron_schedule, move |_uuid, mut _l| {
        let bot = bot_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = fetch_and_compare_sma200(&bot, chat_id).await {
                log::error!("Error in scheduled job (before close): {}", e);
            }
        });
    })?;

    scheduler.add(tracker_job).await?;

    scheduler.start().await?;

    // Keep the scheduler running
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C event");

    info!("Cron job scheduler was stopped");
    Ok(())
}

async fn initialize_bot() -> Bot {
    env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");

    Bot::from_env()
}

async fn start_telegram_bot(bot: Arc<Bot>) -> Result<(), AppError> {
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                // Filter commands: the next handlers will receive a parsed `SimpleCommand`.
                .filter_command::<Command>()
                // If a command parsing fails, this handler will not be executed.
                .endpoint(telegram::reply),
        );

    Dispatcher::builder(bot.as_ref().clone(), handler)
        .enable_ctrlc_handler()
        .error_handler(LoggingErrorHandler::with_custom_text("An error occured in telegram listener"))
        // If no handler succeeded to handle an update, this closure will be called.
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .build()
        .dispatch()
        .await;

    info!("Telegram bot was stopped");
    Ok(())
}

async fn execute_once() -> Result<(), AppError> {
    fetch_and_compare_sma200_botless().await
}