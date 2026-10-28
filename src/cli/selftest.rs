use crate::support::cfg::schema::DEFAULTS;
use crate::support::consts::app::SIM_FILE;
use crate::support::log;

pub fn jalankan() -> crate::support::error::Result<()> {
    let mut gagal = 0;

    if DEFAULTS.len() == 50 {
        log::lg("The default config is fifty bytes long");
    } else {
        log::lr("The default config is not fifty bytes long");
        gagal += 1;
    }

    match std::fs::read_to_string(SIM_FILE) {
        Ok(source) => {
            if source.contains("makeBase") && source.contains("battle") {
                log::lg("The battle engine file is in place");
            } else {
                log::lr("The battle engine file is not the game engine");
                gagal += 1;
            }
        }
        Err(_) => {
            log::lr("The battle engine file is missing");
            gagal += 1;
        }
    }

    match crate::support::private_key() {
        Some(_) => log::lg("The private key is in .env"),
        None => log::ly("The private key is not in .env yet"),
    }

    let proxies = crate::net::proxy::load();
    if proxies.is_empty() {
        log::ly("proxy.txt has no lines so the bot goes out on its own address");
    } else {
        log::ly(&format!("proxy.txt holds {} lines", proxies.len()));
    }

    let state = crate::state::Snapshot::parse(serde_json::json!({
        "player": {"id": 1, "gold": 10, "attacks": 1, "free_builders": 1},
        "buildings": [{"q": 0, "r": 0, "id": 1, "kind": "gold_mine", "level": 1, "stock": 5}],
        "limits": {"gold_mine": {"have": 0, "max": 1, "max_level": 3}}
    }))?;

    let kondisi = crate::logika::Kondisi::new(state, "test", "test");
    let otak = crate::logika::Otak::baru(kondisi);
    let plan = otak.rencana();

    if plan.first().map(|l| l.aksi.label()) == Some("collect") {
        log::lg("The brain picks the collectors first");
    } else {
        log::lr("The brain did not pick the collectors first");
        gagal += 1;
    }

    if gagal == 0 {
        log::lg("The self test passed");
        Ok(())
    } else {
        log::lr(&format!("The self test failed on {} checks", gagal));
        Err(crate::support::error::Error::State(
            "the self test did not pass".to_string(),
        ))
    }
}
