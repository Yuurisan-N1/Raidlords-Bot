use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn find_target(api: &Api) -> Result<Value> {
    api.rpc("api_find_target", json!({}))
}

pub fn next_target(api: &Api) -> Result<Value> {
    api.rpc("api_next_target", json!({}))
}

pub fn battle_log(api: &Api) -> Result<Value> {
    api.rpc("api_battle_log", json!({}))
}

pub fn raid_board(api: &Api) -> Result<Value> {
    api.rpc("api_raid_board", json!({}))
}

pub fn raid_state(api: &Api) -> Result<Value> {
    api.rpc("api_raid_state", json!({}))
}
