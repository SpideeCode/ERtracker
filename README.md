# ERtracker

Tracker de progression Elden Ring (jeu de base + Shadow of the Erdtree) pour PC. Il lit le fichier de sauvegarde (`ER0000.sl2`) et se met à jour à chaque sauvegarde auto du jeu. Il ne lit que le fichier sur le disque : pas de lecture mémoire, pas d'injection, rien n'est écrit.

Voir [PLAN.md](PLAN.md) pour l'architecture et la roadmap.

## Lancer

Prérequis : Rust (rustup), Node.js 18+, WebView2 (inclus dans Windows 11).

```sh
npm install
npm run tauri dev
```

Lire une save sans l'interface :

```sh
cd src-tauri
cargo run --release --bin ertracker-dump            # save détectée automatiquement
cargo run --release --bin ertracker-dump -- C:\chemin\ER0000.sl2
```

## Données

`data/db.json` est généré par `python -I tools/import_esm.py` à partir des fichiers de `third_party/er-save-manager/` (licence MIT, voir `SOURCE.md`).
