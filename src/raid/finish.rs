use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn raid_start(api: &Api) -> Result<Value> {
    api.rpc("api_raid_start", json!({}))
}

pub fn raid_attack(api: &Api, destruction: f64, army: &Value) -> Result<Value> {
    api.rpc(
        "api_raid_attack",
        json!({ "p_d": destruction, "p_army": army }),
    )
}

pub fn battle_finish(api: &Api, battle: Value, log: Value, hash: Value) -> Result<Value> {
    api.battle_finish(json!({
        "battle": battle,
        "log": log,
        "hash": hash,
    }))
}
