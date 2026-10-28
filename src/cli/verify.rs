use crate::support::cfg::schema::DEFAULTS;
use crate::support::consts::app::{CONFIG_FILE, SIM_FILE};
use crate::support::error::Result;
use crate::support::log;

pub fn jalankan() -> Result<()> {
    let mut gagal = 0;

    match std::fs::read(CONFIG_FILE) {
        Ok(bytes) => {
            if bytes.len() == 50 {
                log::lg(&format!("{} holds {} bytes", CONFIG_FILE, bytes.len()));
            } else if bytes.len() == DEFAULTS.len() {
                log::lg(&format!("{} holds the default settings", CONFIG_FILE));
            } else {
                log::ly(&format!(
                    "{} holds {} bytes which is not the default shape",
                    CONFIG_FILE,
                    bytes.len()
                ));
            }
        }
        Err(_) => {
            log::lr(&format!("{} is missing", CONFIG_FILE));
            gagal += 1;
        }
    }

    match std::fs::metadata(SIM_FILE) {
        Ok(meta) => log::lg(&format!("{} holds {} bytes", SIM_FILE, meta.len())),
        Err(_) => {
            log::lr(&format!("{} is missing", SIM_FILE));
            gagal += 1;
        }
    }

    match crate::support::private_key() {
        Some(_) => log::lg(".env holds a private key"),
        None => {
            log::lr(".env holds no usable private key");
            gagal += 1;
        }
    }

    let proxies = crate::net::proxy::load();
    if proxies.is_empty() {
        log::ly("proxy.txt is empty so no proxy line is used");
    } else {
        for spec in &proxies {
            log::ly(&format!("Using proxy {}", spec.masked()));
        }
    }

    if gagal == 0 {
        log::lg("Every file the bot needs is in place");
        Ok(())
    } else {
        log::lr(&format!("{} files still need attention", gagal));
        Err(crate::support::error::Error::Cfg(
            "the local files are not ready".to_string(),
        ))
    }
}
