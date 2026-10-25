# ERtracker

Tracker de progression Elden Ring (jeu de base + Shadow of the Erdtree) pour PC. Il lit le fichier de sauvegarde (`ER0000.sl2`) et se met à jour à chaque sauvegarde auto du jeu. Il ne lit que le fichier sur le disque : pas de lecture mémoire, pas d'injection. La seule écriture possible est l'outil « Runes » (optionnel, jeu fermé, copie de secours automatique, MD5 du slot recalculé).

Voir [PLAN.md](PLAN.md) pour l'architecture et la roadmap.

## Lancer

Prérequis : Rust (rustup), Node.js 18+, WebView2 (inclus dans Windows 11).

```sh
npm install
npm run tauri dev
```

### Launcher (toujours à jour)

À chaque push sur `main`, GitHub Actions ([release.yml](.github/workflows/release.yml)) compile l'app et le launcher et les publie dans la release `latest`, avec un `manifest.json` (commit, SHA-256 des exe, derniers commits).

Pour installer : télécharger `ertracker-launcher.exe` depuis la [release `latest`](https://github.com/SpideeCode/ERtracker/releases/tag/latest) et le lancer. Au premier lancement il s'installe dans `%LOCALAPPDATA%\ERtracker` et crée les raccourcis Bureau et menu Démarrer. Ensuite, à chaque ouverture, il télécharge la nouvelle version s'il y a eu un commit (et affiche les nouveautés), se met à jour lui-même si `launcher/` a changé, puis le bouton **JOUER** lance ERtracker. Pas besoin de Rust ni de Node.

Le code est dans [launcher/](launcher/) (Rust + egui). Un launcher compilé localement (`cargo build --release`, donc depuis un dossier `target`) ne s'installe pas et ne se remplace pas ; `ERTRACKER_RELEASE_URL` permet de le pointer vers un autre serveur pour tester.

Lire une save sans l'interface :

```sh
cd src-tauri
cargo run --release --bin ertracker-dump            # save détectée automatiquement
cargo run --release --bin ertracker-dump -- C:\chemin\ER0000.sl2
```

## Données

`data/db.json` est généré par `python -I tools/import_esm.py` à partir des fichiers de `third_party/er-save-manager/` (licence MIT, voir `SOURCE.md`).

## Déboguer l'app en cours d'exécution

`npm run dev:debug` lance l'app avec le port DevTools de WebView2 ouvert (9222). Ensuite :

```sh
node tools/cdp.mjs targets                         # fenêtres ouvertes
node tools/cdp.mjs shot main capture.png           # capture de la fenêtre principale (ou "overlay")
node tools/cdp.mjs eval main "document.title"      # exécuter du JS dans la page
node tools/cdp.mjs console overlay 5               # messages et erreurs de la console
```
