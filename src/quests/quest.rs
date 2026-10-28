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
    collect(reply, "quest", &mut out);
    out
}

fn collect(reply: &Value, kind: &str, out: &mut Vec<(String, String)>) {
    match reply {
        Value::Array(items) => {
            for item in items {
                collect(item, kind, out);
            }
        }
        Value::Object(map) => {
            let id = map.get("id").or_else(|| map.get("key")).map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string())
            });

            if let Some(id) = id {
                if is_ready(map) {
                    let label = map
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or(kind)
                        .to_string();
                    out.push((label, id));
                    return;
                }
            }

            for (key, value) in map {
                if value.is_array() || value.is_object() {
                    collect(value, key, out);
                }
            }
        }
        _ => {}
    }
}

pub fn is_ready(map: &serde_json::Map<String, Value>) -> bool {
    for key in ["claimable", "can_claim", "ready", "claim"] {
        if map.get(key).and_then(|v| v.as_bool()) == Some(true) {
            return true;
        }
    }

    let claimed = map
        .get("claimed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if claimed {
        return false;
    }

    let progress = map
        .get("progress")
        .or_else(|| map.get("count"))
        .and_then(|v| v.as_f64());
    let target = map.get("target").and_then(|v| v.as_f64());

    match (progress, target) {
        (Some(progress), Some(target)) => target > 0.0 && progress >= target,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_quest_is_ready() {
        let reply = json!({"quests": [{"id": "raid_1", "progress": 3, "target": 3}]});
        assert_eq!(
            ready(&reply),
            vec![("quests".to_string(), "raid_1".to_string())]
        );
    }

    #[test]
    fn a_claimed_quest_is_not_ready_again() {
        let reply = json!([{"id": "raid_1", "progress": 3, "target": 3, "claimed": true}]);
        assert!(ready(&reply).is_empty());
    }

    #[test]
    fn an_unfinished_quest_is_not_ready() {
        let reply = json!([{"id": "raid_1", "progress": 1, "target": 3}]);
        assert!(ready(&reply).is_empty());
    }
}
