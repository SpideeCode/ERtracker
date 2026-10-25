use std::process::Command;

fn main() {
    // Gives the exe the ERtracker icon in Explorer and the taskbar.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../src-tauri/icons/icon.ico");
        res.compile().expect("failed to embed the icon");
    }

    // Last commit touching the launcher: compared with the release manifest to
    // know when the launcher itself must update.
    let commit = Command::new("git")
        .args(["log", "-1", "--format=%H", "--", "."])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    println!("cargo:rustc-env=LAUNCHER_COMMIT={commit}");
    for path in ["src", "build.rs", "../.git/HEAD", "../.git/refs/heads", "../.git/packed-refs"] {
        println!("cargo:rerun-if-changed={path}");
    }
}
