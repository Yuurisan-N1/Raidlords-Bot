use serde_json::{json, Value};

use crate::net::Api;
use crate::quests::quest::is_ready;
use crate::support::error::Result;

pub fn events(api: &Api) -> Result<Value> {
    api.rpc("api_events", json!({}))
}

pub fn event_claim(api: &Api, id: &str) -> Result<Value> {
    api.rpc("api_event_claim", json!({ "p_id": id }))
}

pub fn season(api: &Api) -> Result<Value> {
    api.rpc("api_season", json!({}))
}

pub fn trials(api: &Api) -> Result<Value> {
    api.rpc("api_trials", json!({}))
}

pub fn trial_attack(api: &Api, level: i64) -> Result<Value> {
    api.rpc("api_trial_attack", json!({ "p_lvl": level }))
}

pub fn tourney(api: &Api) -> Result<Value> {
    api.rpc("api_tourney", json!({}))
}

pub fn tourney_join(api: &Api) -> Result<Value> {
    api.rpc("api_tourney_join", json!({}))
}

pub fn campaign(api: &Api) -> Result<Value> {
    api.rpc("api_campaign", json!({}))
}

pub fn campaign_attack(api: &Api, level: i64) -> Result<Value> {
    api.rpc("api_campaign_attack", json!({ "p_lvl": level }))
}

pub fn waitlist_claim(api: &Api) -> Result<Value> {
    api.rpc("api_waitlist_claim", json!({}))
}

pub fn waitlist_prize(api: &Api) -> Result<Value> {
    api.rpc("api_waitlist_prize", json!({}))
}

pub fn vet_claim(api: &Api, day: i64) -> Result<Value> {
    api.rpc("api_vet_claim", json!({ "p_day": day }))
}

pub fn war_pass_claim(api: &Api, day: i64) -> Result<Value> {
    api.rpc("api_war_pass_claim", json!({ "p_day": day }))
}

pub fn burn_goal_claim(api: &Api) -> Result<Value> {
    api.rpc("api_burn_goal_claim", json!({}))
}

pub fn reward_take(api: &Api, gold: u64) -> Result<Value> {
    api.rpc("api_reward_take", json!({ "p_gold": gold }))
}

pub fn open_ids(reply: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk(reply, &mut out);
    out
}

fn walk(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, out);
            }
        }
        Value::Object(map) => {
            if is_ready(map) {
                if let Some(id) = map.get("id").or_else(|| map.get("key")) {
                    let label = id
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| id.to_string());
                    if !out.contains(&label) {
                        out.push(label);
                    }
                }
                return;
            }
            for (_, item) in map {
                if item.is_array() || item.is_object() {
                    walk(item, out);
                }
            }
        }
        _ => {}
    }
}

pub fn flag(reply: &Value, key: &str) -> bool {
    reply.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ready_event_comes_back_once() {
        let reply = json!({"events": [{"id": "moon", "claimable": true}, {"id": "moon", "claimable": true}]});
        assert_eq!(open_ids(&reply), vec!["moon".to_string()]);
    }

    #[test]
    fn a_closed_event_stays_out() {
        assert!(open_ids(&json!({"events": [{"id": "moon", "claimable": false}]})).is_empty());
    }
}
