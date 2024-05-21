use super::Order;
use crate::util::{get_keys, get_timestamp, sign};
use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub success: bool,
    pub orders: Option<Vec<Order>>,
}

pub async fn opens(client: &Client) -> Result<Option<Vec<Order>>> {
    let (api_key, secret_key) = get_keys()?;
    let timestamp = get_timestamp()?;
    let endpoint = "https://coincheck.com";
    let path = "/api/exchange/orders/opens";

    let text = format!("{}{}{}", timestamp, endpoint, path);
    let sign = sign(&text, &secret_key)?;

    let res: Response = client
        .get(&(endpoint.to_string() + path))
        .header("content-type", "application/json")
        .header("ACCESS-KEY", api_key)
        .header("ACCESS-NONCE", timestamp)
        .header("ACCESS-SIGNATURE", sign)
        .send()
        .await?
        .json()
        .await?;

    debug!("{:?}", res);

    if res.success {
        Ok(res.orders)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use log::debug;
    use reqwest::Client;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        env_logger::init();

        let client = Client::new();
        let list = opens(&client).await?;
        debug!("{:?}", list);

        Ok(())
    }
}
