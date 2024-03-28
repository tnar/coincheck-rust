use anyhow::Result;
use dotenv::dotenv;
use ring::hmac;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// Function to get the API and secret keys from the environment
pub fn get_keys() -> Result<(String, String)> {
    // Load environment variables from a .env file, if present
    dotenv().ok();
    let api_key = env::var("API_KEY")?;
    let secret_key = env::var("SECRET_KEY")?;
    Ok((api_key, secret_key))
}

// Function to sign a text message using HMAC-SHA256 and a secret key
pub fn sign(text: &str, secret_key: &str) -> Result<String> {
    // Create a new HMAC-SHA256 key using the secret key
    let signed_key = hmac::Key::new(hmac::HMAC_SHA256, secret_key.as_bytes());
    // Sign the text message using the key
    let signature = hmac::sign(&signed_key, text.as_bytes());
    // Encode the signature as a hexadecimal string and return it
    Ok(hex::encode(signature.as_ref()))
}

// Function to get the current timestamp in milliseconds since the Unix epoch
pub fn get_timestamp() -> Result<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_millis() as u64)
}
