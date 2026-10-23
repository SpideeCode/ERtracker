<script lang="ts">
  // Header search over bosses, graces and items, in French or English.
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { categoryById, pickupCategory, pickupTitle } from "$lib/items";
  import { ui } from "$lib/ui.svelte";

  interface Result {
    id: string;
    title: string;
    subtitle: string;
    kind: string;
    color: string;
    done: boolean;
    rank: number;
  }

  let query = $state("");
  let open = $state(false);
  let selected = $state(0);

  const normalize = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

  const results = $derived.by((): Result[] => {
    const q = normalize(query.trim());
    if (q.length < 2) return [];
    // Prefix matches first, then matches anywhere.
    const rankOf = (...names: (string | null | undefined)[]) => {
      let best = Infinity;
      for (const n of names) {
        if (!n) continue;
        const i = normalize(n).indexOf(q);
        if (i === 0) best = Math.min(best, 0);
        else if (i > 0) best = Math.min(best, 1);
      }
      return best;
    };
    const out: Result[] = [];
    for (const e of db.entries) {
      if (e.category !== "boss" && e.category !== "grace") continue;
      const fr = gamedata.name(e);
      const rank = rankOf(fr, e.name);
      if (rank === Infinity) continue;
      out.push({
        id: e.id,
        title: fr,
        subtitle: fr !== e.name ? e.name : (gamedata.region(e) ?? ""),
        kind: e.category === "boss" ? "Boss" : "Grâce",
        color: e.category === "boss" ? "#e0574a" : "#e8c46a",
        done: tracker.completed.has(e.id),
        rank,
      });
    }
    for (const [id, p] of gamedata.items) {
      const rank = Math.min(...p.items.map((i) => rankOf(i.name.fr, i.name.en)));
      if (rank === Infinity) continue;
      const first = p.items[0];
      const cat = categoryById.get(pickupCategory(p));
      out.push({
        id,
        title: pickupTitle(p),
        subtitle: first.name.fr && first.name.fr !== first.name.en ? first.name.en : (cat?.name ?? ""),
        kind: "Objet",
        color: cat?.color ?? "#ccc",
        done: tracker.completed.has(id),
        rank: rank + 0.5,
      });
    }
    return out.sort((a, b) => a.rank - b.rank || Number(a.done) - Number(b.done) || a.title.localeCompare(b.title)).slice(0, 14);
  });

  function choose(r: Result | undefined) {
    if (!r) return;
    ui.sheet = r.id;
    open = false;
    query = "";
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") selected = Math.min(selected + 1, results.length - 1);
    else if (e.key === "ArrowUp") selected = Math.max(selected - 1, 0);
    else if (e.key === "Enter") choose(results[selected]);
    else if (e.key === "Escape") open = false;
    else return;
    e.preventDefault();
  }
</script>

<div class="search">
  <input
    type="search"
    placeholder="Rechercher un boss, une grâce, un objet… (FR ou EN)"
    bind:value={query}
    onfocus={() => (open = true)}
    oninput={() => {
      open = true;
      selected = 0;
    }}
    onkeydown={onKey}
    onblur={() => setTimeout(() => (open = false), 150)}
  />
  {#if open && results.length}
    <ul class="results">
      {#each results as r, i (r.id)}
        <li>
          <button class:selected={i === selected} onmousedown={() => choose(r)} onmouseenter={() => (selected = i)}>
            <span class="swatch" style="background: {r.color}"></span>
            <span class="text">
              <span class="title" class:done={r.done}>{r.title}{r.done ? " ✓" : ""}</span>
              {#if r.subtitle}<span class="sub">{r.subtitle}</span>{/if}
            </span>
            <span class="kind">{r.kind}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if open && query.trim().length >= 2}
    <div class="results empty">Aucun résultat</div>
  {/if}
</div>

<style>
  .search {
    position: relative;
    flex: 1;
    max-width: 440px;
  }
  input {
    width: 100%;
    box-sizing: border-box;
  }
  .results {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 3000;
    list-style: none;
    margin: 0;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    max-height: 70vh;
    overflow-y: auto;
  }
  .empty {
    padding: 10px 12px;
    color: var(--muted);
  }
  .results button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    background: none;
    border: none;
    color: var(--text);
    padding: 6px 8px;
    border-radius: 6px;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .results button.selected {
    background: var(--surface-2);
  }
  .swatch {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .title.done {
    color: var(--muted);
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .kind {
    font-size: 11px;
    color: var(--muted);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 0 6px;
  }
</style>
