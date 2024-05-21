use super::OrderBook;
use anyhow::Result;
use reqwest::Client;

pub async fn get_order_books(client: &Client) -> Result<OrderBook> {
    let res: OrderBook = client
        .get("https://coincheck.com/api/order_books")
        .send()
        .await?
        .json()
        .await?;

    // debug!("{:?}", res);
    Ok(res)
}
