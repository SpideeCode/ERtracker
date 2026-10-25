//! ERtracker launcher: installs itself, keeps ERtracker on the latest GitHub
//! release (built by CI on every commit) and starts it with the "JOUER" button.

#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, RichText};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const GOLD: Color32 = Color32::from_rgb(201, 166, 92);
const RELEASE_URL: &str = "https://github.com/SpideeCode/ERtracker/releases/download/latest";
const APP_EXE: &str = "ertracker.exe";
const LAUNCHER_EXE: &str = "ertracker-launcher.exe";
/// Passed to the launcher restarted after replacing itself, so a commit mismatch
/// can never make it update in a loop.
const JUST_UPDATED: &str = "--updated";

#[derive(Deserialize)]
struct Manifest {
    commit: String,
    app: Asset,
    launcher: Asset,
    commits: Vec<Commit>,
}

#[derive(Deserialize)]
struct Asset {
    file: String,
    size: u64,
    sha256: String,
    #[serde(default)]
    commit: String,
}

#[derive(Clone, Deserialize)]
struct Commit {
    sha: String,
    subject: String,
    date: String,
}

#[derive(Clone, PartialEq)]
enum Phase {
    Checking,
    UpdatingLauncher,
    Downloading,
    Ready,
    Failed(String),
}

struct State {
    phase: Phase,
    /// Bytes received / expected for the current download.
    progress: (u64, u64),
    current: Option<Commit>,
    /// Commits since the installed version: the "patch notes".
    news: Vec<Commit>,
    note: Option<String>,
}

type Shared = Arc<Mutex<State>>;

fn set_phase(state: &Shared, phase: Phase) {
    state.lock().unwrap().phase = phase;
}

/// %LOCALAPPDATA%\ERtracker: the app, the installed launcher and the installed version.
fn install_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map_or_else(|| PathBuf::from("."), PathBuf::from).join("ERtracker")
}

fn release_url() -> String {
    // Lets a local server stand in for GitHub when testing.
    std::env::var("ERTRACKER_RELEASE_URL").unwrap_or_else(|_| RELEASE_URL.into())
}

/// A launcher started from a cargo target dir is a dev build: it never replaces itself.
fn is_dev_build(exe: &Path) -> bool {
    exe.components().any(|c| c.as_os_str() == "target")
}

fn hidden(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

fn app_running() -> bool {
    hidden("tasklist")
        .args(["/FO", "CSV", "/NH", "/FI", "IMAGENAME eq ertracker.exe"])
        .output()
        .is_ok_and(|out| String::from_utf8_lossy(&out.stdout).to_lowercase().contains("\"ertracker.exe\""))
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .user_agent("ertracker-launcher")
        .build()
}

fn fetch_manifest() -> Result<Manifest, String> {
    let resp = agent()
        .get(&format!("{}/manifest.json", release_url()))
        .set("Cache-Control", "no-cache")
        .call()
        .map_err(|e| e.to_string())?;
    serde_json::from_reader(resp.into_reader()).map_err(|e| e.to_string())
}

/// Downloads an asset next to `dest` and checks its SHA-256; returns the temporary file.
fn download(state: &Shared, asset: &Asset, dest: &Path) -> Result<PathBuf, String> {
    let resp = agent().get(&format!("{}/{}", release_url(), asset.file)).call().map_err(|e| e.to_string())?;
    let part = dest.with_extension("part");
    let mut file = File::create(&part).map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut hasher = Sha256::new();
    let mut buf = vec![0; 1 << 16];
    let mut done = 0;
    state.lock().unwrap().progress = (0, asset.size);
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        hasher.update(&buf[..n]);
        done += n as u64;
        state.lock().unwrap().progress = (done, asset.size);
    }
    drop(file);
    if format!("{:x}", hasher.finalize()) != asset.sha256 {
        let _ = std::fs::remove_file(&part);
        return Err(format!("{} est corrompu, réessaie.", asset.file));
    }
    Ok(part)
}

