use serde_json::{json, Value};

use crate::logika::{Aksi, Hasil};
use crate::net::Api;
use crate::raid::siege::{Battle, Outcome, Sim};
use crate::state::Snapshot;
use crate::support::consts::app::{BATTLE_RETRY_MS, FINISH_TRIES, RAID_STARS_FLOOR, SCAN_TARGETS};
use crate::support::error::{Error, Result};
use crate::support::log;
use crate::{army, champ, chest, quests, raid, store, treasury, village, war};

pub struct Konteks<'a> {
    pub api: &'a Api,
    pub sim: &'a mut Option<Sim>,
    pub path_fix_from: Option<u64>,
    pub air_fix_from: Option<u64>,
}

pub fn lakukan(ctx: &mut Konteks, aksi: &Aksi, sebelum: &Snapshot) -> Hasil {
    match kerja(ctx, aksi, sebelum) {
        Ok(Hasil::Selesai) => Hasil::Selesai,
        Ok(other) => other,
        Err(err) => {
            match &err {
                Error::Denied => {
                    log::lr("The cycle stopped because the server answered 401");
                    return Hasil::Mati;
                }
                Error::Banned => {
                    log::lr("The account is banned so the run ends here");
                    return Hasil::Mati;
                }
                other => {
                    let text = log::explain(other);
                    if masuk_pintu(&text) {
                        log::lr("The village entry is not paid yet so the bot stops here");
                        return Hasil::Mati;
                    }
                    log::lr(&format!(
                        "The {} step failed because {}",
                        aksi.label(),
                        text
                    ));
                }
            }
            Hasil::Ditolak
        }
    }
}

