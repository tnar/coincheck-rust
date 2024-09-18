# Start of Selection
# Coincheck Rust

A high-frequency algorithmic trading bot for [Coincheck](https://coincheck.com/), a major Japanese cryptocurrency exchange. Built with Rust, this project leverages asynchronous programming and WebSockets to handle real-time data for efficient trading operations.

## Features

- **Real-Time Data Handling:** Connects to Coincheck's WebSocket API to receive live order book and trade updates.
- **Automated Trading:** Automatically places buy and sell orders based on the best bid and ask prices.
- **Order Management:** Manages active orders, updates balances, and handles order executions.
- **Graceful Shutdown:** Listens for termination signals to cancel active orders and safely exit.
- **Configurable Parameters:** Easily adjust trading parameters such as order size and price increments.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (comes with Rust)

### Clone the Repository

```sh
git clone https://github.com/yourusername/coincheck-rust.git
cd coincheck-rust
```

### Set Up Environment Variables

Create a `.env` file in the root directory and add your Coincheck API credentials:

```env
API_KEY=your_coincheck_api_key
SECRET_KEY=your_coincheck_secret_key
```

### Build and Run

You can run the application using Cargo:

```sh
RUST_LOG=debug,rustls=off,reqwest=off cargo run
```

## Configuration

The trading parameters can be configured in the `Config` struct found in `src/config.rs`. The default parameters are set as follows:

- **Symbol:** `btc_jpy`
- **Size:** `0.02`
- **Max Size:** `1.0`
- **Price Increment:** `0.00000001`

Modify these values as needed to suit your trading strategy.

## Usage

Once the application is running, it will:

1. Connect to Coincheck's WebSocket API.
2. Subscribe to the order book and trade channels for the specified symbol.
3. Continuously monitor market data to place and manage buy/sell orders.
4. Handle order executions and update balances accordingly.
5. Listen for termination signals (e.g., SIGINT, SIGTERM) to cancel active orders and exit gracefully.

## Logging

The application uses the `env_logger` crate for logging. The log level can be adjusted via the `RUST_LOG` environment variable. For example:

```sh
RUST_LOG=info cargo run
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

### Guidelines

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Commit your changes with clear messages.
4. Submit a pull request detailing your changes.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgements

- [Tokio](https://tokio.rs/) for asynchronous runtime.
- [Reqwest](https://reqwest.rs/) for HTTP requests.
- [Serde](https://serde.rs/) for serialization/deserialization.
- [Coincheck](https://coincheck.com/) for providing the API.
# End of Selection
```
