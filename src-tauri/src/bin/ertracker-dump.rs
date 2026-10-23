//! Prints what the tracker reads from a save, without the UI.
//!
//! Usage: ertracker-dump [path\to\ER0000.sl2] [--watch]

use ertracker_lib::{paths, progress, save::SaveFile};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .or_else(|| {
            let saves = paths::find_saves();
            for s in &saves {
                println!("save found: {} (SteamID {})", s.path.display(), s.steam_id);
            }
            saves.into_iter().next().map(|s| s.path)
        })
        .expect("no save file found");

    match paths::find_game_dir() {
        Some(dir) => println!("game folder: {}", dir.display()),
        None => println!("game folder: not found"),
    }
    println!("reading {}\n", path.display());

    let start = Instant::now();
    let save = SaveFile::load(&path).unwrap_or_else(|e| panic!("{e}"));
    let parsed = start.elapsed();

    let db = progress::db();
    for character in &save.characters {
        let start = Instant::now();
        let done = progress::completed(&save, character);
        let computed = start.elapsed();
        let h = character.seconds_played / 3600;
        let m = character.seconds_played / 60 % 60;
        println!(
            "slot {} - {} - level {} - {h}h{m:02} - save v{} - checksum {}",
            character.slot,
            character.name,
            character.level,
            character.version,
            if character.checksum_ok { "ok" } else { "MISMATCH" }
        );
        if let Some(p) = &character.position {
            println!("  position: {} ({:.1}, {:.1}, {:.1})", p.map, p.x, p.y, p.z);
        }
        let mut per_category: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
        for entry in &db.entries {
            let c = per_category.entry(&entry.category).or_default();
            c.1 += 1;
            if done.contains(&entry.id) {
                c.0 += 1;
            }
        }
        for (category, (n, total)) in per_category {
            println!("  {category:<14} {n:>4} / {total}");
        }
        println!("  (progress computed in {computed:.2?})\n");
    }
    println!("file parsed in {parsed:.2?}");
}
