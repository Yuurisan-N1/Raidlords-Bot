use std::path::Path;

use crate::support::cfg::schema::{Config, DEFAULTS};
use crate::support::consts::app::{CONFIG_FILE, MIN_SLEEP_SECS};
use crate::support::error::{Error, Result};

pub fn load() -> Result<Config> {
    if !Path::new(CONFIG_FILE).exists() {
        std::fs::write(CONFIG_FILE, DEFAULTS)?;
        return Ok(Config::default());
    }

    let text = std::fs::read_to_string(CONFIG_FILE)?;
    if text.trim().is_empty() {
        std::fs::write(CONFIG_FILE, DEFAULTS)?;
        return Ok(Config::default());
    }

    let mut config: Config = serde_json::from_str(&text)
        .map_err(|e| Error::Cfg(format!("config.json could not be read: {}", e)))?;

    if config.settings.sleep_seconds < MIN_SLEEP_SECS {
        config.settings.sleep_seconds = MIN_SLEEP_SECS;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::cfg::schema::DEFAULTS_LEN;

    #[test]
    fn the_defaults_are_fifty_bytes() {
        assert_eq!(DEFAULTS.len(), DEFAULTS_LEN);
    }

    #[test]
    fn the_defaults_parse() {
        let config: Config = serde_json::from_str(DEFAULTS).unwrap();
        assert_eq!(config.settings.sleep_seconds, 3600);
    }
}
