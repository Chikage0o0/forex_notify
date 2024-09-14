use serde::de::Error as _;
use serde_json::Value;
use snafu::ResultExt;
use snafu::Snafu;

const API_URL: &str = "https://api.twelvedata.com/price";

pub async fn get_realtime_price(api_key: &str, symbol: &str) -> Result<f64, Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(API_URL)
        .query(&[("symbol", symbol), ("apikey", api_key)])
        .send()
        .await
        .context(GetRealtimeApiSnafu)?;

    let response = response.text().await.context(GetRealtimeApiSnafu)?;
    let json: Value = serde_json::from_str(&response).context(ParseJsonSnafu {
        text: response.clone(),
    })?;

    let price = json
        .get("price")
        .and_then(Value::as_str)
        .ok_or(serde_json::Error::custom("price not a string"))
        .context(ParseJsonSnafu {
            text: response.clone(),
        })?;

    let price = price
        .parse::<f64>()
        .map_err(|_| serde_json::Error::custom("price not a number"))
        .context(ParseJsonSnafu {
            text: response.clone(),
        })?;

    Ok(price)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to get the real-time price: {}", source))]
    GetRealtimeApi { source: reqwest::Error },

    #[snafu(display("Failed to parse the JSON response: {}\n{}", source, text))]
    ParseJson {
        source: serde_json::Error,
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_realtime_price() {
        let api_key = "demo";
        let symbol = "USD/JPY";
        let price = get_realtime_price(api_key, symbol).await.unwrap();
        assert!(price > 0.0);
    }
}
