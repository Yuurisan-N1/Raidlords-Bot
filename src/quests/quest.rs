use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn quests(api: &Api) -> Result<Value> {
    api.rpc("api_quests", json!({}))
}

pub fn quest_claim(api: &Api, kind: &str, id: &str) -> Result<Value> {
    api.rpc("api_quest_claim", json!({ "p_kind": kind, "p_id": id }))
}

pub fn ready(reply: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for kind in ["starter", "daily"] {
        for item in board(reply, kind) {
            if let Some(id) = id_of(item) {
                if is_ready(item) {
                    out.push((kind.to_string(), id));
                }
            }
        }
    }
    for item in board(reply, "achievements") {
        if let Some(id) = id_of(item) {
            if tier_open(item) {
                out.push(("achievement".to_string(), id));
            }
        }
    }
    out
}

fn board<'a>(reply: &'a Value, key: &str) -> Vec<&'a Value> {
    reply
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn id_of(item: &Value) -> Option<String> {
    item.get("id")
        .and_then(|v| v.as_str())
        .map(|text| text.to_string())
}

fn tier_open(item: &Value) -> bool {
    let tiers: Vec<f64> = item
        .get("tiers")
        .and_then(|v| v.as_array())
        .map(|items| items.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    let taken = item
        .get("claimed")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        .max(0.0) as usize;
    let value = item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
    match tiers.get(taken) {
        Some(next) => value >= *next,
        None => false,
    }
}

pub fn is_ready(item: &Value) -> bool {
    let map = match item.as_object() {
        Some(map) => map,
        None => return false,
    };
    for key in ["claimable", "can_claim", "ready"] {
        if map.get(key).and_then(|v| v.as_bool()) == Some(true) {
            return true;
        }
    }
    if claimed(map) {
        return false;
    }
    let value = map
        .get("value")
        .or_else(|| map.get("progress"))
        .or_else(|| map.get("count"))
        .and_then(|v| v.as_f64());
    let target = map
        .get("n")
        .or_else(|| map.get("target"))
        .and_then(|v| v.as_f64());
    match (value, target) {
        (Some(value), Some(target)) => target > 0.0 && value >= target,
        _ => false,
    }
}

fn claimed(map: &serde_json::Map<String, Value>) -> bool {
    match map.get("claimed") {
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(count)) => count.as_f64().unwrap_or(0.0) > 0.0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_starter_quest_is_ready() {
        let reply = json!({"starter": [
            {"id": "s_one", "n": 1, "value": 1, "claimed": false},
            {"id": "s_two", "n": 2, "value": 0, "claimed": false}
        ]});
        assert_eq!(
            ready(&reply),
            vec![("starter".to_string(), "s_one".to_string())]
        );
    }

    #[test]
    fn a_claimed_quest_is_not_ready_again() {
        let reply = json!({"daily": [{"id": "d_one", "n": 3, "value": 3, "claimed": true}]});
        assert!(ready(&reply).is_empty());
    }

    #[test]
    fn an_unfinished_quest_is_not_ready() {
        let reply = json!({"daily": [{"id": "d_one", "n": 3, "value": 2, "claimed": false}]});
        assert!(ready(&reply).is_empty());
    }

    #[test]
    fn an_achievement_counts_the_tiers_it_already_took() {
        let reply = json!({"achievements": [
            {"id": "a_one", "value": 6, "claimed": 0, "tiers": [5, 30, 100]},
            {"id": "a_two", "value": 6, "claimed": 1, "tiers": [5, 30, 100]},
            {"id": "a_three", "value": 1, "claimed": 0, "tiers": [5, 30, 100]}
        ]});
        assert_eq!(
            ready(&reply),
            vec![("achievement".to_string(), "a_one".to_string())]
        );
    }

    #[test]
    fn the_daily_board_keeps_its_own_kind() {
        let reply = json!({"daily": [{"id": "d_one", "n": 1, "value": 1, "claimed": false}]});
        assert_eq!(
            ready(&reply),
            vec![("daily".to_string(), "d_one".to_string())]
        );
    }
}
