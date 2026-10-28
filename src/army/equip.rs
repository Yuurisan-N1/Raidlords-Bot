use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn equipment(api: &Api) -> Result<Value> {
    api.rpc("api_equipment", json!({}))
}

pub fn equip_upgrade(api: &Api, item: &str) -> Result<Value> {
    api.rpc("api_equip_upgrade", json!({ "p_item": item }))
}

pub fn equip_set(api: &Api, hero: &str, items: &[String]) -> Result<Value> {
    api.rpc("api_equip_set", json!({ "p_hero": hero, "p_items": items }))
}
