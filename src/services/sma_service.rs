use crate::errors::app_error::AppError;
use crate::models::finance_data::FinanceData;
use crate::models::yahoo_finance::YahooFinanceResponse;
use log::{error, info};
use reqwest::header::{ACCEPT, USER_AGENT};
use std::env;
use teloxide::prelude::{ChatId, Requester};
use teloxide::types::InputFile;
use teloxide::Bot;

pub async fn fetch_and_compare_sma200_botless() -> Result<(), AppError> {
    let data = fetch_data().await?;

    match data.current_price.partial_cmp(&data.current_sma200) {
        Some(std::cmp::Ordering::Greater) => info!("überschritten"),
        Some(std::cmp::Ordering::Less) => info!("unterschritten"),
        _ => info!("Gleich"),
    };

    Ok(())
}

pub async fn fetch_and_compare_sma200(bot: &Bot, chat_id: i64) -> Result<(), AppError> {
    let data = fetch_data().await?;

    let comparison = match data.yesterday_close_price.partial_cmp(&data.current_sma200) {
        Some(std::cmp::Ordering::Less) => "🚀🚀🚀🚀🚀 Kaufgelegenheit 🚀🚀🚀🚀🚀",
        Some(std::cmp::Ordering::Greater) => {
            match data.current_price.partial_cmp(&data.current_sma200) {
                Some(std::cmp::Ordering::Greater) => "🫰 zu teuer",
                Some(std::cmp::Ordering::Less) => "🧘 abwarten und ☕ trinken",
                _ => "🤔 Lotto spielen",
            }
        }
        _ => "Something weird happened 🤷‍",
    };

    let message = format!(
        "{}\nAktueller Preis: {:.2}\nSMA 200: {:.2} ({:.2}%)\nYCP: {:.2}",
        comparison,
        data.current_price,
        data.current_sma200,
        ((data.current_sma200 - data.current_price) / data.current_price) * 100.0,
        data.yesterday_close_price
    );

    send_private_message(bot, chat_id, message.as_str()).await?;

    Ok(())
}

pub async fn fetch_data() -> Result<FinanceData, AppError> {
    info!("Fetching S&P 500 data...");

    let api_url = env::var("YAHOO_FINANCE_API_URL")
        .unwrap_or_else(|_| "https://query1.finance.yahoo.com".to_string());

    // Fetch historical data for the S&P 500 (^GSPC) for the last 200 days
    let endpoint = "/v8/finance/chart/%5EGSPC?interval=1d&range=200d";

    // Create a reqwest client
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}{}", api_url, endpoint).as_str())
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "application/json")
        .send()
        .await?
        .json::<YahooFinanceResponse>()
        .await?;

    // Extract the first result from the response
    let result = response.chart.result.first().ok_or_else(|| {
        error!("No data available in the API response");
        AppError::NoDataAvailable
    })?;

    let current_price = result.meta.regular_market_price;

    // Calculate the SMA200
    let close_prices = &result.indicators.quote[0].close;
    let current_sma200 = calculate_sma200(close_prices).ok_or_else(|| {
        error!("Insufficient data to calculate SMA200");
        AppError::InsufficientData
    })?;

    let yesterday_close_price = result.indicators.quote[0].close[198].unwrap_or(0.0);

    info!(
        "Current Price: {}, SMA200: {}",
        current_price, current_sma200
    );

    Ok(FinanceData {
        current_price,
        current_sma200,
        yesterday_close_price,
    })
}

fn calculate_sma200(close_prices: &[Option<f64>]) -> Option<f64> {
    let valid_prices: Vec<f64> = close_prices.iter().filter_map(|&x| x).collect();

    // Ensure we have at least 200 data points
    if valid_prices.len() < 200 {
        return None;
    }

    let sum: f64 = valid_prices.iter().sum();
    Some(sum / valid_prices.len() as f64)
}

async fn send_private_message(bot: &Bot, chat_id: i64, message: &str) -> Result<(), AppError> {
    info!("Sending message \"{}\" to chatId {}", message, chat_id);
    let chat_id = ChatId(chat_id);
    bot.send_message(chat_id, message).await.map_err(|e| {
        log::error!("Failed to send message: {}", e);
        AppError::TeloxideRequestError(e)
    })?;

    Ok(())
}
