use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn collect(api: &Api) -> Result<Value> {
    api.rpc("api_collect", json!({}))
}
