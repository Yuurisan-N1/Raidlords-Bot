use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn raid_board(api: &Api) -> Result<Value> {
    api.rpc("api_raid_board", json!({}))
}
