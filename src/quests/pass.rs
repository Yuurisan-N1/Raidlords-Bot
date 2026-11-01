use serde_json::{json, Value};

use crate::net::Api;
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
    let reached = reply.get("reached").and_then(|v| v.as_i64()).unwrap_or(0);
    let premium = reply
        .get("premium")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let tiers = match reply.get("tiers").and_then(|v| v.as_array()) {
        Some(tiers) => tiers,
        None => return out,
    };

    for item in tiers {
        let tier = match item.get("tier").and_then(|v| v.as_i64()) {
            Some(tier) => tier,
            None => continue,
        };
        if tier > reached {
            continue;
        }
        if !flag(item, "free_claimed") {
            out.push((tier, "free".to_string()));
        }
        if premium && !flag(item, "premium_claimed") {
            out.push((tier, "premium".to_string()));
        }
    }

    out.sort_by_key(|(tier, track)| (*tier, track.clone()));
    out
}

fn flag(item: &Value, key: &str) -> bool {
    item.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Value {
        json!({
            "reached": 2,
            "premium": false,
            "tiers": [
                {"tier": 1, "free": {"gold": 100}, "premium": {"gold": 200},
                 "free_claimed": true, "premium_claimed": false},
                {"tier": 2, "free": {"gold": 300}, "premium": {"gold": 400},
                 "free_claimed": false, "premium_claimed": false},
                {"tier": 3, "free": {"gold": 500}, "premium": {"gold": 600},
                 "free_claimed": false, "premium_claimed": false}
            ]
        })
    }

    #[test]
    fn only_the_unlocked_and_unclaimed_tiers_come_back() {
        assert_eq!(ready(&board()), vec![(2, "free".to_string())]);
    }

    #[test]
    fn a_premium_pass_also_takes_the_premium_track() {
        let mut reply = board();
        reply["premium"] = json!(true);
        assert_eq!(
            ready(&reply),
            vec![
                (1, "premium".to_string()),
                (2, "free".to_string()),
                (2, "premium".to_string())
            ]
        );
    }

    #[test]
    fn a_board_without_tiers_has_nothing_to_claim() {
        assert!(ready(&json!({"reached": 9})).is_empty());
    }
}
