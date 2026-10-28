use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn market(api: &Api) -> Result<Value> {
    api.rpc("api_market", json!({}))
}

pub fn market_buy(api: &Api, what: &str, pick: &str) -> Result<Value> {
    api.rpc("api_market_buy", json!({ "p_what": what, "p_pick": pick }))
}
