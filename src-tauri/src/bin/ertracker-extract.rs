//! Runs the map extraction without the UI.
//!
//! Usage: ertracker-extract <out dir> [path\to\ELDEN RING]

use ertracker_lib::extract::build;
use ertracker_lib::paths;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = PathBuf::from(args.next().expect("usage: ertracker-extract <out dir> [game dir]"));
    let root = args.next().map(PathBuf::from).or_else(paths::find_game_dir).expect("game folder not found");
    let start = Instant::now();
    let manifest = build::build(&root, &out, |p| {
        if p.done % 200 == 0 || p.done == p.total {
            print!("\r{} {}/{}   ", p.step, p.done, p.total);
            let _ = std::io::stdout().flush();
        }
    })
    .unwrap_or_else(|e| panic!("{e}"));
    println!();
    for m in &manifest.maps {
        println!("{:?}: {}x{} px", m.id, m.width, m.height);
    }
    println!("done in {:.1?} -> {}", start.elapsed(), out.display());
}
