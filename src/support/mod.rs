pub mod banner;
pub mod cfg;
pub mod codec;
pub mod consts;
pub mod error;
pub mod log;

use std::collections::HashMap;

pub fn read_env() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let text = std::fs::read_to_string(crate::support::consts::app::ENV_FILE).unwrap_or_default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

pub fn private_key() -> Option<String> {
    let map = read_env();
    let raw = map.get("PRIVATEKEY").or_else(|| map.get("privatekey"))?;
    let body = raw.trim().trim_matches('"');
    let body = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
        .unwrap_or(body);
    if body.len() == 64 && body.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(body.to_ascii_lowercase())
    } else {
        None
    }
}
