use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn forge_collect(api: &Api) -> Result<Value> {
    api.rpc("api_forge_collect", json!({}))
}

pub fn forge_craft(api: &Api, resource: &str) -> Result<Value> {
    api.rpc("api_forge_craft", json!({ "p_res": resource }))
}
