use serde::Deserialize;
use serde_json::Value;

use crate::state::Katalog;
use crate::support::error::{Error, Result};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Building {
    #[serde(default)]
    pub q: i32,
    #[serde(default)]
    pub r: i32,
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub stock: f64,
    #[serde(default)]
    pub secs_left: Option<f64>,
    #[serde(default)]
    pub target_level: Option<i64>,
    #[serde(default)]
    pub res: Option<String>,
}

impl Building {
    pub fn busy(&self) -> bool {
        self.secs_left.is_some()
    }

    pub fn waiting_for_stock(&self) -> bool {
        self.stock > 0.0 && !self.busy()
    }

    pub fn wants_level(&self) -> Option<i64> {
        let target = self.target_level?;
        if target > self.level {
            Some(target)
        } else {
            None
        }
    }

    pub fn seconds_left(&self) -> u64 {
        self.secs_left.unwrap_or(0.0).max(0.0) as u64
    }
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    raw: Value,
}

impl Snapshot {
    pub fn parse(raw: Value) -> Result<Snapshot> {
        if !raw.is_object() {
            return Err(Error::State(
                "the state reply was not an object".to_string(),
            ));
        }
        Ok(Snapshot { raw })
    }

    pub fn raw(&self) -> &Value {
        &self.raw
    }

    fn player(&self) -> &Value {
        self.raw.get("player").unwrap_or(&Value::Null)
    }

