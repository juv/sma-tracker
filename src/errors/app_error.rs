use reqwest::Error as ReqwestError;
use thiserror::Error;
use tokio_cron_scheduler::JobSchedulerError as TokioCronScheduleError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] ReqwestError),
    #[error("No data available in the API response")]
    NoDataAvailable,
    #[error("Insufficient data to calculate SMA200")]
    InsufficientData,
    #[error("Scheduler error: {0}")]
    SchedulerError(#[from] TokioCronScheduleError),
}
