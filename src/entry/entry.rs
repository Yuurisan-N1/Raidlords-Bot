use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct Gate {
    pub paid: bool,
    pub required: bool,
    pub usd: f64,
    pub banned: bool,
    pub name: Option<String>,
}

impl Gate {
    pub fn read(reply: &Value) -> Gate {
        Gate {
            paid: reply.get("paid").and_then(|v| v.as_bool()).unwrap_or(false),
            required: reply
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            usd: reply.get("usd").and_then(|v| v.as_f64()).unwrap_or(0.0),
            banned: reply
                .get("banned")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            name: reply
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }

    pub fn open(&self) -> bool {
        !self.required || self.paid
    }
}

pub fn entry(api: &Api) -> Result<Value> {
    api.rpc("api_entry", json!({}))
}

pub fn quote(api: &Api) -> Result<Value> {
    api.rpc("api_entry_quote", json!({}))
}

pub fn submit(api: &Api, quote: &Value, tx: &str) -> Result<Value> {
    api.rpc("api_entry_submit", json!({ "p_quote": quote, "p_tx": tx }))
}

pub fn set_name(api: &Api, name: &str) -> Result<Value> {
    api.rpc("api_set_name", json!({ "p_name": name }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unpaid_entry_is_closed() {
        let gate = Gate::read(&json!({"paid": false, "required": true, "usd": 1}));
        assert!(!gate.open());
        assert_eq!(gate.usd, 1.0);
    }

    #[test]
    fn a_paid_entry_is_open() {
        let gate = Gate::read(&json!({"paid": true, "required": true}));
        assert!(gate.open());
    }

    #[test]
    fn a_free_village_needs_no_entry() {
        let gate = Gate::read(&json!({"paid": false, "required": false}));
        assert!(gate.open());
    }
}
