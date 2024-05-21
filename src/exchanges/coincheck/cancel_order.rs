use crate::util::{get_keys, get_timestamp, sign};
use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub success: bool,
}

pub async fn cancel_order(client: &Client, id: usize) -> Result<Response> {
    let (api_key, secret_key) = get_keys()?;
    let timestamp = get_timestamp()?;
    let endpoint = "https://coincheck.com";
    let path = format!("/api/exchange/orders/{}", id);

    let text = format!("{}{}{}", timestamp, endpoint, path);
    let sign = sign(&text, &secret_key)?;

    let res: Response = client
        .delete(&(endpoint.to_string() + &path))
        .header("content-type", "application/json")
        .header("ACCESS-KEY", api_key)
        .header("ACCESS-NONCE", timestamp)
        .header("ACCESS-SIGNATURE", sign)
        .send()
        .await?
        .json()
        .await?;

    debug!("{:?}", res);

    Ok(res)
}
