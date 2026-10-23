"""Build data/db.json from the MIT-licensed er-save-manager data in third_party/.

The source files are Python modules made of literals. They are parsed with
`ast.literal_eval` and never imported or executed.

Usage: python -I tools/import_esm.py
"""

import ast
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "third_party" / "er-save-manager"
OUT = ROOT / "data" / "db.json"

# er-save-manager category -> (our category id, keep entries whose name matches)
FLAG_CATEGORIES = {
    "Grace": "grace",
    "Cookbook": "cookbook",
    "Bell Bearings": "bell_bearing",
    "Maps": "map_reveal",
    "Whetblades": "whetblade",
}

CATEGORIES = [
    {"id": "boss", "name": {"fr": "Boss", "en": "Bosses"}},
    {"id": "grace", "name": {"fr": "Sites de grâce", "en": "Sites of Grace"}},
    {"id": "great_rune", "name": {"fr": "Grandes runes", "en": "Great Runes"}},
    {"id": "cookbook", "name": {"fr": "Livres de recettes", "en": "Cookbooks"}},
    {"id": "bell_bearing", "name": {"fr": "Cloches de marchand", "en": "Bell Bearings"}},
    {"id": "whetblade", "name": {"fr": "Lames à aiguiser", "en": "Whetblades"}},
    {"id": "map_reveal", "name": {"fr": "Cartes révélées", "en": "Map Reveals"}},
]


def load_literals(path: Path, names: set[str]) -> dict:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    out = {}
    for node in tree.body:
        target = None
        if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name):
            target = node.targets[0].id
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            target = node.target.id
        if target in names and node.value is not None:
            out[target] = ast.literal_eval(node.value)
    return out


def clean_name(name: str) -> tuple[str, str | None]:
    """'[Limgrave] Church of Elleh | Unlocked' -> ('Church of Elleh', 'Limgrave')"""
    region = None
    m = re.match(r"^\[([^\]]+)\]\s*(.*)$", name)
    if m:
        region, name = m.group(1), m.group(2)
    # Drop status suffixes such as "| Unlocked" or "| NG+ Persistent".
    name = name.split(" | ")[0]
    return name.strip(), region


def slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", text.lower()).strip("_")


def main() -> None:
    bosses = load_literals(SRC / "boss_data.py", {"BOSSES"})["BOSSES"]
    flags_db = load_literals(SRC / "event_flags_db.py", {"EVENT_FLAGS", "FLAGS_BY_CATEGORY"})
    quests = load_literals(SRC / "quest_flags_db.py", {"QUEST_FLAGS"})["QUEST_FLAGS"]
    names = flags_db["EVENT_FLAGS"]
    by_cat = flags_db["FLAGS_BY_CATEGORY"]

    entries = []
    seen = set()

    def add(category: str, name: str, flag: int, region: str | None) -> None:
        eid = f"{category}:{flag}"
        if eid in seen:
            return
        seen.add(eid)
        entries.append({"id": eid, "category": category, "name": name, "region": region, "flag": flag})

    # The first flag of each boss is its defeat flag; the others are arena/warp side effects.
    for boss_name, info in bosses.items():
        add("boss", boss_name, info["flags"][0], info.get("category"))

    for src_cat, cat in FLAG_CATEGORIES.items():
        for group_region, ids in by_cat[src_cat].items():
            for flag in ids:
                raw = names.get(flag, {}).get("name")
                if not raw or raw == "Crafting Unlocked":
                    continue
                name, region = clean_name(raw)
                add(cat, name, flag, region or group_region)

    for flag in by_cat["System"]["Great Runes"]:
        raw = names[flag]["name"]
        if raw.startswith("Acquired"):
            add("great_rune", raw.removeprefix("Acquired ").strip(), flag, None)

    quest_list = [
        {
            "id": f"quest:{slug(npc)}",
            "npc": npc,
            "steps": [
                {
                    "description": step["description"],
                    "location": step.get("location"),
                    "flags": [[f["id"], f["value"]] for f in step["flags"]],
                }
                for step in steps
            ],
        }
        for npc, steps in quests.items()
    ]

    db = {"version": 1, "categories": CATEGORIES, "entries": entries, "quests": quest_list}
    OUT.write_text(json.dumps(db, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")

    counts = {}
    for e in entries:
        counts[e["category"]] = counts.get(e["category"], 0) + 1
    print(f"wrote {OUT.relative_to(ROOT)}: {counts}, quests={len(quest_list)}")


if __name__ == "__main__":
    main()
