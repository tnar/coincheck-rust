pub mod cancel_order;
pub mod get_active_orders;
pub mod get_balance;
pub mod get_order_books;
pub mod place_order;

use crate::{config::Config, string_or_float, vec_string_or_float};
use anyhow::Result;
use log::debug;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum CoincheckWebsocketEvent {
    OrderBookUpdateEvent(OrderBookResponse),
    ExecutionEvent(ExecutionResponse),
}

pub type ExecutionResponse = Vec<[String; 8]>;

#[derive(Debug, Deserialize, Clone)]
pub struct Execution {
    pub size: f64,
    pub maker_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderBookResponse(pub String, pub OrderBook);

#[derive(Debug, Deserialize, Clone)]
pub struct OrderBook {
    #[serde(with = "vec_string_or_float")]
    pub bids: Vec<[f64; 2]>,
    #[serde(with = "vec_string_or_float")]
    pub asks: Vec<[f64; 2]>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ask {
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub size: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: usize,
    #[serde(alias = "order_type")]
    pub side: String,
    #[serde(alias = "rate")]
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(alias = "amount")]
    #[serde(alias = "pending_amount")]
    #[serde(with = "string_or_float")]
    pub size: f64,
}

pub struct State {
    pub symbol: String,
    pub btc_balance: f64,
    pub buy_order: Option<Order>,
    pub sell_order: Option<Order>,
    pub order_book: Option<OrderBook>,
    pub best_ask_price: Option<f64>,
    pub best_bid_price: Option<f64>,
}

impl State {
    pub fn new(symbols: &str) -> Result<State> {
        Ok(State {
            symbol: symbols.to_string(),
            btc_balance: 0.0,
            buy_order: None,
            sell_order: None,
            order_book: None,
            best_ask_price: None,
            best_bid_price: None,
        })
    }

    pub async fn get_btc_balance(&mut self, config: &Config) -> Result<()> {
        if let Some(balance) = get_balance::balance().await? {
            if let (Some(btc), Some(btc_reserved)) = (balance.btc, balance.btc_reserved) {
                self.btc_balance =
                    ((btc + btc_reserved) * config.size_base).round() / config.size_base;
            }
        }
        Ok(())
    }

