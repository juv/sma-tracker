mod errors;
mod models;
mod services;
mod utils;

use errors::app_error::AppError;
use log::info;
use services::sma_service::fetch_and_compare_sma200;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::logging::init_logger;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_logger();

    info!("Starting sma-tracker...");

    let execution_mode = env::var("EXECUTION_MODE").unwrap_or_else(|_| "once".into());
    info!("Execution mode: {}", execution_mode);
    match execution_mode.as_str() {
        "cron" => execute_cron_job().await,
        "once" => execute_once().await,
        _ => Err(AppError::UnsupportedExecutionMode)
    }
}

async fn execute_cron_job() -> Result<(), AppError> {
    let cron_schedule =
        env::var("CRON_SCHEDULE").unwrap_or_else(|_| "0 */15 * * * 1-5".to_string());

    // Create a scheduler
    let scheduler = JobScheduler::new().await?;
    info!("Executing job scheduler at cron schedule {}", cron_schedule);

    let tracker_job = Job::new(cron_schedule, |_uuid, mut _l| {
        tokio::spawn(async {
            if let Err(e) = fetch_and_compare_sma200().await {
                log::error!("Error in scheduled job (before close): {}", e);
            }
        });
    })?;

    scheduler.add(tracker_job).await?;

    scheduler.start().await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn execute_once() -> Result<(), AppError> {
    fetch_and_compare_sma200().await
}