use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn hero_upgrade(api: &Api, hero: &str) -> Result<Value> {
    api.rpc("api_hero_upgrade", json!({ "p_hero": hero }))
}

pub fn champion(api: &Api) -> Result<Value> {
    api.rpc("api_champion", json!({}))
}

pub fn champion_unlock(api: &Api) -> Result<Value> {
    api.rpc("api_champion_unlock", json!({}))
}

pub fn champion_upgrade(api: &Api) -> Result<Value> {
    api.rpc("api_champion_upgrade", json!({}))
}

pub fn champion_stance(api: &Api, stance: &str) -> Result<Value> {
    api.rpc("api_champion_stance", json!({ "p_stance": stance }))
}
