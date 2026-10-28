use serde_json::{json, Value};

use crate::net::Api;
use crate::support::error::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Missing {
    None,
    Needs { gold: u64, elixir: u64, dark: u64 },
    Unknown,
}

pub fn place(api: &Api, kind: &str, q: i32, r: i32) -> Result<Value> {
    api.rpc("api_place", json!({ "p_kind": kind, "p_q": q, "p_r": r }))
}

pub fn upgrade(api: &Api, bid: i64) -> Result<Value> {
    api.rpc("api_upgrade", json!({ "bid": bid }))
}

pub fn missing_cost(api: &Api, kind: &str, bid: i64) -> Result<Missing> {
    let reply = api.rpc(
        "api_missing_cost",
        json!({ "p_kind": kind, "p_building": bid }),
    )?;
    Ok(read_missing(&reply))
}

pub fn finish_now(api: &Api, kind: &str, reference: i64) -> Result<Value> {
    api.rpc(
        "api_finish_now",
        json!({ "p_kind": kind, "p_ref": reference }),
    )
}

pub fn finish_cost(api: &Api) -> Result<Value> {
    api.rpc("api_finish_cost", json!({}))
}

pub fn read_missing(reply: &Value) -> Missing {
    let source = reply.get("missing").unwrap_or(reply);
    if !source.is_object() {
        return Missing::Unknown;
    }

    let mut seen = false;
    let mut gold = 0u64;
    let mut elixir = 0u64;
    let mut dark = 0u64;

    for (key, value) in source.as_object().map(|m| m.iter()).into_iter().flatten() {
        let amount = match value.as_f64() {
            Some(number) => number.max(0.0) as u64,
            None => continue,
        };
        match key.as_str() {
            "gold" | "p_gold" => {
                gold = amount;
                seen = true;
            }
            "elixir" | "p_elixir" => {
                elixir = amount;
                seen = true;
            }
            "dark" | "p_dark" => {
                dark = amount;
                seen = true;
            }
            _ => {}
        }
    }

    if !seen {
        return Missing::Unknown;
    }

    if gold == 0 && elixir == 0 && dark == 0 {
        Missing::None
    } else {
        Missing::Needs { gold, elixir, dark }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_reply_means_the_upgrade_is_affordable() {
        let reply = json!({"gold": 0, "elixir": 0, "dark": 0});
        assert_eq!(read_missing(&reply), Missing::None);
    }

    #[test]
    fn a_wrapped_reply_is_read_too() {
        let reply = json!({"missing": {"gold": 120, "elixir": 0}});
        assert_eq!(
            read_missing(&reply),
            Missing::Needs {
                gold: 120,
                elixir: 0,
                dark: 0
            }
        );
    }

    #[test]
    fn a_shape_we_do_not_know_is_unknown() {
        assert_eq!(read_missing(&json!({"cost": 5})), Missing::Unknown);
        assert_eq!(read_missing(&json!(null)), Missing::Unknown);
    }
}
