use crate::opt_string_or_float;
use crate::util::{get_keys, get_timestamp, sign};
use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub success: bool,
    #[serde(with = "opt_string_or_float")]
    #[serde(default)]
    pub btc: Option<f64>,
    #[serde(with = "opt_string_or_float")]
    #[serde(default)]
    pub btc_reserved: Option<f64>,
}

pub async fn balance() -> Result<Option<Response>> {
    let (api_key, secret_key) = get_keys()?;
    let timestamp = get_timestamp()?;
    let endpoint = "https://coincheck.com";
    let path = "/api/accounts/balance";

    let text = format!("{}{}{}", timestamp, endpoint, path);
    let sign = sign(&text, &secret_key)?;

    let res: Response = Client::new()
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
        Ok(Some(res))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use log::debug;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        env_logger::init();

        let list = balance().await?;
        debug!("{:?}", list);

        Ok(())
    }
}
