use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn withdraw(api: &Api, amount: f64) -> Result<Value> {
    api.rpc("api_withdraw", json!({ "p_amount": amount }))
}

pub fn room_usd(reply: &Value) -> f64 {
    reply
        .get("room_usd")
        .or_else(|| reply.get("sell").and_then(|v| v.get("room_usd")))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}
