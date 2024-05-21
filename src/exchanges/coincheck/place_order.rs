use super::Order;
use crate::util::{get_keys, get_timestamp, sign};
use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Response {
    pub success: bool,
    pub id: Option<usize>,
    pub rate: Option<String>,
    pub amount: Option<String>,
    pub order_type: Option<String>,
}

pub async fn order(
    client: &Client,
    symbol: &str,
    side: &str,
    price: f64,
    size: f64,
) -> Result<Option<Order>> {
    let (api_key, secret_key) = get_keys()?;
    let timestamp = get_timestamp()?;
    let endpoint = "https://coincheck.com";
    let path = "/api/exchange/orders";
    let parameters = json!({ "pair": symbol, "order_type": side,  "rate": price, "amount": size, "time_in_force": "post_only" });

    let text = format!("{}{}{}{}", timestamp, endpoint, path, &parameters);
    let sign = sign(&text, &secret_key)?;

    let res: Response = client
        .post(&(endpoint.to_string() + path))
        .header("content-type", "application/json")
        .header("ACCESS-KEY", api_key)
        .header("ACCESS-NONCE", timestamp)
        .header("ACCESS-SIGNATURE", sign)
        .json(&parameters)
        .send()
        .await?
        .json()
        .await?;

    debug!("{:?}", res);

    if res.success {
        if let (Some(id), Some(rate), Some(amount), Some(order_type)) =
            (res.id, res.rate, res.amount, res.order_type)
        {
            let order = Order {
                id,
                side: order_type,
                price: rate.parse()?,
                size: amount.parse()?,
            };
            Ok(Some(order))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
