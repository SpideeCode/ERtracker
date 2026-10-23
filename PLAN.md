# ERtracker — Plan complet

Tracker de progression Elden Ring (jeu de base + Shadow of the Erdtree) qui lit le fichier de sauvegarde en direct et affiche, sur les cartes du jeu, tout ce qui a été fait / pas fait : boss, objets, sites de grâce, quêtes PNJ, fragments de carte, etc.

---

## 1. Principe de fonctionnement

```
 Elden Ring ──(autosave)──► %APPDATA%\EldenRing\<SteamID>\ER0000.sl2
                                         │  événement fichier (ReadDirectoryChangesW)
                                         ▼
                      ┌────────── Backend Rust ──────────┐
                      │ 1. debounce 300 ms               │
                      │ 2. copie en mémoire + MD5 check  │
                      │ 3. parse BND4 → slot actif       │
                      │ 4. extrait event flags,          │
                      │    inventaire, position, stats   │
                      │ 5. diff avec l'état précédent    │
                      └───────────────┬──────────────────┘
                                      │ événement IPC (uniquement le diff)
                                      ▼
                      ┌──────── Frontend (WebView) ───────┐
                      │ Cartes Leaflet + listes + stats   │
                      │ Toast « Boss vaincu : Malenia »   │
                      └───────────────────────────────────┘
```

**Clé du système : les event flags.** Le jeu stocke presque toute la progression sous forme de bits ("event flags") dans le slot du personnage :
- chaque **objet ramassé au sol** possède un flag d'acquisition (`getItemFlagId` dans `ItemLotParam_map`) ;
- chaque **boss vaincu** a son flag (ex. Godrick, Margit, Malenia…) ;
- chaque **site de grâce découvert**, **fragment de carte**, **étape de quête PNJ**, **porte ouverte**, etc. a le sien.

Il suffit donc de lire le bitfield et de le croiser avec une base de données « flag → objet/boss + position sur la carte ».

**Aucun risque EAC/ban** : on lit uniquement le fichier de sauvegarde sur le disque, sans lecture mémoire du processus, sans injection et sans écriture.

---

## 2. Stack choisie

| Couche | Choix | Pourquoi |
|---|---|---|
| Shell d'app | **Tauri 2** | Binaire d'environ 8 à 12 Mo. Utilise WebView2, déjà présent sur Windows 11. Environ 60 à 90 Mo de RAM contre plus de 300 Mo pour Electron. |
| Backend | **Rust** | Parsing binaire rapide (< 10 ms pour un slot), crate `notify` pour surveiller le fichier sans polling, aucun GC. |
| Frontend | **Svelte 5 + TypeScript + Vite** | Pas de virtual DOM, bundle minuscule, réactivité fine (seuls les marqueurs modifiés sont re-rendus). |
| Carte | **Leaflet** (`CRS.Simple`) + tuiles WebP | Léger (40 Ko), gère très bien des milliers de marqueurs avec clustering ou rendu canvas. |
| Données | **JSON compressé embarqué** (ou SQLite via `rusqlite` si ça grossit) | Chargé une seule fois au démarrage. |
| Pipeline de données | **Python** (scripts hors app, lancés une fois) | Extraction des params et MSB du jeu, puis génération du JSON. |

**Budget de perf visé** : 0 % de CPU au repos (aucun polling, réveil uniquement sur écriture du fichier), traitement d'une autosave en moins de 20 ms, environ 80 Mo de RAM, aucun impact sur les FPS du jeu.

---

## 3. Format de la sauvegarde (vérifié sur ta save)

```
ER0000.sl2  (BND4, ~28,9 Mo, NON chiffré sur PC)
├─ USER_DATA000..009   10 slots personnage, 0x280010 octets chacun
│    ├─ 0x00  MD5 (16 o) du bloc de données
│    └─ 0x10  données du slot (0x280000)
│          ├─ version, map_id + coordonnées du joueur (dernière position)
│          ├─ GaItem map (handles d'objets)
│          ├─ PlayerGameData (nom, niveau, stats, runes)
│          ├─ inventaire + coffre (équipement, consommables, objets clés)
│          ├─ … (sections de taille variable → parsing séquentiel)
│          └─ EventFlags bitfield (~0x1BF99F octets)
├─ USER_DATA010   ProfileSummary : slots actifs, nom, niveau, temps de jeu
└─ USER_DATA011   regulation / divers
```

