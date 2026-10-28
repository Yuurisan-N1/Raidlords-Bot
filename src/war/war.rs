use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn war(api: &Api) -> Result<Value> {
    api.rpc("api_war", json!({}))
}

pub fn war_attack(api: &Api, position: i64) -> Result<Value> {
    api.rpc("api_war_attack", json!({ "p_pos": position }))
}

pub fn open_positions(reply: &Value) -> Vec<i64> {
    let mut out = Vec::new();
    let items = reply
        .get("members")
        .or_else(|| reply.get("opponents"))
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for (index, item) in items.iter().enumerate() {
            let used = item
                .get("attacks")
                .and_then(|v| v.as_u64())
                .map(|v| v > 0)
                .unwrap_or(false);
            let stars = item.get("stars").and_then(|v| v.as_u64()).unwrap_or(0);
            if !used && stars < 3 {
                let position = item
                    .get("pos")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(index as i64);
                out.push(position);
            }
        }
    }

    out
}
