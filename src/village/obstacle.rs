use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn clear_obstacle(api: &Api, obstacle: i64) -> Result<Value> {
    api.rpc("api_clear_obstacle", json!({ "p_obstacle": obstacle }))
}
