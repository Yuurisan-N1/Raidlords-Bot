use serde_json::{json, Value};

use crate::entry::Gate;
use crate::logika::{Diam, Hasil, Kondisi, Otak, Sumber};
use crate::net::pace;
use crate::net::Api;
use crate::raid::siege::Sim;
use crate::run::plan::{lakukan, Konteks};
use crate::state::{Katalog, Snapshot};
use crate::support::cfg::Config;
use crate::support::consts::app::{CYCLE_JITTER_SECS, CYCLE_STEPS_MAX, GAME_BUILD};
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

    if !pintu_terbuka(api)? {
        return Ok(());
    }

    loop {
        nomor += 1;

        let state = tarik_state(api)?;
        let versi = version_of(&state);

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
            let hasil = lakukan(&mut Konteks { api, sim: &mut sim }, &next.aksi, &sebelum);

            langkah += 1;
            otak.catat(&next.aksi, hasil, &next.alasan);

            match hasil {
                Hasil::Mati => {
                    berhenti = true;
                    break;
                }
                Hasil::Selesai => match tarik_state(api) {
                    Ok(fresh) => otak.segarkan(fresh),
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
            return Err(Error::Denied);
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

pub fn version_of(state: &Snapshot) -> String {
    state
        .raw()
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| GAME_BUILD.to_string())
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
