use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn revenge(api: &Api, battle: i64) -> Result<Value> {
    api.rpc("api_revenge", json!({ "p_battle": battle }))
}

pub fn replay(api: &Api, battle: i64) -> Result<Value> {
    api.rpc("api_replay", json!({ "p_battle": battle }))
}

pub fn revenge_ids(log: &Value) -> Vec<i64> {
    let mut out = Vec::new();
    let items = match log.as_array() {
        Some(items) => items,
        None => return out,
    };
    for item in items {
        let open = item
            .get("revenge")
            .and_then(|v| v.as_bool())
            .or_else(|| item.get("can_revenge").and_then(|v| v.as_bool()))
            .unwrap_or(false);
        if !open {
            continue;
        }
        if let Some(id) = item.get("id").and_then(|v| v.as_i64()) {
            out.push(id);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_open_entries_come_back() {
        let log = json!([
            {"id": 11, "revenge": true},
            {"id": 12, "revenge": false},
            {"id": 13}
        ]);
        assert_eq!(revenge_ids(&log), vec![11]);
    }
}