/// Copies the launcher into the install dir and adds the Desktop and Start menu
/// shortcuts, the first time it runs from somewhere else (e.g. Downloads).
fn install_self(exe: &Path) {
    let installed = install_dir().join(LAUNCHER_EXE);
    if is_dev_build(exe) || installed.is_file() || std::fs::copy(exe, &installed).is_err() {
        return;
    }
    let target = installed.display().to_string().replace('\'', "''");
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
         foreach ($d in [Environment]::GetFolderPath('Desktop'), [Environment]::GetFolderPath('Programs')) {{ \
           $s = $w.CreateShortcut((Join-Path $d 'ERtracker.lnk')); $s.TargetPath = '{target}'; \
           $s.WorkingDirectory = (Split-Path '{target}'); $s.Save() }}"
    );
    let _ = hidden("powershell").args(["-NoProfile", "-Command", &script]).status();
}

/// Swaps the running exe for the new one and restarts. Windows allows renaming a
/// running exe, not overwriting it.
fn replace_self(state: &Shared, manifest: &Manifest, exe: &Path) -> Result<(), String> {
    let part = download(state, &manifest.launcher, exe)?;
    let old = exe.with_extension("old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(exe, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&part, exe) {
        let _ = std::fs::rename(&old, exe);
        return Err(e.to_string());
    }
    Command::new(exe).arg(JUST_UPDATED).spawn().map_err(|e| e.to_string())?;
    std::process::exit(0);
}

fn update(state: Shared, force: bool) {
    set_phase(&state, Phase::Checking);
    let dir = install_dir();
    let app = dir.join(APP_EXE);
    let version_file = dir.join("version.txt");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return set_phase(&state, Phase::Failed(format!("Impossible de créer {} : {e}", dir.display())));
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let _ = std::fs::remove_file(exe.with_extension("old"));
    install_self(&exe);

    let installed = std::fs::read_to_string(&version_file).unwrap_or_default().trim().to_owned();
    let manifest = match fetch_manifest() {
        Ok(m) => m,
        Err(e) if app.is_file() => {
            state.lock().unwrap().note = Some(format!("Impossible de vérifier les mises à jour ({e}) : version installée."));
            return set_phase(&state, Phase::Ready);
        }
        Err(e) => return set_phase(&state, Phase::Failed(format!("Téléchargement impossible : {e}"))),
    };

    // The launcher first, so a new launcher handles the rest.
    let own = env!("LAUNCHER_COMMIT");
    let just_updated = std::env::args().any(|a| a == JUST_UPDATED);
    if !just_updated && !is_dev_build(&exe) && !manifest.launcher.commit.is_empty() && manifest.launcher.commit != own {
        set_phase(&state, Phase::UpdatingLauncher);
        if let Err(e) = replace_self(&state, &manifest, &exe) {
            state.lock().unwrap().note = Some(format!("Mise à jour du launcher impossible : {e}"));
        }
    }

    {
        let mut s = state.lock().unwrap();
        s.current = manifest.commits.iter().find(|c| c.sha == installed).cloned();
        if !installed.is_empty() && installed != manifest.commit {
            s.news = manifest.commits.iter().take_while(|c| c.sha != installed).cloned().collect();
        }
    }

    if force || installed != manifest.commit || !app.is_file() {
        set_phase(&state, Phase::Downloading);
        let part = match download(&state, &manifest.app, &app) {
            Ok(part) => part,
            Err(e) => return set_phase(&state, Phase::Failed(format!("Téléchargement impossible : {e}"))),
        };
        // The running exe is locked and can't be replaced.
        if app_running() {
            let _ = hidden("taskkill").args(["/IM", APP_EXE, "/F"]).status();
            std::thread::sleep(Duration::from_millis(500));
        }
        if let Err(e) = std::fs::rename(&part, &app) {
            return set_phase(&state, Phase::Failed(format!("Installation impossible : {e}")));
        }
        let _ = std::fs::write(&version_file, &manifest.commit);
        state.lock().unwrap().current = manifest.commits.first().cloned();
    }

    set_phase(&state, Phase::Ready);
}

