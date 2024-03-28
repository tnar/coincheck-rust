use anyhow::Result;

pub struct Config {
    pub symbol: String,
    pub size: f64,
    pub max_size: f64,
    pub price_increment: f64,
    pub price_base: f64,
    pub size_increment: f64,
    pub size_base: f64,
}

impl Config {
    pub fn new(
        symbol: &str,
        size: f64,
        price_increment: f64,
        size_increment: f64,
    ) -> Result<Config> {
        Ok(Config {
            symbol: symbol.to_string(),
            size,
            max_size: size * 1.0,
            price_increment,
            price_base: 1.0 / price_increment,
            size_increment,
            size_base: 1.0 / size_increment,
        })
    }
}
