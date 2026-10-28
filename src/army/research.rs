use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn research(api: &Api, troop: &str) -> Result<Value> {
    api.rpc("api_research", json!({ "p_troop": troop }))
}
