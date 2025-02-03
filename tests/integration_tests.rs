use sma_tracker::services::sma_service::fetch_and_compare_sma200;

use serde_json::json;
use sma_tracker::errors::app_error::AppError;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fetch_and_compare_sma200_valid_response() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock a valid API response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chart": {
                "result": [
                    {
                        "meta": {
                            "currency": "USD",
                            "symbol": "^GSPC",
                            "regularMarketPrice": 4500.0
                        },
                        "indicators": {
                            "quote": [
                                {
                                    "close": vec![Some(4400.0); 200] // 200 data points
                                }
                            ]
                        }
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fetch_and_compare_sma200_no_data() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock an empty API response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chart": {
                "result": []
            }
        })))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(matches!(result, Err(AppError::NoDataAvailable)));
}

#[tokio::test]
async fn test_fetch_and_compare_sma200_insufficient_data() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock an API response with insufficient data
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chart": {
                "result": [
                    {
                        "meta": {
                            "currency": "USD",
                            "symbol": "^GSPC",
                            "regularMarketPrice": 4500.0
                        },
                        "indicators": {
                            "quote": [
                                {
                                    "close": vec![Some(4400.0); 199] // 199 data points
                                }
                            ]
                        }
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(matches!(result, Err(AppError::InsufficientData)));
}

#[tokio::test]
async fn test_fetch_and_compare_sma200_invalid_json() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock an invalid JSON response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(matches!(result, Err(AppError::ReqwestError(_))));
}

#[tokio::test]
async fn test_fetch_and_compare_sma200_api_error() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock an API error response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(matches!(result, Err(AppError::ReqwestError(_))));
}

#[tokio::test]
async fn test_fetch_and_compare_sma200_mixed_data() {
    // Mock the Yahoo Finance API
    let mock_server = MockServer::start().await;

    // Mock an API response with mixed valid and invalid data
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/%5EGSPC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "chart": {
                "result": [
                    {
                        "meta": {
                            "currency": "USD",
                            "symbol": "^GSPC",
                            "regularMarketPrice": 4500.0
                        },
                        "indicators": {
                            "quote": [
                                {
                                    "close": vec![Some(4400.0), None, Some(4300.0)]
                                }
                            ]
                        }
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Override the API URL to use the mock server
    std::env::set_var("YAHOO_FINANCE_API_URL", mock_server.uri());

    // Call the function
    let result = fetch_and_compare_sma200().await;

    // Assert the result
    assert!(matches!(result, Err(AppError::InsufficientData)));
}
