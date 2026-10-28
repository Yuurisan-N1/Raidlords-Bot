use serde_json::json;

use crate::net::Api;
use crate::support::error::{Error, Result};

pub fn message(api: &Api, address: &str) -> Result<String> {
    let reply = api.wallet_auth(json!({ "wallet": address }))?;

    if let Some(text) = reply.get("message").and_then(|v| v.as_str()) {
        return Ok(text.to_string());
    }

    if let Some(nonce) = reply.get("nonce").and_then(|v| v.as_str()) {
        return Ok(nonce.to_string());
    }

    Err(Error::Auth(
        "the sign in message was not in the reply".to_string(),
    ))
}