fn kerja(ctx: &mut Konteks, aksi: &Aksi, sebelum: &Snapshot) -> Result<Hasil> {
    let api = ctx.api;

    match aksi {
        Aksi::Collect => {
            village::collect(api)?;
            Ok(Hasil::Selesai)
        }
        Aksi::Place { kind, q, r } => {
            village::place(api, kind, *q, *r)?;
            log::lg(&format!("A {} was placed on the map", kind));
            Ok(Hasil::Selesai)
        }
        Aksi::Upgrade { bid, kind, level } => {
            village::upgrade(api, *bid)?;
            log::lg(&format!(
                "A builder started the {} upgrade to level {}",
                kind, level
            ));
            Ok(Hasil::Selesai)
        }
        Aksi::UpgradeWalls { ids, res } => {
            village::upgrade_walls(api, ids, res)?;
            log::lg(&format!("{} wall pieces went up together", ids.len()));
            Ok(Hasil::Selesai)
        }
        Aksi::FinishNow { kind, reference } => {
            village::finish_now(api, kind, *reference)?;
            log::lg(&format!("The {} job was finished with gems", kind));
            Ok(Hasil::Selesai)
        }
        Aksi::ClearObstacle { id } => {
            village::clear_obstacle(api, *id)?;
            log::lg("An obstacle was cleared from the map");
            Ok(Hasil::Selesai)
        }
        Aksi::TrainArmy { kind, count } => {
            let army = army_shape(sebelum, kind, *count);
            if army.is_null() {
                return Ok(Hasil::Kosong);
            }
            army::set_army(api, &army)?;
            log::lg("Army training started");
            Ok(Hasil::Selesai)
        }
        Aksi::Retrain => {
            army::train_army(api)?;
            log::lg("The fallen troops are training again");
            Ok(Hasil::Selesai)
        }
        Aksi::Research { troop } => {
            army::research::research(api, troop)?;
            log::lg(&format!("The lab started on {}", troop));
            Ok(Hasil::Selesai)
        }
        Aksi::Raid => serangan(ctx, sebelum),
        Aksi::Revenge { battle } => {
            raid::revenge::revenge(api, *battle)?;
            log::lg(&format!("Battle {} was answered", battle));
            Ok(Hasil::Selesai)
        }
        Aksi::WarAttack { pos } => {
            war::war_attack(api, *pos)?;
            log::lg(&format!("The war attack on position {} was sent", pos));
            Ok(Hasil::Selesai)
        }
        Aksi::CwlAttack { pos } => {
            war::cwl_attack(api, *pos)?;
            log::lg(&format!("The league attack on position {} was sent", pos));
            Ok(Hasil::Selesai)
        }
        Aksi::Daily => {
            quests::daily::daily_claim(api)?;
            log::lg("The daily reward was claimed");
            Ok(Hasil::Selesai)
        }
        Aksi::Quest { kind, id } => {
            quests::quest::quest_claim(api, kind, id)?;
            log::lg(&format!("The {} quest {} was claimed", kind, id));
            Ok(Hasil::Selesai)
        }
        Aksi::PassTier { tier, track } => {
            quests::pass::pass_claim(api, *tier, track)?;
            log::lg(&format!("The {} pass tier {} was claimed", track, tier));
            Ok(Hasil::Selesai)
        }
        Aksi::EventClaim { id } => {
            quests::event::event_claim(api, id)?;
            log::lg(&format!("The event reward {} was claimed", id));
            Ok(Hasil::Selesai)
        }
        Aksi::SeasonClaim { id } => {
            quests::event::event_claim(api, id)?;
            log::lg(&format!("The season reward {} was claimed", id));
            Ok(Hasil::Selesai)
        }
        Aksi::TrialAttack { level } => {
            quests::event::trial_attack(api, *level)?;
            log::lg(&format!("Trial {} was attacked", level));
            Ok(Hasil::Selesai)
        }
        Aksi::CampaignAttack { level } => kampanye(ctx, sebelum, *level),
        Aksi::TourneyJoin => {
            quests::event::tourney_join(api)?;
            log::lg("The tournament was joined");
            Ok(Hasil::Selesai)
        }
        Aksi::WaitlistClaim => {
            quests::event::waitlist_claim(api)?;
            log::lg("The waitlist reward was claimed");
            Ok(Hasil::Selesai)
        }
        Aksi::VetClaim { day } => {
            quests::event::vet_claim(api, *day)?;
            log::lg(&format!("Veteran day {} was claimed", day));
            Ok(Hasil::Selesai)
        }
        Aksi::WarPassClaim { day } => {
            quests::event::war_pass_claim(api, *day)?;
            log::lg(&format!("War pass day {} was claimed", day));
            Ok(Hasil::Selesai)
        }
        Aksi::BurnGoal => {
            quests::event::burn_goal_claim(api)?;
            log::lg("The burn goal reward was claimed");
            Ok(Hasil::Selesai)
        }
        Aksi::RewardTake { gold } => {
            quests::event::reward_take(api, *gold)?;
            log::lg(&format!("{} gold was taken back", gold));
            Ok(Hasil::Selesai)
        }
        Aksi::ChestUnlock { id } => {
            chest::chest_unlock(api, id)?;
            log::lg(&format!("Chest {} was opened", id));
            Ok(Hasil::Selesai)
        }
        Aksi::ChestLock { days } => {
            chest::chest_lock(api, sebelum.tokens(), *days)?;
            log::lg(&format!("A chest was locked for {} days", days));
            Ok(Hasil::Selesai)
        }
        Aksi::SellGold { gold } => {
            treasury::sell_gold(api, *gold, 0.0)?;
            log::lg(&format!("{} gold went to the treasury", gold));
            Ok(Hasil::Selesai)
        }
        Aksi::Withdraw { cents } => {
            let amount = *cents as f64 / 100.0;
            treasury::withdraw(api, amount)?;
            log::lg(&format!(
                "{:.2} usd was withdrawn from the treasury",
                amount
            ));
            Ok(Hasil::Selesai)
        }
        Aksi::StoreBuy { item } => {
            store::store_buy(api, item)?;
            log::lg(&format!("{} was bought from the store", item));
            Ok(Hasil::Selesai)
        }
        Aksi::ArenaFight => {
            champ::champ_arena_start(api)?;
            log::lg("An arena fight was started");
            Ok(Hasil::Selesai)
        }
        Aksi::DuelFight => {
            champ::champ_duel_start(api, false)?;
            log::lg("A champion duel was started");
            Ok(Hasil::Selesai)
        }
        Aksi::HeroUpgrade { hero } => {
            army::hero::hero_upgrade(api, hero)?;
            log::lg(&format!("{} went up a level", hero));
            Ok(Hasil::Selesai)
        }
        Aksi::ForgeCraft { res } => {
            army::forge::forge_craft(api, res)?;
            log::lg(&format!("The forge started on {}", res));
            Ok(Hasil::Selesai)
        }
        Aksi::Citadel { res, amount } => {
            village::citadel::contribute(api, res, *amount)?;
            log::lg(&format!("{} {} went to the citadel", amount, res));
            Ok(Hasil::Selesai)
        }
    }
}

fn kampanye(ctx: &mut Konteks, sebelum: &Snapshot, level: i64) -> Result<Hasil> {
    for percobaan in 0..2 {
        let (hasil, sebab_tolak) = satu_kampanye(ctx, sebelum, level)?;
        let tutup = sebab_tolak
            .as_deref()
            .map(|sebab| sebab.to_ascii_lowercase().contains("already closed"))
            .unwrap_or(false);
        if tutup && percobaan == 0 {
            log::ly(&format!(
                "Stage {} was closed before the log arrived so a fresh battle is opened",
                level
            ));
            continue;
        }
        return Ok(hasil);
    }
    Ok(Hasil::Ditolak)
}

