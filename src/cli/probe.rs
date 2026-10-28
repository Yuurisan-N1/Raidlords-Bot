use crate::logika::{Diam, Kondisi, Otak, Sumber};
use crate::net::Api;
use crate::run::cycle::{isi_sumber_publik, tarik_state, version_of};
use crate::support::consts::app::{CYCLE_STEPS_MAX, DEFAULT_SLEEP_SECS, GAME_BUILD};
use crate::support::error::Result;
use crate::support::log;

pub fn jalankan(api: &Api) -> Result<()> {
    let state = tarik_state(api)?;
    let versi = version_of(&state);
    let mut otak = Otak::baru(Kondisi::new(state, &versi, GAME_BUILD));

    isi_sumber_publik(api, &mut otak);

    let plan = otak.rencana();
    let jumlah = plan.len().min(CYCLE_STEPS_MAX);

    log::ly(&format!("The plan holds {} steps for this cycle", jumlah));

    if let Some(first) = plan.first() {
        log::ly(&format!(
            "The first step is {} because {}",
            first.aksi.label(),
            first.alasan
        ));
    } else {
        log::ly("The plan is empty because every task is already done");
    }

    if let Some(last) = plan.last() {
        if plan.len() > 1 {
            log::ly(&format!(
                "The last step is {} because {}",
                last.aksi.label(),
                last.alasan
            ));
        }
    }

    let menunggu = otak.butuh();
    if !menunggu.is_empty() {
        let labels: Vec<&str> = menunggu.iter().map(|s| s.label()).collect();
        log::ly(&format!(
            "These boards are not read yet: {}",
            labels.join(", ")
        ));
    }

    match otak.diam(DEFAULT_SLEEP_SECS) {
        Diam::Tidur { secs, alasan } => {
            log::ly(&format!("The bot would sleep because {}", alasan));
            log::ly(&format!(
                "The next cycle would start in {}",
                log::stamp(secs)
            ));
        }
        Diam::Tunggu { secs, alasan } => {
            log::ly(&format!("The bot would wait because {}", alasan));
            log::ly(&format!(
                "The next cycle would start in {}",
                log::stamp(secs)
            ));
        }
    }

    if let Some(entry) = otak.kondisi().kutip(Sumber::Treasury) {
        if let Some(room) = entry.get("room_usd").and_then(|v| v.as_f64()) {
            log::ly(&format!("The treasury has room for {:.2} usd", room));
        }
    }

    Ok(())
}
