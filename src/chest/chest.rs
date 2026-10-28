use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn chest(api: &Api) -> Result<Value> {
    api.rpc("api_chest", json!({}))
}

pub fn chest_lock(api: &Api, amount: u64, days: u64) -> Result<Value> {
    api.rpc(
        "api_chest_lock",
        json!({ "p_amount": amount, "p_days": days }),
    )
}

pub fn chest_unlock(api: &Api, chest: &str) -> Result<Value> {
    api.rpc("api_chest_unlock", json!({ "p_chest": chest }))
}

pub fn ready(reply: &Value) -> Option<String> {
    let chests = reply.get("chests").and_then(|v| v.as_array())?;
    for item in chests {
        let ready = item
            .get("ready")
            .and_then(|v| v.as_bool())
            .or_else(|| item.get("unlockable").and_then(|v| v.as_bool()))
            .unwrap_or(false);
        if ready {
            if let Some(id) = item.get("id").or_else(|| item.get("key")) {
                return Some(
                    id.as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| id.to_string()),
                );
            }
        }
    }
    None
}

pub fn lock_room(reply: &Value) -> Option<u64> {
    let amount = reply.get("lock_amount").and_then(|v| v.as_u64())?;
    let days = reply.get("lock_days").and_then(|v| v.as_u64()).unwrap_or(1);
    if amount > 0 && days > 0 {
        Some(days)
    } else {
        None
    }
}
