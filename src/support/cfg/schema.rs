use serde::{Deserialize, Serialize};

use crate::support::consts::app::DEFAULT_SLEEP_SECS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub sleep_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                sleep_seconds: DEFAULT_SLEEP_SECS,
            },
        }
    }
}

pub const DEFAULTS: &str = "{\n  \"settings\": {\n    \"sleep_seconds\": 3600\n  }\n}\n";

pub const DEFAULTS_LEN: usize = 50;
