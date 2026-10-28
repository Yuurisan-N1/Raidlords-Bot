use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn champ_duel(api: &Api) -> Result<Value> {
    api.rpc("api_champ_duel", json!({}))
}

pub fn champ_duel_start(api: &Api, revenge: bool) -> Result<Value> {
    api.rpc("api_champ_duel_start", json!({ "p_revenge": revenge }))
}

pub fn champ_duel_forfeit(api: &Api) -> Result<Value> {
    api.rpc("api_champ_duel_forfeit", json!({}))
}
