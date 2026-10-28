use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn citadel(api: &Api) -> Result<Value> {
    api.rpc("api_citadel", json!({}))
}

pub fn contribute(api: &Api, resource: &str, amount: u64) -> Result<Value> {
    api.rpc(
        "api_citadel_contribute",
        json!({ "p_d": resource, "p_amount": amount }),
    )
}
