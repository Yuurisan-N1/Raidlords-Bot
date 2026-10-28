use serde_json::{json, Value};

use crate::net::Api;
use crate::quests::quest::is_ready;
use crate::support::error::Result;

pub fn daily(api: &Api) -> Result<Value> {
    api.rpc("api_daily", json!({}))
}

pub fn daily_claim(api: &Api) -> Result<Value> {
    api.rpc("api_daily_claim", json!({}))
}

pub fn ready(reply: &Value) -> bool {
    match reply {
        Value::Array(items) => items.iter().any(ready),
        Value::Object(map) => {
            if is_ready(map) {
                return true;
            }
            map.values().any(ready)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_daily_board_with_one_open_reward_is_ready() {
        assert!(ready(&json!({"days": [{"id": 1, "claimable": true}]})));
        assert!(!ready(&json!({"days": [{"id": 1, "claimable": false}]})));
    }
}