struct Launcher {
    state: Shared,
    icon: egui::TextureHandle,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext, icon: &egui::IconData) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let image = egui::ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba);
        let launcher = Self {
            state: Arc::new(Mutex::new(State {
                phase: Phase::Checking,
                progress: (0, 0),
                current: None,
                news: Vec::new(),
                note: None,
            })),
            icon: cc.egui_ctx.load_texture("icon", image, Default::default()),
        };
        launcher.start(false);
        launcher
    }

    fn start(&self, force: bool) {
        self.state.lock().unwrap().note = None;
        let state = self.state.clone();
        std::thread::spawn(move || update(state, force));
    }

    fn play(&self, ctx: &egui::Context) {
        if !app_running() {
            let dir = install_dir();
            if let Err(e) = Command::new(dir.join(APP_EXE)).current_dir(&dir).spawn() {
                self.state.lock().unwrap().phase = Phase::Failed(format!("Impossible de lancer ERtracker : {e}"));
                return;
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (phase, busy) = {
            let s = self.state.lock().unwrap();
            let busy = !matches!(s.phase, Phase::Ready | Phase::Failed(_));
            (s.phase.clone(), busy)
        };
        if busy {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let play = egui::Button::new(RichText::new("JOUER").size(22.0).strong().color(Color32::BLACK))
                    .fill(GOLD)
                    .min_size(egui::vec2(200.0, 50.0));
                if ui.add_enabled(phase == Phase::Ready, play).clicked() {
                    self.play(ctx);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if matches!(phase, Phase::Failed(_)) && ui.button("Réessayer").clicked() {
                        self.start(false);
                    }
                    if !busy && ui.button("Réinstaller").on_hover_text("Retélécharge la dernière version").clicked() {
                        self.start(true);
                    }
                });
            });
            ui.add_space(12.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let s = self.state.lock().unwrap();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(egui::Image::new(egui::load::SizedTexture::from_handle(&self.icon)).fit_to_exact_size(egui::vec2(64.0, 64.0)));
                ui.vertical(|ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new("ERtracker").size(30.0).strong().color(GOLD));
                    let version = match &s.current {
                        Some(c) => format!("Version {} du {}", &c.sha[..7.min(c.sha.len())], c.date),
                        None => "Version …".into(),
                    };
                    ui.label(RichText::new(version).weak());
                });
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let status = match &phase {
                    Phase::Checking => "Recherche de mises à jour…",
                    Phase::UpdatingLauncher => "Mise à jour du launcher…",
                    Phase::Downloading => "Téléchargement de la mise à jour…",
                    Phase::Ready => "Prêt à jouer",
                    Phase::Failed(msg) => msg,
                };
                if busy {
                    ui.spinner();
                }
                let color = match phase {
                    Phase::Ready => Color32::from_rgb(120, 200, 120),
                    Phase::Failed(_) => Color32::from_rgb(230, 100, 90),
                    _ => ui.visuals().text_color(),
                };
                ui.label(RichText::new(status).size(16.0).color(color));
            });
            if matches!(phase, Phase::Downloading | Phase::UpdatingLauncher) {
                let (done, total) = s.progress;
                let mb = |b: u64| b as f32 / 1_048_576.0;
                ui.add(
                    egui::ProgressBar::new(if total > 0 { done as f32 / total as f32 } else { 0.0 })
                        .text(format!("{:.1} / {:.1} Mo", mb(done), mb(total)))
                        .fill(GOLD.gamma_multiply(0.7)),
                );
            }
            if let Some(note) = &s.note {
                ui.label(RichText::new(note).color(Color32::from_rgb(230, 190, 90)));
            }
            ui.add_space(10.0);

            if !s.news.is_empty() {
                ui.label(RichText::new("Nouveautés").size(18.0).strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for c in &s.news {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("•").color(GOLD));
                            ui.label(&c.subject);
                            ui.label(RichText::new(&c.date).weak().small());
                        });
                    }
                });
            }
        });
    }
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../src-tauri/icons/128x128.png")).unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ERtracker Launcher")
            .with_inner_size([520.0, 560.0])
            .with_min_inner_size([420.0, 400.0])
            .with_icon(icon.clone()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("ERtracker Launcher", options, Box::new(move |cc| Ok(Box::new(Launcher::new(cc, &icon)))))
}