    fn number(&self, key: &str) -> f64 {
        self.player()
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn id(&self) -> String {
        self.player()
            .get("id")
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn name(&self) -> String {
        self.player()
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string()
    }

    pub fn gold(&self) -> u64 {
        self.number("gold").max(0.0) as u64
    }

    pub fn elixir(&self) -> u64 {
        self.number("elixir").max(0.0) as u64
    }

    pub fn dark(&self) -> u64 {
        self.number("dark").max(0.0) as u64
    }

    pub fn tokens(&self) -> u64 {
        self.number("token").max(0.0) as u64
    }

    pub fn tokens_f64(&self) -> f64 {
        self.number("token").max(0.0)
    }

    pub fn town_hall(&self) -> u64 {
        self.number("th").max(0.0) as u64
    }

    pub fn clan_castle(&self) -> Value {
        self.player().get("cc").cloned().unwrap_or(Value::Null)
    }

    pub fn gems(&self) -> u64 {
        self.number("gems").max(0.0) as u64
    }

    pub fn cap_gold(&self) -> u64 {
        self.number("cap_gold").max(0.0) as u64
    }

    pub fn cap_elixir(&self) -> u64 {
        self.number("cap_elixir").max(0.0) as u64
    }

    pub fn attacks(&self) -> u64 {
        self.number("attacks").max(0.0) as u64
    }

    pub fn builders(&self) -> u64 {
        self.number("builders").max(0.0) as u64
    }

    pub fn free_builders(&self) -> u64 {
        self.number("free_builders").max(0.0) as u64
    }

    pub fn army_secs(&self) -> Option<u64> {
        self.player()
            .get("army_secs")
            .and_then(|v| v.as_f64())
            .map(|v| v.max(0.0) as u64)
            .filter(|secs| *secs > 0)
    }

    pub fn shield_secs(&self) -> u64 {
        self.number("shield_secs").max(0.0) as u64
    }

    pub fn village_radius(&self) -> i64 {
        self.player()
            .get("village_radius")
            .and_then(|v| v.as_i64())
            .unwrap_or(8)
    }

    pub fn trophies(&self) -> u64 {
        self.number("trophies").max(0.0) as u64
    }

    pub fn in_clan(&self) -> bool {
        match self.player().get("clan") {
            Some(Value::Null) | None => false,
            Some(Value::Array(items)) => !items.is_empty(),
            Some(value) => !value.is_null(),
        }
    }

    pub fn clan_tag(&self) -> Option<String> {
        let clan = self.player().get("clan")?;
        if clan.is_null() {
            return None;
        }
        if let Some(tag) = clan.get("tag").and_then(|v| v.as_str()) {
            return Some(tag.to_string());
        }
        if let Some(name) = clan.get("name").and_then(|v| v.as_str()) {
            return Some(name.to_string());
        }
        clan.as_str().map(|s| s.to_string())
    }

    pub fn army_total(&self) -> u64 {
        let army = self.player().get("army_ready").and_then(|v| v.as_object());
        match army {
            Some(map) => map
                .values()
                .map(|v| v.as_f64().unwrap_or(0.0).max(0.0) as u64)
                .sum(),
            None => 0,
        }
    }

    pub fn army_kinds(&self) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        if let Some(map) = self.player().get("army_ready").and_then(|v| v.as_object()) {
            for (kind, value) in map {
                let count = value.as_f64().unwrap_or(0.0).max(0.0) as u64;
                if count > 0 {
                    out.push((kind.clone(), count));
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn buildings(&self) -> Vec<Building> {
        self.raw
            .get("buildings")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| serde_json::from_value::<Building>(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn reward_gold(&self) -> u64 {
        self.number("reward_gold").max(0.0) as u64
    }

    pub fn gold_cap(&self) -> u64 {
        self.number("cap_gold").max(0.0) as u64
    }

    pub fn barracks_level(&self) -> i64 {
        self.buildings()
            .iter()
            .filter(|building| building.kind == "barracks")
            .map(|building| building.level)
            .max()
            .unwrap_or(0)
    }

    pub fn housing_cap(&self, katalog: &Katalog) -> u64 {
        self.buildings()
            .iter()
            .filter(|building| building.kind == "army_camp")
            .map(|building| katalog.camp_cap(building.level))
            .sum()
    }

    pub fn housing_used(&self, katalog: &Katalog) -> u64 {
        self.army_kinds()
            .iter()
            .map(|(kind, count)| katalog.troop_housing(kind).unwrap_or(0) * count)
            .sum()
    }

    pub fn obstacles(&self) -> Vec<Building> {
        self.raw
            .get("obstacles")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| serde_json::from_value::<Building>(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn ready_obstacles(&self) -> Vec<Building> {
        self.obstacles()
            .into_iter()
            .filter(|item| !item.busy())
            .collect()
    }

    pub fn waiting_stock(&self) -> Vec<Building> {
        self.buildings()
            .into_iter()
            .filter(|item| item.waiting_for_stock())
            .collect()
    }

    pub fn stock_total(&self) -> f64 {
        self.buildings().iter().map(|item| item.stock).sum()
    }

    pub fn busy_buildings(&self) -> Vec<Building> {
        self.buildings()
            .into_iter()
            .filter(|item| item.busy())
            .collect()
    }

    pub fn nearest_finish(&self) -> Option<u64> {
        self.busy_buildings()
            .iter()
            .map(|item| item.seconds_left())
            .min()
    }

    pub fn lab_idle(&self) -> bool {
        match self.raw.get("lab_job") {
            None => true,
            Some(Value::Null) => true,
            Some(_) => false,
        }
    }

    pub fn sell_room(&self) -> f64 {
        self.raw
            .get("sell")
            .and_then(|v| v.get("room_usd"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn sell_cap(&self) -> f64 {
        self.raw
            .get("sell")
            .and_then(|v| v.get("cap_usd"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn limit(&self, kind: &str) -> (u64, u64, i64) {
        let entry = self
            .raw
            .get("limits")
            .and_then(|v| v.get(kind))
            .cloned()
            .unwrap_or(Value::Null);
        let have = entry.get("have").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;
        let max = entry.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;
        let max_level = entry.get("max_level").and_then(|v| v.as_i64()).unwrap_or(0);
        (have, max, max_level)
    }

    pub fn limit_kinds(&self) -> Vec<String> {
        self.raw
            .get("limits")
            .and_then(|v| v.as_object())
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn missing_builds(&self) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        for kind in self.limit_kinds() {
            let (have, max, _) = self.limit(kind.as_str());
            if max > have {
                out.push((kind, max - have));
            }
        }
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Snapshot {
        Snapshot::parse(json!({
            "player": {
                "id": 41,
                "name": "tester",
                "gold": 1200,
                "elixir": 800,
                "dark": 0,
                "token": 2,
                "attacks": 2,
                "builders": 1,
                "free_builders": 1,
                "army_secs": null,
                "shield_secs": 82850,
                "village_radius": 8,
                "clan": null,
                "army_ready": {"barbarian": 12, "archer": 4}
            },
            "buildings": [
                {"q": 0, "r": 0, "id": 1, "kind": "town_hall", "level": 1, "stock": 0, "secs_left": null},
                {"q": 1, "r": 0, "id": 2, "kind": "gold_mine", "level": 1, "stock": 250, "secs_left": null},
                {"q": 2, "r": 0, "id": 3, "kind": "elixir_pump", "level": 1, "stock": 0, "secs_left": 120.0, "target_level": 2}
            ],
            "army": {"barbarian": 12, "archer": 4},
            "obstacles": [{"q": 3, "r": 3, "id": 7, "kind": "tree"}],
            "lab_job": null,
            "sell": {"room_usd": 2.5, "cap_usd": 10.0},
            "limits": {"gold_mine": {"have": 1, "max": 3, "max_level": 5}}
        }))
        .unwrap()
    }

    #[test]
    fn the_village_is_read_from_the_state() {
        let state = fixture();
        assert_eq!(state.gold(), 1200);
        assert_eq!(state.attacks(), 2);
        assert_eq!(state.army_total(), 16);
        assert!(!state.in_clan());
        assert!(state.lab_idle());
        assert_eq!(state.nearest_finish(), Some(120));
        assert_eq!(state.waiting_stock().len(), 1);
        assert_eq!(state.missing_builds(), vec![("gold_mine".to_string(), 2)]);
    }

    #[test]
    fn a_busy_building_is_not_waiting_for_stock() {
        let state = fixture();
        let busy = state.busy_buildings();
        assert_eq!(busy.len(), 1);
        assert!(busy[0].busy());
        assert!(!busy[0].waiting_for_stock());
        assert_eq!(busy[0].wants_level(), Some(2));
    }
}
