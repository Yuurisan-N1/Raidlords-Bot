use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn champ_armory(api: &Api) -> Result<Value> {
    api.rpc("api_champ_armory", json!({}))
}

pub fn champ_gear_buy(api: &Api, item: &str) -> Result<Value> {
    api.rpc("api_champ_gear_buy", json!({ "p_item": item }))
}

pub fn champ_gear_upgrade(api: &Api, item: &str) -> Result<Value> {
    api.rpc("api_champ_gear_upgrade", json!({ "p_item": item }))
}

pub fn champ_gear_wear(api: &Api, item: &str) -> Result<Value> {
    api.rpc("api_champ_gear_wear", json!({ "p_item": item }))
}
