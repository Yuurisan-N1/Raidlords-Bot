use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn champ_arena(api: &Api) -> Result<Value> {
    api.rpc("api_champ_arena", json!({}))
}

pub fn champ_arena_start(api: &Api) -> Result<Value> {
    api.rpc("api_champ_arena_start", json!({}))
}

pub fn has_tickets(reply: &Value) -> bool {
    reply
        .get("tickets")
        .and_then(|v| v.as_f64())
        .map(|v| v > 0.0)
        .unwrap_or(false)
}