fn satu_kampanye(
    ctx: &mut Konteks,
    sebelum: &Snapshot,
    level: i64,
) -> Result<(Hasil, Option<String>)> {
    let api = ctx.api;
    let Some(reply) = tempur(api, || quests::event::campaign_attack(api, level))? else {
        return Ok((Hasil::Kosong, None));
    };

    if ctx.sim.is_none() {
        *ctx.sim = Some(Sim::load()?);
    }

    let mut battle = match Battle::from_reply(&reply, ctx.path_fix_from, ctx.air_fix_from) {
        Some(battle) => battle,
        None => {
            log::ly(&format!(
                "Stage {} came back without a map so it waits",
                level
            ));
            return Ok((Hasil::Ditolak, None));
        }
    };
    if battle.empty_army() {
        battle.army = full_army(sebelum);
    }
    if battle.empty_army() {
        log::ly(&format!(
            "Stage {} waits because there are no troops to send",
            level
        ));
        return Ok((Hasil::Ditolak, None));
    }

    let id = battle.id.clone();
    if id.is_null() {
        log::ly(&format!(
            "Stage {} came back without a battle id so it waits",
            level
        ));
        return Ok((Hasil::Ditolak, None));
    }

    let sim = ctx.sim.as_mut().expect("the simulation was just loaded");
    let kunci_waktu = std::time::Instant::now();
    let outcome = sim.run(&battle)?;
    if std::env::var_os("RL_TIMING").is_some() {
        eprintln!(
            "[timing] stage {} sim took {} ms",
            level,
            kunci_waktu.elapsed().as_millis()
        );
    }

    if !outcome.verified {
        log::lr(&format!(
            "The battle log for stage {} does not replay so it is skipped",
            level
        ));
        return Ok((Hasil::Ditolak, None));
    }

    let done = match kirim(api, &id, &outcome) {
        Ok(done) => done,
        Err(Error::Api { status, message }) => {
            log::ly(&format!(
                "The battle check answered {} so stage {} is skipped because {}",
                status, level, message
            ));
            return Ok((Hasil::Ditolak, Some(message)));
        }
        Err(other) => return Err(other),
    };

    let (stars, gold) = match spoils(&done) {
        Some(spoils) => spoils,
        None => {
            let alasan = sebab(&done);
            log::ly(&format!(
                "The battle check turned stage {} down because {}",
                level, alasan
            ));
            return Ok((Hasil::Ditolak, Some(alasan)));
        }
    };
    log::lg(&format!(
        "Stage {} finished with {} {} and {} gold",
        level,
        stars,
        log::unit_word(stars, "star", "stars"),
        gold
    ));
    Ok((Hasil::Selesai, None))
}

fn masih_latih(message: &str) -> bool {
    message.contains("still training") || message.contains("train your army")
}

fn ada_siap(api: &Api) -> Result<bool> {
    let status = army::train::army_status(api)?;
    let ready = status
        .get("ready")
        .and_then(|v| v.as_object())
        .map(|map| map.values().any(|v| v.as_i64().unwrap_or(0) > 0))
        .unwrap_or(false);
    Ok(ready)
}

fn tempur<F>(api: &Api, call: F) -> Result<Option<Value>>
where
    F: Fn() -> Result<Value>,
{
    for attempt in 1..=2 {
        match call() {
            Ok(reply) => return Ok(Some(reply)),
            Err(Error::Api { message, .. }) if masih_latih(&message) => {
                if attempt == 2 || !ada_siap(api)? {
                    return Ok(None);
                }
                std::thread::sleep(std::time::Duration::from_millis(BATTLE_RETRY_MS));
            }
            Err(err) => return Err(err),
        }
    }
    Ok(None)
}

