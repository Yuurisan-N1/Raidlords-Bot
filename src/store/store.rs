use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

pub fn store(api: &Api) -> Result<Value> {
    api.rpc("api_store", json!({}))
}

pub fn store_buy(api: &Api, item: &str) -> Result<Value> {
    api.rpc("api_store_buy", json!({ "p_item": item }))
}

pub fn my_items(api: &Api) -> Result<Value> {
    api.rpc("api_my_items", json!({}))
}

pub fn use_item(api: &Api, item: &str, reference: &str) -> Result<Value> {
    api.rpc(
        "api_use_item",
        json!({ "p_item": item, "p_ref": reference }),
    )
}

pub fn affordable(reply: &Value, token_balance: u64) -> Option<String> {
    let items = reply.get("items").and_then(|v| v.as_array())?;
    let mut best: Option<(u64, String)> = None;
    for item in items {
        let price = item
            .get("price_token")
            .or_else(|| item.get("price"))
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::MAX) as u64;
        if price == 0 || price > token_balance {
            continue;
        }
        let id = match item.get("id").or_else(|| item.get("key")) {
            Some(value) => value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| value.to_string()),
            None => continue,
        };
        match &best {
            Some((seen, _)) if *seen <= price => {}
            _ => best = Some((price, id)),
        }
    }
    best.map(|(_, id)| id)
}
