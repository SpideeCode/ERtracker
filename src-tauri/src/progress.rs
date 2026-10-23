use crate::save::{Character, SaveFile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Deserialize)]
pub struct Db {
    pub entries: Vec<Entry>,
    pub quests: Vec<Quest>,
}

#[derive(Deserialize)]
pub struct Entry {
    pub id: String,
    pub category: String,
    pub name: String,
    pub flag: u32,
}

#[derive(Deserialize)]
pub struct Quest {
    pub id: String,
    pub steps: Vec<QuestStep>,
}

#[derive(Deserialize)]
pub struct QuestStep {
    /// (flag id, expected value)
    pub flags: Vec<(u32, u8)>,
}

static DB: OnceLock<Db> = OnceLock::new();

pub fn db() -> &'static Db {
    DB.get_or_init(|| serde_json::from_str(include_str!("../../data/db.json")).expect("data/db.json is invalid"))
}

/// Ids of completed entries, plus `quest:<npc>#<step>` for every quest step whose flags all match.
pub fn completed(save: &SaveFile, character: &Character) -> BTreeSet<String> {
    let db = db();
    let mut done: BTreeSet<String> = db
        .entries
        .iter()
        .filter(|e| save.flag(character, e.flag) == Some(true))
        .map(|e| e.id.clone())
        .collect();
    for quest in &db.quests {
        for (i, step) in quest.steps.iter().enumerate() {
            let matches = step
                .flags
                .iter()
                .all(|&(id, value)| save.flag(character, id) == Some(value != 0));
            if matches && !step.flags.is_empty() {
                done.insert(format!("{}#{i}", quest.id));
            }
        }
    }
    done
}

#[derive(Clone, Serialize)]
pub struct Diff {
    pub gained: Vec<String>,
    pub lost: Vec<String>,
}

pub fn diff(before: &BTreeSet<String>, after: &BTreeSet<String>) -> Diff {
    Diff {
        gained: after.difference(before).cloned().collect(),
        lost: before.difference(after).cloned().collect(),
    }
}