fn serangan(ctx: &mut Konteks, sebelum: &Snapshot) -> Result<Hasil> {
    let api = ctx.api;

    if ctx.sim.is_none() {
        *ctx.sim = Some(Sim::load()?);
    }

    let mut weak = 0;
    for attempt in 1..=SCAN_TARGETS {
        let Some(target) = tempur(api, || {
            if attempt == 1 {
                raid::find_target(api)
            } else {
                raid::next_target(api)
            }
        })?
        else {
            return Ok(Hasil::Kosong);
        };
        let mut battle = match Battle::from_reply(&target, ctx.path_fix_from, ctx.air_fix_from) {
            Some(battle) => battle,
            None => {
                log::ly("The target reply carried no map so the raid is skipped");
                return Ok(Hasil::Ditolak);
            }
        };
        if battle.empty_army() {
            battle.army = full_army(sebelum);
        }
        if battle.empty_army() {
            log::ly("There are no troops to send so the raid is skipped");
            return Ok(Hasil::Ditolak);
        }

        let id = battle.id.clone();
        if id.is_null() {
            log::ly("The target reply carried no battle id so the raid is skipped");
            return Ok(Hasil::Ditolak);
        }

        let sim = ctx.sim.as_mut().expect("the simulation was just loaded");
        let outcome = sim.run(&battle)?;

        if !outcome.verified {
            log::lr("The battle log does not replay so the raid is skipped");
            return Ok(Hasil::Ditolak);
        }

        if outcome.stars < RAID_STARS_FLOOR {
            weak += 1;
            if attempt < SCAN_TARGETS {
                continue;
            }
            log::ly(&format!(
                "{} targets would end below the {} {} floor so the raid waits",
                weak,
                RAID_STARS_FLOOR,
                log::unit_word(RAID_STARS_FLOOR as u64, "star", "stars")
            ));
            return Ok(Hasil::Ditolak);
        }

        let done = match kirim(api, &id, &outcome) {
            Ok(done) => done,
            Err(Error::Api { status, message }) => {
                log::ly(&format!(
                    "The battle check answered {} so the raid is skipped because {}",
                    status, message
                ));
                return Ok(Hasil::Ditolak);
            }
            Err(other) => return Err(other),
        };

        let (stars, gold) = match spoils(&done) {
            Some(spoils) => spoils,
            None => {
                log::ly(&format!(
                    "The battle check turned the raid down because {}",
                    sebab(&done)
                ));
                return Ok(Hasil::Ditolak);
            }
        };
        log::lg(&format!(
            "Raid finished with {} {} and {} gold",
            stars,
            log::unit_word(stars, "star", "stars"),
            gold
        ));

        return Ok(Hasil::Selesai);
    }

    Ok(Hasil::Ditolak)
}

fn kirim(api: &Api, id: &Value, outcome: &Outcome) -> Result<Value> {
    let mut done = raid::battle_finish(api, id.clone(), outcome.log.clone(), outcome.hash.clone())?;

    for _ in 0..FINISH_TRIES {
        let late = done.get("status").and_then(|v| v.as_str()) == Some("error")
            && done.get("wait_ms").and_then(|v| v.as_u64()).is_some();
        if !late {
            return Ok(done);
        }
        let wait = done.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        std::thread::sleep(std::time::Duration::from_millis(wait + 400));
        done = raid::battle_finish(api, id.clone(), outcome.log.clone(), outcome.hash.clone())?;
    }

    Ok(done)
}

fn spoils(done: &Value) -> Option<(u64, u64)> {
    if done.get("status").and_then(|v| v.as_str()) == Some("error") {
        return None;
    }
    let stars = done.get("stars").and_then(|v| v.as_u64())?;
    let gold = done
        .get("result")
        .and_then(|r| r.get("loot"))
        .and_then(|l| l.get("gold"))
        .and_then(|v| v.as_u64())
        .or_else(|| done.get("gold").and_then(|v| v.as_u64()))
        .unwrap_or(0);
    Some((stars, gold))
}

fn sebab(done: &Value) -> String {
    done.get("error")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "the battle check did not take the log".to_string())
}

fn masuk_pintu(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("entry") || lower.contains("pay the")
}

fn full_army(state: &Snapshot) -> Value {
    let kinds = state.army_kinds();
    if kinds.is_empty() {
        return Value::Null;
    }
    let mut map = serde_json::Map::new();
    for (kind, count) in kinds {
        map.insert(kind, json!(count));
    }
    Value::Object(map)
}

fn army_shape(state: &Snapshot, kind: &str, count: u64) -> Value {
    let mut map = serde_json::Map::new();
    for (name, have) in state.army_kinds() {
        map.insert(name, json!(have));
    }
    let current = map.get(kind).and_then(|v| v.as_u64()).unwrap_or(0);
    map.insert(kind.to_string(), json!(current.max(count)));
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stars_and_the_gold_come_from_the_server_reply() {
        let done = json!({
            "dark": 0,
            "gold": 0,
            "stars": 2,
            "elixir": 0,
            "status": "done",
            "campaign": { "lvl": 3, "best": 2, "new_best": true },
            "destruction": 97,
            "result": {
                "ticks": 1477,
                "stars": 2,
                "loot": { "gold": 1400, "elixir": 1400, "dark": 0 }
            }
        });

        assert_eq!(spoils(&done), Some((2, 1400)));
    }

    #[test]
    fn a_battle_that_is_too_early_is_not_a_finished_battle() {
        let done = json!({ "status": "error", "error": "too early", "wait_ms": 4698 });

        assert_eq!(spoils(&done), None);
        assert_eq!(sebab(&done), "too early");
    }

    #[test]
    fn a_reply_without_stars_is_not_a_finished_battle() {
        let done = json!({ "status": "done" });

        assert_eq!(spoils(&done), None);
    }
}
