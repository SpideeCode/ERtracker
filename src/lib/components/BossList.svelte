<script lang="ts">
  import { db, tracker } from "$lib/tracker.svelte";
  import { spoiler } from "$lib/spoiler.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { formatDistance, heading } from "$lib/route";
  import { mapdata } from "$lib/mapdata.svelte";
  import { ui } from "$lib/ui.svelte";
  import { locked } from "$lib/accessCheck.svelte";

  let filter = $state<"todo" | "done" | "all">("todo");
  let sort = $state<"near" | "region" | "runes">("near");
  let legendaryOnly = $state(false);
  let search = $state("");

  const me = $derived(tracker.snapshot?.map_position ?? null);

  const rows = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return db.entries
      .filter((e) => e.category === "boss")
      .map((e) => {
        const pos = gamedata.positions.get(e.id);
        const sheet = gamedata.bosses.get(e.id);
        const near = me && pos && pos.map === me.map;
        const distance = near ? Math.hypot(pos.x - me.x, pos.y - me.y) : null;
        const direction = near ? heading(pos.x - me.x, pos.y - me.y) : null;
        return { e, name: gamedata.name(e), region: gamedata.region(e) ?? "", sheet, distance, direction, done: tracker.completed.has(e.id) };
      })
      .filter((r) => (filter === "todo" ? !r.done : filter === "done" ? r.done : true))
      .filter((r) => !legendaryOnly || r.sheet?.legendary)
      .filter((r) => !q || r.name.toLowerCase().includes(q) || r.e.name.toLowerCase().includes(q) || r.region.toLowerCase().includes(q))
      .sort((a, b) => {
        if (sort === "runes") return (b.sheet?.runes ?? 0) - (a.sheet?.runes ?? 0);
        if (sort === "near") return (a.distance ?? Infinity) - (b.distance ?? Infinity) || a.region.localeCompare(b.region);
        return a.region.localeCompare(b.region) || a.name.localeCompare(b.name);
      });
  });

</script>


{#if mapdata.running}
  <p class="notice">Mise à jour des données du jeu ({mapdata.percent} %) : noms en français, positions et fiches arrivent…</p>
{:else if !gamedata.loaded}
  <p class="notice">
    Prépare les cartes (onglet <button class="link" onclick={() => ui.go({ kind: "map" })}>Carte</button>) pour avoir
    les positions, les itinéraires, les noms en français et les défenses de chaque boss.
  </p>
{/if}

<div class="toolbar">
  <input type="search" placeholder="Rechercher un boss ou une région…" bind:value={search} />
  <div class="segmented">
    <button class:active={filter === "todo"} onclick={() => (filter = "todo")}>À vaincre</button>
    <button class:active={filter === "done"} onclick={() => (filter = "done")}>Vaincus</button>
    <button class:active={filter === "all"} onclick={() => (filter = "all")}>Tous</button>
  </div>
  <select bind:value={sort}>
    <option value="near">Plus proches de moi</option>
    <option value="region">Par région</option>
    <option value="runes">Plus de runes</option>
  </select>
  <label class="check"><input type="checkbox" bind:checked={legendaryOnly} /> Légendaires</label>
</div>

<table>
  <tbody>
    {#each rows as r (r.e.id)}
      <tr class:done={r.done} class:selected={ui.sheet === r.e.id} onclick={() => ui.openSheet(r.e.id)}>
        <td class="state"><span class="box" class:checked={r.done}>{r.done ? "✓" : ""}</span></td>
        <td>
          <div class="name">
            {spoiler.name(r.e.id, r.name)}
            {#if r.sheet?.legendary}<span class="legend" title="Boss légendaire">★</span>{/if}
            {#if !r.done && locked(r.e)}<span class="lock" title="Accès à débloquer (voir la fiche)">🔒</span>{/if}
          </div>
          <div class="muted small">{r.region}</div>
        </td>
        <td class="num muted">{r.sheet?.runes ? `${r.sheet.runes.toLocaleString("fr-FR")} runes` : ""}</td>
        <td class="num">
          {#if r.distance !== null}
            {formatDistance(r.distance)}
            <div class="muted small">{r.direction}</div>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td class="muted" colspan="4">Aucun boss avec ces filtres.</td></tr>
    {/each}
  </tbody>
</table>

<style>
  .notice {
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .link {
    background: none;
    border: none;
    color: var(--gold);
    padding: 0;
    font: inherit;
    cursor: pointer;
    text-decoration: underline;
  }
  .toolbar {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    align-items: center;
    margin-bottom: 12px;
    position: sticky;
    top: 0;
    background: var(--bg);
    padding: 8px 0;
    z-index: 1;
  }
  .toolbar input[type="search"] {
    flex: 1;
    min-width: 180px;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
    cursor: pointer;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  tr {
    cursor: pointer;
    border-bottom: 1px solid var(--border);
  }
  tr:hover,
  tr.selected {
    background: var(--surface);
  }
  tr.done .name {
    color: var(--muted);
  }
  td {
    padding: 7px 8px;
    vertical-align: middle;
  }
  .state {
    width: 22px;
  }
  .box {
    width: 16px;
    height: 16px;
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    display: grid;
    place-items: center;
    font-size: 11px;
    color: var(--bg);
  }
  .box.checked {
    background: #5fae6a;
    border-color: #5fae6a;
  }
  tr.selected {
    box-shadow: inset 3px 0 0 var(--gold);
  }
  .num {
    text-align: right;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .small {
    font-size: 12px;
  }
  .lock {
    margin-left: 4px;
    font-size: 12px;
  }
  .legend {
    color: var(--gold);
    margin-left: 4px;
  }
</style>