    pub async fn get_active_orders(&mut self) -> Result<()> {
        if let Some(orders) = get_active_orders::opens().await? {
            let (mut buy_orders, mut sell_orders) = (Vec::new(), Vec::new());

            orders
                .into_iter()
                .for_each(|order| match order.side.as_ref() {
                    "buy" => buy_orders.push(order),
                    "sell" => sell_orders.push(order),
                    _ => (),
                });

            async fn get_order(orders: &mut Vec<Order>) -> Result<Option<Order>> {
                match orders.len() {
                    0 => Ok(None),
                    1 => Ok(Some(orders[0].clone())),
                    // If there are multiple orders, pop one and cancel the rest
                    _ => {
                        if let Some(order) = orders.pop() {
                            for order in orders {
                                cancel_order::cancel_order(order.id).await?;
                            }
                            Ok(Some(order))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }

            self.buy_order = get_order(&mut buy_orders).await?;
            self.sell_order = get_order(&mut sell_orders).await?;
        }

        Ok(())
    }

    pub fn handle_execution_events(
        &mut self,
        execution_response: ExecutionResponse,
        config: &Config,
    ) {
        // Convert the execution response into a vector of Execution objects
        let executions: Vec<Execution> = execution_response
            .into_iter()
            .map(|exec| Execution {
                size: exec[4].parse().unwrap(),
                maker_id: exec[7].clone(),
            })
            .collect();

        executions.into_iter().for_each(|exec| {
            if let Some(order) = &mut self.buy_order {
                // Check if the maker ID of the execution matches the ID of the buy order
                if order.id == exec.maker_id.parse::<usize>().unwrap() {
                    if order.size == exec.size {
                        // If the sizes match, remove the buy order from the state
                        self.buy_order = None;
                    } else {
                        // If the sizes do not match, update the size of the buy order
                        order.size = ((order.size - exec.size) * config.size_base).round()
                            / config.size_base;
                    }
                    // Update the BTC balance in the state
                    self.btc_balance = ((self.btc_balance + exec.size) * config.size_base).round()
                        / config.size_base;
                    debug!("self.btc_balance: {}", self.btc_balance);
                }
            }
            if let Some(order) = &mut self.sell_order {
                // Check if the maker ID of the execution matches the ID of the sell order
                if order.id == exec.maker_id.parse::<usize>().unwrap() {
                    if order.size == exec.size {
                        // If the sizes match, remove the sell order from the state
                        self.sell_order = None;
                    } else {
                        // If the sizes do not match, update the size of the sell order
                        order.size = ((order.size - exec.size) * config.size_base).round()
                            / config.size_base;
                    }
                    // Update the BTC balance in the state
                    self.btc_balance = ((self.btc_balance - exec.size) * config.size_base).round()
                        / config.size_base;
                    debug!("self.btc_balance: {}", self.btc_balance);
                }
            }
        });
    }

    pub fn update_order_book(&mut self, delta_data: &OrderBook) {
        if let Some(order_book_data) = &mut self.order_book {
            // Iterate over the bids in the new data
            for delta_bid in delta_data.bids.iter() {
                // Flag to track if the bid was found in the existing order book
                let mut found = false;
                // Iterate over the bids in the existing order book
                for bid in order_book_data.bids.iter_mut() {
                    // Check if the bid price matches
                    if bid[0] == delta_bid[0] {
                        // Set the found flag to true
                        found = true;
                        // Update the bid size with the new value
                        bid[1] = delta_bid[1];
                        // Break out of the inner loop
                        break;
                    }
                }
                // If the bid was not found and its size is not zero, add it to the order book
                if !found && delta_bid[1] != 0.0 {
                    order_book_data.bids.push(*delta_bid);
                }
            }
            // Remove any bids with a size of zero from the order book
            order_book_data.bids.retain(|&x| x[1] != 0.0);
            // Sort the bids in descending order by price
            order_book_data
                .bids
                .sort_by(|a, b| b[0].partial_cmp(&a[0]).unwrap());

            // Iterate over the asks in the new data
            for delta_ask in delta_data.asks.iter() {
                // Flag to track if the ask was found in the existing order book
                let mut found = false;
                // Iterate over the asks in the existing order book
                for ask in order_book_data.asks.iter_mut() {
                    // Check if the ask price matches
                    if ask[0] == delta_ask[0] {
                        // Set the found flag to true
                        found = true;
                        // Update the ask size with the new value
                        ask[1] = delta_ask[1];
                        // Break out of the inner loop
                        break;
                    }
                }
                // If the ask was not found and its size is not zero, add it to the order book
                if !found && delta_ask[1] != 0.0 {
                    order_book_data.asks.push(*delta_ask);
                }
            }
            // Remove any asks with a size of zero from the order book
            order_book_data.asks.retain(|&x| x[1] != 0.0);
            // Sort the asks in ascending order by price
            order_book_data
                .asks
                .sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

            self.best_bid_price = Some(order_book_data.bids[0][0]);
            self.best_ask_price = Some(order_book_data.asks[0][0]);
        }
    }

    pub async fn execute_orders(&mut self, config: &Config) -> Result<()> {
        if let (Some(best_bid_price), Some(best_ask_price)) =
            (self.best_bid_price, self.best_ask_price)
        {
            if let Some(order) = &self.buy_order {
                // If the best bid price has changed, cancel the existing buy order
                if best_bid_price != order.price {
                    let res = cancel_order::cancel_order(order.id).await?;
                    if res.success {
                        self.buy_order = None;
                    }
                }
            } else if self.btc_balance < 0.005 {
                // If there is no existing buy order and the BTC balance is below the minimum order size, place a new buy order
                self.buy_order =
                    place_order::order(&self.symbol, "buy", best_bid_price, config.size).await?;
            }

            if let Some(order) = &self.sell_order {
                // If the best ask price has changed, cancel the existing sell order
                if best_ask_price != order.price {
                    let res = cancel_order::cancel_order(order.id).await?;
                    if res.success {
                        self.sell_order = None;
                    }
                }
            } else if self.btc_balance >= 0.005 {
                // If there is no existing sell order and the BTC balance is above the minimum order size, place a new sell order
                self.sell_order =
                    place_order::order(&self.symbol, "sell", best_ask_price, self.btc_balance)
                        .await?;
            }
        }

        Ok(())
    }
}
