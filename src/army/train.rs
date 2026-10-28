use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn set_army(api: &Api, army: &Value) -> Result<Value> {
    api.rpc("api_set_army", json!({ "p_army": army }))
}

pub fn train_army(api: &Api) -> Result<Value> {
    api.rpc("api_train_army", json!({}))
}

pub fn army_status(api: &Api) -> Result<Value> {
    api.rpc("api_army_status", json!({}))
}
