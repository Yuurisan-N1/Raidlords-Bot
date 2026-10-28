use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn treasury(api: &Api) -> Result<Value> {
    api.rpc("api_treasury", json!({}))
}

pub fn sell_gold(api: &Api, gold: u64, min_token: f64) -> Result<Value> {
    api.rpc(
        "api_sell_gold",
        json!({ "p_gold": gold, "p_min_token": min_token }),
    )
}

pub fn buy_gold(api: &Api, token: f64, min_gold: u64) -> Result<Value> {
    api.rpc(
        "api_buy_gold",
        json!({ "p_token": token, "p_min_gold": min_gold }),
    )
}

pub fn price_of(reply: &Value) -> Option<f64> {
    for key in ["price", "token_price", "min_token", "rate"] {
        if let Some(value) = reply.get(key).and_then(|v| v.as_f64()) {
            if value > 0.0 {
                return Some(value);
            }
        }
    }
    None
}

pub fn gold_for_room(room_usd: f64, gold: u64, reserve: u64) -> Option<u64> {
    if room_usd <= 0.0 {
        return None;
    }
    let spare = gold.saturating_sub(reserve);
    if spare < 500 {
        return None;
    }
    Some(spare)
}
