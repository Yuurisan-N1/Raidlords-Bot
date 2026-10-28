use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn cwl(api: &Api) -> Result<Value> {
    api.rpc("api_cwl", json!({}))
}

pub fn cwl_attack(api: &Api, position: i64) -> Result<Value> {
    api.rpc("api_cwl_attack", json!({ "p_pos": position }))
}
