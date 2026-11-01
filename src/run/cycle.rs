use std::sync::Mutex;

use serde_json::{json, Value};

use crate::entry::Gate;
use crate::logika::{Diam, Hasil, Kondisi, Otak, Sumber};
use crate::net::pace;
use crate::net::Api;
use crate::raid::siege::{fingerprint, Sim};
use crate::run::plan::{lakukan, Konteks};
use crate::state::{Katalog, Snapshot};
use crate::support::cfg::Config;
use crate::support::consts::app::{
    CYCLE_JITTER_SECS, CYCLE_STEPS_MAX, GAME_BUILD, RECOVER_TRIES, SIM_SHA256, TRANSIENT_PAUSE_SECS,
};
use crate::support::consts::net::{SIM_URL, VERSION_URL};
use crate::support::error::{Error, Result};
use crate::support::log;
use crate::{army, champ, chest, quests, raid, store, treasury, village, war};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Once,
    Loop,
}

pub fn jalankan(api: &Api, cfg: &Config, mode: Mode) -> Result<()> {
    let mut sim: Option<Sim> = None;
    let mut nomor: u64 = 0;
    let mut katalog: Option<Katalog> = None;
    let mut katalog_versi = String::new();
    let mut katalog_gagal = false;
    let mut gagal: u32 = 0;

    match pintu_terbuka(api) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        Err(err) => {
            if !pulih(api, &err) {
                return Err(err);
            }
        }
    }

    loop {
        nomor += 1;

        let state = match tarik_state(api) {
            Ok(state) => {
                gagal = 0;
                state
            }
            Err(err) => {
                if !pulih(api, &err) {
                    return Err(err);
                }
                gagal += 1;
                if mode == Mode::Once && gagal > RECOVER_TRIES {
                    return Err(err);
                }
                continue;
            }
        };
        let versi = versi_hidup(api);

        if katalog.is_none() || versi != katalog_versi {
            match tarik_katalog(api) {
                Ok(fresh) => {
                    katalog = Some(fresh);
                    katalog_versi = versi.clone();
                    katalog_gagal = false;
                }
                Err(Error::Denied) => {
                    log::lr("The cycle stopped because the server answered 401");
                    return Err(Error::Denied);
                }
                Err(other) => {
                    if !katalog_gagal {
                        log::ly(&format!(
                            "The game catalog could not be read because {}",
                            log::explain(&other)
                        ));
                        katalog_gagal = true;
                    }
                }
            }
        }

        let path_fix_from = katalog.as_ref().and_then(|k| k.path_fix_from());
        let air_fix_from = katalog.as_ref().and_then(|k| k.air_fix_from());
        let mut kondisi = Kondisi::new(state, &versi, GAME_BUILD);
        if let Some(katalog) = &katalog {
            kondisi = kondisi.dengan_katalog(katalog.clone());
        }
        let mut otak = Otak::baru(kondisi);

        if !otak.kondisi().versi_cocok() {
            log::ly(&format!(
                "The game moved to build {} so raids are on hold",
                versi
            ));
        }

        isi_sumber_publik(api, &mut otak);

        let mut langkah: usize = 0;
        let mut berhenti = false;

        while langkah < CYCLE_STEPS_MAX {
            let next = match otak.berikut() {
                Some(next) => next,
                None => break,
            };

            let sebelum = otak.kondisi().state.clone();
            let hasil = lakukan(
                &mut Konteks {
                    api,
                    sim: &mut sim,
                    path_fix_from,
                    air_fix_from,
                },
                &next.aksi,
                &sebelum,
            );

            langkah += 1;
            otak.catat(&next.aksi, hasil, &next.alasan);

            match hasil {
                Hasil::Mati => {
                    berhenti = true;
                    break;
                }
                Hasil::Selesai => match tarik_state(api) {
                    Ok(fresh) => {
                        otak.segarkan(fresh);
                        if let Some(sumber) = next.aksi.sumber() {
                            otak.buang(sumber);
                            isi_sumber_publik(api, &mut otak);
                        }
                    }
                    Err(Error::Denied) => {
                        log::lr("The cycle stopped because the server answered 401");
                        berhenti = true;
                        break;
                    }
                    Err(other) => {
                        log::ly(&format!(
                            "The state could not be read again because {}",
                            log::explain(&other)
                        ));
                    }
                },
                _ => {}
            }
        }

        if langkah >= CYCLE_STEPS_MAX {
            log::ly("The cycle reached its step limit so it stops here");
        }

        if berhenti {
            if !pulih(api, &Error::Denied) {
                return Err(Error::Denied);
            }
            gagal += 1;
            if mode == Mode::Once && gagal > RECOVER_TRIES {
                return Err(Error::Denied);
            }
            continue;
        }

        log::lg(&format!("Automation cycle number {} is complete", nomor));

        if mode == Mode::Once {
            return Ok(());
        }

        match otak.diam(cfg.settings.sleep_seconds) {
            Diam::Tunggu { secs, alasan } => {
                log::ly(&format!("The bot waits because {}", alasan));
                log::countdown(
                    secs + pace::jitter_secs(CYCLE_JITTER_SECS),
                    "Next cycle starts in",
                );
            }
            Diam::Tidur { secs, alasan } => {
                log::ly(&format!("The bot sleeps because {}", alasan));
                log::countdown(
                    secs + pace::jitter_secs(CYCLE_JITTER_SECS),
                    "Next cycle starts in",
                );
            }
        }
    }
}

pub fn fatal(err: &Error) -> bool {
    matches!(err, Error::Banned | Error::Cfg(_))
}

