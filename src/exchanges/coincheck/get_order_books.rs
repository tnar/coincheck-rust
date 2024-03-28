use super::OrderBook;
use anyhow::Result;
// use log::debug;

pub async fn get_order_books() -> Result<OrderBook> {
    let res: OrderBook = reqwest::get("https://coincheck.com/api/order_books")
        .await?
        .json()
        .await?;

    // debug!("{:?}", res);
    Ok(res)
}
