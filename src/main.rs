use anyhow::Result;
use coincheck_rust::{
    config::Config,
    exchanges::coincheck::{self, CoincheckWebsocketEvent},
};
use futures_util::{SinkExt, StreamExt};
use log::debug;
use serde_json::json;
use std::{process, time::Duration};
use tokio::signal::unix::SignalKind;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

static SYMBOL: &str = "btc_jpy";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Config::new(SYMBOL, 0.02, 1.0, 0.00000001)?;
    let mut state = coincheck::State::new(SYMBOL)?;

    // Connect to the Coincheck WebSocket API
    let (coincheck_stream, _) = connect_async("wss://ws-api.coincheck.com").await?;
    let (mut coincheck_write, mut coincheck_read) = coincheck_stream.split();

    // Subscribe to the order book for the specified symbol
    coincheck_write
        .send(Message::Text(
            json!({
                "type": "subscribe",
                "channel": "btc_jpy-orderbook",
            })
            .to_string(),
        ))
        .await?;
    // Wait for 1 second
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Subscribe to the trades for the specified symbol
    coincheck_write
        .send(Message::Text(
            json!({
                "type": "subscribe",
                "channel": "btc_jpy-trades",
            })
            .to_string(),
        ))
        .await?;

    // Set the interval for getting the state to 20 seconds. This is for recovering from occasional misbehavior.
    let mut get_state_interval = tokio::time::interval(Duration::from_secs(20));
    // Set the interval for executing orders to 200 milliseconds
    let mut execute_orders_interval = tokio::time::interval(Duration::from_millis(200));
    // Set up signal handlers for SIGTERM and SIGINT
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;

    let orderbook = coincheck::get_order_books::get_order_books().await?;
    state.order_book = Some(orderbook);

    loop {
        // Use the `select!` macro to wait for multiple events
        tokio::select! {
            // Wait for a message from the Coincheck WebSocket API
            msg = coincheck_read.next() => {
                if let Some(msg) = msg {
                    debug!("msg: {:?}", msg);
                    match msg? {
                        // If the message is a ping, send a pong response
                        Message::Ping(ping) => {
                            coincheck_write.send(Message::Pong(ping)).await?;
                        }
                        // If the message is text, parse it as JSON
                        Message::Text(text) =>  {
                            match serde_json::from_str(&text)? {
                                // If the message is an order book update, update the state
                                CoincheckWebsocketEvent::OrderBookUpdateEvent(order_book_response) => {
                                    let order_book_event = order_book_response.1;
                                    state.update_order_book(&order_book_event);
                                }
                                // If the message is an execution event, handle it
                                CoincheckWebsocketEvent::ExecutionEvent(execution_events) => {
                                    state.handle_execution_events(execution_events, &config);
                                }
                            }
                        }
                        _ => ()
                    }
                }
            }
            // Wait for the get state interval to tick
            _ = get_state_interval.tick() => {
                state.get_btc_balance(&config).await?;
                state.get_active_orders().await?;
            }
            // Wait for the execute orders interval to tick
            _ = execute_orders_interval.tick() => {
                state.execute_orders(&config).await?;
            }
            // Wait for a SIGTERM signal
            _ = sigterm.recv() => {
                if let Some(order) = state.buy_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                if let Some(order) = state.sell_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                coincheck_write.close().await?;
                process::exit(0);
            },
            // Wait for a SIGINT signal
            _ = sigint.recv() => {
                if let Some(order) = state.buy_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                if let Some(order) = state.sell_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                coincheck_write.close().await?;
                process::exit(0);
            },
            // Wait for a Ctrl-C signal from the user
            _ = tokio::signal::ctrl_c() => {
                if let Some(order) = state.buy_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                if let Some(order) = state.sell_order {
                    coincheck::cancel_order::cancel_order(order.id).await?;
                }
                coincheck_write.close().await?;
                process::exit(0);
            }
        }
    }
}
