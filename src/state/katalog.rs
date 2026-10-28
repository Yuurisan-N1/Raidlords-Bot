use std::collections::HashMap;

use serde_json::Value;

use crate::support::error::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Biaya {
    pub level: i64,
    pub res: String,
    pub cost: u64,
    pub secs: u64,
    pub gate: i64,
}

impl Biaya {
    fn read(item: &Value, gate_key: &str) -> Option<Biaya> {
        let level = item.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
        let res = item
            .get("res")
            .and_then(|v| v.as_str())
            .unwrap_or("gold")
            .to_string();
        let cost = item
            .get("cost")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .max(0.0) as u64;
        let secs = item
            .get("secs")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .max(0.0) as u64;
        let gate = item
            .get(gate_key)
            .and_then(|v| v.as_i64())
            .or_else(|| item.get("th_req").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        Some(Biaya {
            level,
            res,
            cost,
            secs,
            gate,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Katalog {
    levels: HashMap<String, Vec<Biaya>>,
    obstacles: HashMap<String, Biaya>,
    research: HashMap<String, Vec<Biaya>>,
    heroes: HashMap<String, Vec<Biaya>>,
    search: Vec<u64>,
}

impl Katalog {
    pub fn parse(raw: &Value) -> Result<Katalog> {
        let levels = read_steps(raw.get("levels"), "th_req")?;
        if levels.is_empty() {
            return Err(Error::State(
                "the catalog carried no building levels".to_string(),
            ));
        }

        let mut obstacles = HashMap::new();
        if let Some(map) = raw.get("obstacles").and_then(|v| v.as_object()) {
            for (kind, item) in map {
                if let Some(mut biaya) = Biaya::read(item, "th_req") {
                    biaya.level = 0;
                    obstacles.insert(kind.clone(), biaya);
                }
            }
        }

        let search = raw
            .get("search_cost")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_f64())
                    .map(|value| value.max(0.0) as u64)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Katalog {
            levels,
            obstacles,
            research: read_steps(raw.get("research"), "lab")?,
            heroes: read_steps(raw.get("heroes"), "hall")?,
            search,
        })
    }

    pub fn ceiling(&self, kind: &str) -> i64 {
        self.levels
            .get(kind)
            .and_then(|steps| steps.iter().map(|step| step.level).max())
            .unwrap_or(0)
    }

    pub fn next(&self, kind: &str, level: i64) -> Option<Biaya> {
        self.step(kind, level + 1)
    }

    pub fn first(&self, kind: &str) -> Option<Biaya> {
        self.step(kind, 1)
    }

    pub fn step(&self, kind: &str, level: i64) -> Option<Biaya> {
        let steps = self.levels.get(kind)?;
        steps.iter().find(|step| step.level == level).cloned()
    }

    pub fn priced(&self, kind: &str) -> bool {
        self.levels.contains_key(kind)
    }

    pub fn obstacle(&self, kind: &str) -> Option<Biaya> {
        self.obstacles.get(kind).cloned()
    }

    pub fn research_next(&self, troop: &str, level: i64) -> Option<Biaya> {
        let steps = self.research.get(troop)?;
        steps.iter().find(|step| step.level == level + 1).cloned()
    }

    pub fn hero_next(&self, hero: &str, level: i64) -> Option<Biaya> {
        let steps = self.heroes.get(hero)?;
        steps.iter().find(|step| step.level == level + 1).cloned()
    }

    pub fn hero_cheapest(&self, hero: &str) -> Option<Biaya> {
        let steps = self.heroes.get(hero)?;
        steps.iter().min_by_key(|step| step.cost).cloned()
    }

    pub fn search_cost(&self, town_hall: u64) -> u64 {
        if self.search.is_empty() {
            return 0;
        }
        let index = town_hall.saturating_sub(1) as usize;
        self.search
            .get(index)
            .copied()
            .unwrap_or_else(|| *self.search.last().unwrap_or(&0))
    }
}

fn read_steps(source: Option<&Value>, gate_key: &str) -> Result<HashMap<String, Vec<Biaya>>> {
    let mut out: HashMap<String, Vec<Biaya>> = HashMap::new();
    let map = match source {
        Some(Value::Object(map)) => map,
        _ => return Ok(out),
    };

    for (kind, items) in map {
        let list = match items.as_array() {
            Some(list) => list,
            None => continue,
        };
        let mut steps: Vec<Biaya> = list
            .iter()
            .filter_map(|item| Biaya::read(item, gate_key))
            .collect();
        if steps.is_empty() {
            continue;
        }
        steps.sort_by_key(|step| step.level);
        out.insert(kind.clone(), steps);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn catalog() -> Katalog {
        Katalog::parse(&json!({
            "levels": {
                "gold_mine": [
                    {"level": 1, "res": "gold", "cost": 150, "secs": 2, "th_req": 1},
                    {"level": 2, "res": "gold", "cost": 300, "secs": 7, "th_req": 1},
                    {"level": 3, "res": "gold", "cost": 700, "secs": 30, "th_req": 2}
                ],
                "town_hall": [
                    {"level": 1, "res": "gold", "cost": 0, "secs": 0, "th_req": 1},
                    {"level": 2, "res": "gold", "cost": 1000, "secs": 5, "th_req": 1}
                ]
            },
            "obstacles": {
                "stone": {"res": "gold", "cost": 500, "secs": 10, "name": "Stone"}
            },
            "research": {
                "heal": [
                    {"level": 2, "res": "elixir", "cost": 75000, "secs": 5400, "lab": 2}
                ]
            },
            "heroes": {
                "aq": [
                    {"level": 2, "res": "dark", "cost": 5000, "secs": 3600, "hall": 2}
                ]
            },
            "search_cost": [5, 10, 50]
        }))
        .unwrap()
    }

    #[test]
    fn the_next_step_carries_its_own_price() {
        let katalog = catalog();
        let step = katalog.next("gold_mine", 1).unwrap();
        assert_eq!(step.cost, 300);
        assert_eq!(step.res, "gold");
        assert_eq!(step.secs, 7);
        assert_eq!(katalog.ceiling("gold_mine"), 3);
        assert_eq!(katalog.first("gold_mine").unwrap().cost, 150);
    }

    #[test]
    fn a_kind_that_is_not_in_the_catalog_has_no_price() {
        let katalog = catalog();
        assert!(katalog.next("deco_bush", 1).is_none());
        assert!(!katalog.priced("deco_bush"));
        assert!(katalog.priced("gold_mine"));
    }

    #[test]
    fn obstacles_research_and_heroes_are_priced_too() {
        let katalog = catalog();
        assert_eq!(katalog.obstacle("stone").unwrap().cost, 500);
        assert_eq!(katalog.research_next("heal", 1).unwrap().cost, 75000);
        assert_eq!(katalog.research_next("heal", 1).unwrap().gate, 2);
        assert_eq!(katalog.hero_next("aq", 1).unwrap().cost, 5000);
        assert_eq!(katalog.hero_cheapest("aq").unwrap().level, 2);
    }

    #[test]
    fn the_search_price_follows_the_town_hall() {
        let katalog = catalog();
        assert_eq!(katalog.search_cost(1), 5);
        assert_eq!(katalog.search_cost(2), 10);
        assert_eq!(katalog.search_cost(9), 50);
    }

    #[test]
    fn a_catalog_without_levels_is_rejected() {
        assert!(Katalog::parse(&json!({"levels": {}})).is_err());
        assert!(Katalog::parse(&json!({"counts": {}})).is_err());
    }
}
