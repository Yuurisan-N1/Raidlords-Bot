use serde::{Deserialize, Serialize};

use crate::support::consts::app::SESSION_FILE;
use crate::support::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached {
    pub wallet: String,
    pub token: String,
    pub exp: u64,
}

pub fn read() -> Option<Cached> {
    let text = std::fs::read_to_string(SESSION_FILE).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn write(cached: &Cached) -> Result<()> {
    let text = serde_json::to_string_pretty(cached)?;
    std::fs::write(SESSION_FILE, text)?;
    Ok(())
}

pub fn clear() {
    let _ = std::fs::remove_file(SESSION_FILE);
}