**Conversion flag → bit** : les IDs de flags ne sont pas linéaires. On calcule `bloc = id / 1000` et `reste = id % 1000`, puis on cherche l'offset du bloc dans une table BST (`eventflag_bst`, connue de la communauté, voir ER-Save-Editor de ClayAmore). On obtient `octet = offset*125 + reste/8`, puis `bit = 7 - reste%8`.

**Robustesse** :
- le jeu écrit le fichier en plusieurs passes, donc on attend un debounce puis on vérifie le MD5 du slot ; si ça ne correspond pas, on relit 200 ms plus tard ;
- la lecture se fait avec `FILE_SHARE_READ|WRITE` pour ne jamais bloquer le jeu ;
- prise en charge de `.co2` (Seamless Coop) et de plusieurs SteamID / dossiers, avec choix du chemin dans les réglages ;
- détection auto du slot actif (le plus récemment modifié via le diff, ou choix manuel).

### Détection automatique des chemins (`src-tauri/src/paths.rs`)

Au premier lancement, l'app trouve le jeu et la save toute seule. Le chemin trouvé est mémorisé dans `config.json` et reste modifiable dans les Réglages avec un sélecteur de dossier.

**Dossier du jeu.** L'app essaie ces emplacements dans l'ordre et garde le premier où `Game\eldenring.exe` existe :
1. le chemin déjà enregistré dans la config ;
2. **Steam (emplacement par défaut)** : `SteamPath` lu dans le registre (`HKCU\Software\Valve\Steam`, sinon `HKLM\SOFTWARE\WOW6432Node\Valve\Steam`, valeur `InstallPath`), puis chaque bibliothèque listée dans `steamapps\libraryfolders.vdf` → `<lib>\steamapps\common\ELDEN RING`. On peut aussi viser directement l'app id 1245620 via `appmanifest_1245620.acf` ;
3. les emplacements courants : `C:\Program Files (x86)\Steam\steamapps\common\ELDEN RING`, `C:\Program Files\Steam\…`, et `<X>:\SteamLibrary\steamapps\common\ELDEN RING` sur chaque disque ;
4. les installations hors Steam : `<X>:\Games\ELDEN RING` et `<X>:\ELDEN RING` sur chaque disque (c'est ce qui couvre `C:\Games\ELDEN RING`) ;
5. si rien n'est trouvé : écran d'accueil qui demande « Sélectionne ton dossier ELDEN RING », avec validation de la présence de `eldenring.exe`.

Le dossier du jeu ne sert **qu'une seule fois**, pour l'extraction des cartes et des données. Le suivi en direct n'a besoin que de la save.

**Fichier de sauvegarde.**
1. `%APPDATA%\EldenRing\*\` : on liste les sous-dossiers SteamID qui contiennent `ER0000.sl2` (ou `.co2` pour Seamless Coop) ;
2. s'il y en a plusieurs, on présélectionne celui modifié le plus récemment et l'utilisateur peut changer via un menu déroulant (avec les noms des persos lus dans USER_DATA010) ;
3. les dossiers dont le nom n'est pas un SteamID valide (ex. « … - Copie ») sont ignorés par défaut ;
4. sinon, sélection manuelle du fichier.

---

## 4. Base de données de progression

### 4.1 Catégories suivies

| Catégorie | Source du statut |
|---|---|
| Boss (majeurs, mineurs, de terrain, DLC) | event flag de défaite |
| Sites de grâce | event flag de découverte |
| Objets au sol / dans les coffres / sur des cadavres | flag d'acquisition de l'item lot |
| Drops de boss / d'ennemis uniques | flag de l'item lot du boss |
| Armes, armures, talismans, sorts, incantations, cendres de guerre, cendres d'esprit | flag d'acquisition **et** présence dans l'inventaire (pour les objets achetés) |
| Graines d'or, larmes sacrées, perles de Larval, fragments d'Arbre-Ombre, cendres vénérées | flag d'acquisition |
| Livres de recettes, cloches de marchands, clés de pierre tombale… | flag d'acquisition |
| Fragments de carte | flag |
| Quêtes PNJ (étapes) | séquence de flags par PNJ |
| Grandes runes, souvenirs | flag et inventaire |
| Fins / NG+ | flags et compteur NG dans le slot |

### 4.2 Schéma (simplifié)

```jsonc
{
  "id": "boss_malenia",
  "category": "boss",            // boss | grace | item | npc_quest | map_fragment ...
  "subcategory": "major",
  "name": { "fr": "Malenia, Lame de Miquella", "en": "Malenia, Blade of Miquella" },
  "flag": 9120,                  // ou "flags": [...] avec logique all/any
  "map": "surface",              // surface | underground | dlc
  "region": "Haligtree",
  "pos": [x, y],                 // coordonnées dans l'espace de la carte (pixels de la tuile max)
  "item_ids": [ ... ],           // pour le check inventaire
  "notes": "Derrière la grâce Haligtree Roots",
  "requires": ["boss_loretta_haligtree"]
}
```

### 4.3 Comment obtenir les données (étape la plus lourde)

**Méthode A (principale) : extraction depuis TES fichiers de jeu**, donc exacte et à jour avec les patchs.
1. Dépaqueter le jeu avec **Smithbox** ou **UXM Selective Unpacker**, puis WitchyBND.
2. Lire `regulation.bin` : `ItemLotParam_map`, `ItemLotParam_enemy`, `EquipParam*`, `BonfireWarpParam` (grâces avec leurs positions), `WorldMapPointParam`, `WorldMapLegacyConvParam`.
3. Lire les **MSB** (`map/mapstudio/*.msb.dcx`) : entités « Treasure » (objet → item lot → position monde) et ennemis boss.
4. Lire les FMG (textes) pour avoir les noms FR et EN.
5. **Conversion des coordonnées monde vers la carte** : les tuiles `m60_XX_YY` (surface), `m61_XX_YY` (DLC) et `m12_*` (souterrain) donnent les coordonnées via `XX*256 + local`. Les donjons legacy (`m10`, `m11`, `m13`…) passent par `WorldMapLegacyConvParam`.
6. Le script Python génère `data/progress.json`.

**Méthode B (amorçage et validation)** : croiser avec des données communautaires open source (randomizer de thefifthmatt pour les lieux des objets, listes de flags de boss d'ER-Save-Editor et des wikis de modding), puis corriger à la main les cas particuliers.

> Les assets du jeu (cartes, textes) **ne sont pas redistribués** dans le dépôt : ils sont extraits localement depuis ton installation.

---

## 5. Les cartes

| Calque | Contenu | Source |
|---|---|---|
| **Lands Between** (surface) | Limgrave → Haligtree, avec les donjons legacy projetés | `menu/71_maptile.tpfbhd` (MENU_MapTile, niveau L0) |
| **Souterrain** | Siofra, Ainsel, Deeproot, lac de pourriture, Mohgwyn, Nokron… | même archive, carte souterraine |
| **Royaume de l'Ombre** (DLC) | Gravesite Plain → Enir-Ilim | même archive, carte DLC |

Pipeline des tuiles : extraction des DDS, conversion en PNG, assemblage, puis découpe en pyramide de tuiles WebP 256 px (`tiles/{map}/{z}/{x}/{y}.webp`, environ 50 à 120 Mo au total). Script `tools/build_tiles.py` avec Pillow.

**Fonctions de carte** :
- marqueurs colorés : ✅ fait (estompé ou masqué), ⭕ pas fait, 🔒 inaccessible pour l'instant (prérequis) ;
- filtres par catégorie et sous-catégorie, avec option « masquer les complétés » ;
- recherche (« Lame de Miquella », « larme sacrée »…) qui centre la carte sur le résultat ;
- **position du joueur** à la dernière sauvegarde (lue dans le slot) avec le calque correspondant ;
- popup sur chaque marqueur : nom, icône, description, prérequis, statut ;
- clustering à faible zoom ; rendu en `L.canvas` pour des milliers de points sans lag ;
- marquage manuel possible (pour les rares éléments sans flag fiable).

---

## 6. Interface

```
┌────────────────────────────────────────────────────────────────────────┐
│ ERtracker   [Perso: Tarnished Lv.120 ▾]   ● LIVE (save il y a 12 s)    │
├──────────┬─────────────────────────────────────────────────────────────┤
│ Dashboard│   [Surface] [Souterrain] [DLC]                    🔍        │
│ Carte    │                                                             │
│ Boss     │              (carte Leaflet + marqueurs)                    │
│ Objets   │                                                             │
│ Grâces   │                                                ┌──────────┐ │
│ Quêtes   │                                                │ Filtres  │ │
│ Réglages │                                                └──────────┘ │
├──────────┴─────────────────────────────────────────────────────────────┤
│ 🔔 Boss vaincu : Rennala, reine de la pleine lune                       │
└────────────────────────────────────────────────────────────────────────┘
```

- **Dashboard** : % global, % par catégorie, % par région (barres), dernières actions (journal du diff), temps de jeu, niveau, runes.
- **Listes** : tableaux filtrables et triables, avec un clic pour localiser l'élément sur la carte.
- **Quêtes PNJ** : timeline des étapes par PNJ, avec la prochaine étape et les avertissements (« ne pas tuer X avant Y »).
- **Mode overlay** (optionnel) : petite fenêtre borderless toujours au premier plan avec les compteurs, pour un 2e écran ou en jeu fenêtré.
- **Notifications** : toast in-app et notification Windows facultative lors d'un nouveau boss ou objet rare.
- Thème sombre façon UI d'Elden Ring, interface en FR et EN.

---

## 7. Architecture du code

```
ERtracker/
├─ PLAN.md
├─ src-tauri/                     # Backend Rust
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  └─ src/
│     ├─ main.rs                  # bootstrap Tauri, commandes IPC
│     ├─ save/
│     │  ├─ bnd4.rs               # parse du conteneur
│     │  ├─ slot.rs               # parse séquentiel du slot (inventaire, position, stats)
│     │  ├─ event_flags.rs        # BST + lecture des bits
│     │  └─ profile.rs            # USER_DATA010 (liste des persos)
│     ├─ watcher.rs               # notify + debounce + retry MD5
│     ├─ progress.rs              # croise flags/inventaire avec la DB → statuts
│     ├─ diff.rs                  # calcule les changements entre deux saves
│     ├─ paths.rs                 # détection auto du jeu (Steam/registre/disques) et des saves
│     └─ config.rs                # chemins, slot choisi, préférences
├─ src/                           # Frontend Svelte
│  ├─ lib/map/                    # Leaflet, calques, marqueurs, filtres
│  ├─ lib/views/                  # Dashboard, Boss, Objets, Grâces, Quêtes
│  ├─ lib/stores/progress.ts      # état réactif alimenté par les events Tauri
│  └─ App.svelte
├─ data/
│  ├─ eventflag_bst.bin           # table de conversion des flags
│  ├─ progress.json               # base générée (flags → éléments + positions)
│  └─ overrides.json              # corrections manuelles
├─ tools/                         # Pipeline Python (lancé une seule fois)
│  ├─ extract_params.py
│  ├─ extract_msb.py
│  ├─ build_db.py
│  └─ build_tiles.py
└─ tests/
   └─ fixtures/                   # copies de saves pour les tests de parsing
```

**IPC Tauri** :
- `get_characters()`, `select_character(slot)`, `get_full_state()` au démarrage ;
- event `progress://diff` → `{ completed: [ids], uncompleted: [ids], player_pos, stats }`.

---

## 8. Roadmap

| Phase | Contenu | Livrable |
|---|---|---|
| **0. Setup** | Installer Rust (rustup) et le CLI Tauri, scaffold Tauri + Svelte | App vide qui démarre |
| **1. Parser de save** | BND4, ProfileSummary (persos), MD5, event flags + BST, tests sur ta save | CLI `ertracker-dump` qui affiche les persos et teste des flags connus (Margit, Godrick) |
| **2. Watcher live** | `notify`, debounce, retry, diff, events IPC | Le log s'actualise à chaque autosave en jeu |
| **3. Données v1** | Boss et grâces (≈ 200 + 400 entrées), positions | Liste Boss/Grâces fonctionnelle |
| **4. Cartes** | Extraction des tuiles des 3 cartes, Leaflet, calibrage des coordonnées | Carte interactive avec boss et grâces |
| **5. Données v2** | Tous les objets (item lots via MSB), fragments, objets clés, DLC | Tracker objets complet |
| **6. Parsing slot avancé** | Inventaire, position joueur, stats, NG+ | « Vous êtes ici » et statut des objets achetés |
| **7. Quêtes PNJ** | Séquences de flags par PNJ, UI timeline | Onglet Quêtes |
| **8. Polish** | Dashboard, overlay, notifications, i18n, thème, installeur MSI/NSIS | v1.0 |

**Ordre critique** : les phases 1 et 2 prouvent que le live fonctionne ; la phase 3 et ses données sont ce qui demande le plus de travail. On commence avec boss et grâces parce qu'ils ont peu d'entrées et des flags bien documentés, puis on étend.

---

## 9. Risques et solutions

| Risque | Solution |
|---|---|
| Lecture pendant l'écriture du jeu | Debounce + vérification MD5 + retry |
| Sections du slot de taille variable (offsets qui bougent selon la version du jeu) | Parsing séquentiel comme ER-Save-Editor, sans offsets fixes ; tests sur plusieurs saves |
| Conversion des coordonnées des donjons legacy et du souterrain | `WorldMapLegacyConvParam` + calibrage manuel avec quelques points de référence (grâces) |
| Volume de données (plus de 3 000 objets) | Génération automatique depuis les params et MSB, pas de saisie à la main |
| Items sans flag (achats, drops aléatoires) | Vérification de l'inventaire ou case manuelle |
| Patch du jeu qui change le format | Version du slot détectée, avec message clair si non supportée |
| Droits sur les assets | Extraction locale depuis l'install de l'utilisateur, rien n'est redistribué |

---

## 10. Trackers existants (analysés le 2026-10-23)

| Projet | Ce qu'il fait | Réutilisable ? |
|---|---|---|
| [eldenring.krut.top](https://eldenring.krut.top) (Elden Ring Tracker 2.0) | Site web : on dépose la save, carte base/DLC, boss légendaires ou ordinaires, grâces, fragments de carte, fragments d'Arbre-Ombre et cendres vénérées (niveau de bénédiction), scarabées, marchands, ordre de progression, filtre « manquants seulement », 15 langues | Non : code fermé, compte obligatoire, offre Pro. **Idées seulement.** |
| [EldenRingMap](https://github.com/egormagurin/EldenRingMap) | Serveur Python local, save en direct, 3 506 marqueurs, carte extraite du jeu, 4 calques, altitude des marqueurs | Non : aucune licence. **Idées seulement.** |
| [Elden Ring Compass](https://github.com/EthanShoeDev/elden-ring-compass) | Web, lecture de la save dans le navigateur | Non : aucune licence (le fork « plus » se dit MIT sans en avoir le droit) |
| [er-save-manager](https://github.com/Hapfel1/er-save-manager) @ `bde1359` | Éditeur de save : table BST, boss, grâces, quêtes | **Oui (MIT)**, déjà intégré dans `third_party/` |
| [ER-Save-Editor](https://github.com/ClayAmore/ER-Save-Editor) | Éditeur de save en Rust | **Oui (MIT/Apache)**, structure du slot reprise |

**Ce qu'on reprend comme idées (à implémenter nous-mêmes)** :
- un sélecteur **Jeu de base / DLC** au-dessus de la carte ;
- des **calques activables un par un** (grâces, boss légendaires, boss, fragments d'Arbre-Ombre, cendres vénérées, sources spirituelles, objets, scarabées, marchands, lieux), avec un bouton **« manquants seulement »** et un compteur « repères : X sur Y affichés » ;
- les **boss légendaires** (souvenirs) séparés des boss ordinaires ;
- un **niveau de bénédiction** (Arbre-Ombre /20, cendres vénérées /10) avec ce qui reste à trouver ;
- un **ordre de progression conseillé** ;
- un mode **« save de quelqu'un d'autre »** en lecture seule ;
- des **badges** ou succès.

**Ce qui nous distingue** : une app native en direct (surveillance du fichier, rien à déposer), sans compte, hors ligne, tous les objets ramassables (pas seulement boss et grâces), les quêtes PNJ, et des cartes extraites de l'install du joueur.

---

## 11. Prérequis côté machine

- **Rust** (absent actuellement, à installer via rustup) et **Node.js** (présent) ;
- WebView2 (déjà intégré à Windows 11) ;
- Python 3.12 (présent) pour le pipeline de données, avec Pillow ;
- Accès aux fichiers du jeu installé, une seule fois, pour l'extraction des cartes et des données. Sur la machine de dev, le jeu est dans `C:\Games\ELDEN RING` (v1.17.0, archives `Data0-3.bdt` + `DLC.bdt` encore empaquetées : il faut les dépaqueter avec Smithbox ou UXM, ou implémenter le lecteur BHD5 + Oodle via `oo2core_6_win64.dll`, déjà présente dans le dossier du jeu).