pub fn pulih(api: &Api, err: &Error) -> bool {
    match err {
        Error::Banned => {
            log::lr("The account is banned so the run ends here");
            false
        }
        Error::Cfg(_) => {
            log::lr(&format!(
                "The run cannot start because {}",
                log::explain(err)
            ));
            false
        }
        Error::Denied => {
            log::ly("The session is no longer accepted so the bot signs in again");
            for _ in 0..RECOVER_TRIES {
                match crate::run::session::masuk(api, true) {
                    Ok(_) => return true,
                    Err(Error::Cfg(_)) => {
                        log::lr("The sign in needs the private key so fill .env with PRIVATEKEY=0x followed by 64 hex characters");
                        return false;
                    }
                    Err(Error::Banned) => return false,
                    Err(again) => {
                        log::ly(&format!(
                            "The sign in failed because {} so it tries again",
                            log::explain(&again)
                        ));
                        std::thread::sleep(std::time::Duration::from_secs(TRANSIENT_PAUSE_SECS));
                    }
                }
            }
            log::ly("The run stays alive and signs in again on the next cycle");
            true
        }
        other => {
            log::ly(&format!(
                "The request failed because {} so the next cycle tries again",
                log::explain(other)
            ));
            std::thread::sleep(std::time::Duration::from_secs(TRANSIENT_PAUSE_SECS));
            true
        }
    }
}

pub fn pintu_terbuka(api: &Api) -> Result<bool> {
    let reply = crate::entry::entry::entry(api)?;
    let gate = Gate::read(&reply);

    if gate.banned {
        log::lr("The account is banned so the run ends here");
        return Ok(false);
    }

    if gate.open() {
        return Ok(true);
    }

    log::lr("The village entry is not paid yet so the bot stops here");
    match gate.name {
        Some(name) => log::ly(&format!(
            "The entry costs {:.0} usd and {} already holds the name",
            gate.usd, name
        )),
        None => log::ly("The entry needs a name first so run ./run.sh name <your name>"),
    }
    log::ly("Run ./run.sh entry to see the payment then ./run.sh paid <hash>");

    Ok(false)
}

pub fn tarik_katalog(api: &Api) -> Result<Katalog> {
    let raw = api.rpc("api_catalog", json!({}))?;
    Katalog::parse(&raw)
}

pub fn tarik_state(api: &Api) -> Result<Snapshot> {
    let raw = api.rpc("api_state", json!({}))?;
    Snapshot::parse(raw)
}

pub fn versi_hidup(api: &Api) -> String {
    let live = match api.fetch(VERSION_URL) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).trim().to_string(),
        Err(_) => return GAME_BUILD.to_string(),
    };

    if live == GAME_BUILD {
        return live;
    }
    if live.len() < 6 || live.len() > 40 || !live.chars().all(|c| c.is_ascii_hexdigit()) {
        return GAME_BUILD.to_string();
    }
    if sim_cocok(api, &live) {
        return GAME_BUILD.to_string();
    }

    live
}

fn sim_cocok(api: &Api, live: &str) -> bool {
    static CACHE: Mutex<Option<(String, bool)>> = Mutex::new(None);

    let mut cache = match CACHE.lock() {
        Ok(cache) => cache,
        Err(poison) => poison.into_inner(),
    };
    if let Some((stamp, ok)) = cache.as_ref() {
        if stamp == live {
            return *ok;
        }
    }

    let url = format!("{}{}", SIM_URL, live);
    let ok = match api.fetch(&url) {
        Ok(bytes) => fingerprint(&bytes) == SIM_SHA256,
        Err(_) => true,
    };
    *cache = Some((live.to_string(), ok));

    ok
}

pub fn isi_sumber_publik(api: &Api, otak: &mut Otak) {
    let butuh = otak.butuh();

    for sumber in butuh {
        let hasil = match sumber {
            Sumber::Quests => quests::quest::quests(api),
            Sumber::Daily => quests::daily::daily(api),
            Sumber::Pass => quests::pass::pass(api),
            Sumber::Events => quests::event::events(api),
            Sumber::Season => quests::event::season(api),
            Sumber::Campaign => quests::event::campaign(api),
            Sumber::Chest => chest::chest::chest(api),
            Sumber::War => war::war::war(api),
            Sumber::Cwl => war::cwl::cwl(api),
            Sumber::BattleLog => raid::battle_log(api),
            Sumber::Store => store::store::store(api),
            Sumber::Arena => champ::champ_arena(api),
            Sumber::Duel => champ::champ_duel(api),
            Sumber::Treasury => treasury::treasury(api),
            Sumber::Army => army::army_status(api),
            Sumber::Forge => army::forge::forge_collect(api),
            Sumber::Citadel => village::citadel::citadel(api),
            Sumber::State => Ok(Value::Null),
        };

        match hasil {
            Ok(value) => otak.simpan(sumber, value),
            Err(Error::Denied) => {
                log::lr("The cycle stopped because the server answered 401");
                return;
            }
            Err(other) => {
                log::ly(&format!(
                    "The {} could not be read because {}",
                    sumber.label(),
                    log::explain(&other)
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_banned_account_is_fatal() {
        assert!(fatal(&Error::Banned));
    }

    #[test]
    fn a_missing_key_is_fatal() {
        assert!(fatal(&Error::Cfg("no key".to_string())));
    }

    #[test]
    fn a_lost_session_is_not_fatal() {
        assert!(!fatal(&Error::Denied));
        assert!(!fatal(&Error::Http("the request failed".to_string())));
        assert!(!fatal(&Error::Api {
            status: 503,
            message: "the server is busy".to_string()
        }));
    }
}
