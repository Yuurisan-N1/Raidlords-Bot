use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn upgrade_walls(api: &Api, ids: &[i64], resource: &str) -> Result<Value> {
    api.rpc(
        "api_upgrade_walls",
        json!({ "p_ids": ids, "p_res": resource }),
    )
}
