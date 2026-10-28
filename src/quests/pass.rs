use serde_json::{json, Value};

use crate::net::Api;
use crate::quests::quest::is_ready;
use crate::support::error::Result;

pub fn pass(api: &Api) -> Result<Value> {
    api.rpc("api_pass", json!({}))
}

pub fn pass_claim(api: &Api, tier: i64, track: &str) -> Result<Value> {
    api.rpc(
        "api_pass_claim",
        json!({ "p_tier": tier, "p_track": track }),
    )
}

pub fn ready(reply: &Value) -> Vec<(i64, String)> {
    let mut out = Vec::new();
    walk(reply, "free", &mut out);
    out.sort_by_key(|(tier, _)| *tier);
    out
}

fn walk(value: &Value, track: &str, out: &mut Vec<(i64, String)>) {
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, track, out);
            }
        }
        Value::Object(map) => {
            let tier = map.get("tier").and_then(|v| v.as_i64());
            if let Some(tier) = tier {
                if is_ready(map) {
                    let label = map
                        .get("track")
                        .and_then(|v| v.as_str())
                        .unwrap_or(track)
                        .to_string();
                    out.push((tier, label));
                    return;
                }
            }
            for (key, item) in map {
                if item.is_array() || item.is_object() {
                    walk(item, key, out);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_tier_comes_back_sorted() {
        let reply =
            json!({"tiers": [{"tier": 4, "claimable": true}, {"tier": 2, "claimable": true}]});
        assert_eq!(
            ready(&reply),
            vec![(2, "tiers".to_string()), (4, "tiers".to_string())]
        );
    }
}
