use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct YahooFinanceResponse {
    pub chart: Chart,
}

#[derive(Deserialize, Debug)]
pub struct Chart {
    pub result: Vec<Result>,
}

#[derive(Deserialize, Debug)]
pub struct Result {
    pub meta: Meta,
    pub indicators: Indicators,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub currency: String,
    pub symbol: String,
    #[serde(rename(deserialize = "regularMarketPrice"))]
    pub regular_market_price: f64,
}

#[derive(Deserialize, Debug)]
pub struct Indicators {
    pub quote: Vec<Quote>,
}

#[derive(Deserialize, Debug)]
pub struct Quote {
    pub close: Vec<Option<f64>>,
}
