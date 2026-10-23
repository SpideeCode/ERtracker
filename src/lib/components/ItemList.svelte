<script lang="ts">
  import { tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { formatDistance, heading } from "$lib/route";
  import { ITEM_CATEGORIES, categoryById, matches, pickupCategory, pickupTitle } from "$lib/items";
  import { ui } from "$lib/ui.svelte";
  import { catalog } from "$lib/catalog.svelte";
  import { estimate, rarityOf } from "$lib/equipment";
  import type { ItemPickup } from "$lib/types";

  let filter = $state<"todo" | "done" | "all">("todo");
  let category = $state<string>("important");
  let search = $state("");
  let limit = $state(150);
  let sort = $state<"near" | "rarity" | "power">("near");

  const rarity = (p: ItemPickup) => Math.max(...p.items.map((i) => catalog.rarity(i.cat, i.id)));
  /** Comparable strength: estimated damage for weapons, absorption for armour, rarity otherwise. */
  function power(p: ItemPickup): number {
    return Math.max(
      ...p.items.map((i) => {
        if (i.cat === 2) {
          const w = catalog.weapons.get(Math.floor(i.id / 10000) * 10000);
          return w ? estimate(w, tracker.character?.stats).total : 0;
        }
        if (i.cat === 3) {
          const a = catalog.armor.get(i.id);
          return a ? a.absorption.reduce((x, y) => x + y, 0) : 0;
        }
        return catalog.rarity(i.cat, i.id) * 10;
      }),
    );
  }

  const me = $derived(tracker.snapshot?.map_position ?? null);

  const all = $derived(
    [...gamedata.items].map(([id, p]) => ({ id, p, category: pickupCategory(p), title: pickupTitle(p), done: tracker.completed.has(id) })),
  );

  const counts = $derived(
    Object.fromEntries(
      ITEM_CATEGORIES.map((c) => {
        const list = all.filter((r) => r.category === c.id);
        return [c.id, { done: list.filter((r) => r.done).length, total: list.length }];
      }),
    ),
  );

  const rows = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return all
      .filter((r) => (filter === "todo" ? !r.done : filter === "done" ? r.done : true))
      .filter((r) =>
        q
          ? matches(r.p, q)
          : category === "important"
            ? categoryById.get(r.category)?.important
            : category === "all" || r.category === category,
      )
      .map((r) => {
        const near = me && r.p.map === me.map;
        return {
          ...r,
          distance: near ? Math.hypot(r.p.x - me.x, r.p.y - me.y) : null,
          direction: near ? heading(r.p.x - me.x, r.p.y - me.y) : null,
        };
      })
      .map((r) => ({ ...r, rarity: rarity(r.p) }))
      .sort((a, b) => {
        if (sort === "rarity") return b.rarity - a.rarity || (a.distance ?? Infinity) - (b.distance ?? Infinity);
        if (sort === "power") return power(b.p) - power(a.p);
        return (a.distance ?? Infinity) - (b.distance ?? Infinity) || a.title.localeCompare(b.title);
      });
  });
  const done = $derived(all.filter((r) => r.done).length);
</script>

<header>
  <h2>Objets</h2>
  <span class="muted">{done} / {all.length} ramassés</span>
</header>

{#if mapdata.running}
  <p class="notice">Mise à jour des données du jeu ({mapdata.percent} %) : la liste des objets arrive…</p>
{:else if !gamedata.items.size}
  <p class="notice">
    Prépare les cartes (onglet <button class="link" onclick={() => (ui.view = { kind: "map" })}>Carte</button>) : les objets
    sont lus dans ton installation du jeu.
  </p>
{/if}

<div class="toolbar">
  <input type="search" placeholder="Nom d'objet en français ou en anglais…" bind:value={search} oninput={() => (limit = 150)} />
  <div class="segmented">
    <button class:active={filter === "todo"} onclick={() => (filter = "todo")}>À récupérer</button>
    <button class:active={filter === "done"} onclick={() => (filter = "done")}>Récupérés</button>
    <button class:active={filter === "all"} onclick={() => (filter = "all")}>Tous</button>
  </div>
  <select bind:value={sort} title="Trier">
    <option value="near">Plus proches de moi</option>
    <option value="rarity">Plus rares</option>
    <option value="power">Plus puissants</option>
  </select>
</div>

<div class="chips">
  <button class="chip" class:active={category === "important"} onclick={() => (category = "important")}>Essentiels</button>
  {#each ITEM_CATEGORIES as c (c.id)}
    <button class="chip" class:active={category === c.id} onclick={() => (category = c.id)}>
      <span class="swatch" style="background: {c.color}"></span>
      {c.name}
      <span class="muted">{counts[c.id]?.done}/{counts[c.id]?.total}</span>
    </button>
  {/each}
  <button class="chip" class:active={category === "all"} onclick={() => (category = "all")}>Tout</button>
</div>
{#if search.trim()}
  <p class="muted small">Recherche dans toutes les catégories.</p>
{/if}

<table>
  <tbody>
    {#each rows.slice(0, limit) as r (r.id)}
      <tr class:done={r.done} class:selected={ui.sheet === r.id} onclick={() => (ui.sheet = r.id)}>
        <td class="state"><span class="box" class:checked={r.done}>{r.done ? "✓" : ""}</span></td>
        <td>
          <div class="name">
            <span class="swatch" style="background: {categoryById.get(r.category)?.color}"></span>
            <span style="color: {r.rarity > 0 ? rarityOf(r.rarity).color : 'inherit'}">{r.title}</span>
            {#if r.p.items.some((i) => catalog.isOwned(i.cat, i.cat === 2 ? Math.floor(i.id / 10000) * 10000 : i.id))}
              <span class="owned" title="Déjà dans ton inventaire">possédé</span>
            {/if}
          </div>
          {#if r.p.items[0].name.fr && r.p.items[0].name.fr !== r.p.items[0].name.en}
            <div class="muted small">{r.p.items[0].name.en}</div>
          {/if}
        </td>
        <td class="num">
          {#if r.distance !== null}
            {formatDistance(r.distance)}
            <div class="muted small">{r.direction}</div>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td class="muted" colspan="3">Aucun objet avec ces filtres.</td></tr>
    {/each}
  </tbody>
</table>
{#if rows.length > limit}
  <button class="more" onclick={() => (limit += 300)}>Afficher plus ({rows.length - limit} restants)</button>
{/if}

<style>
  header {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  h2 {
    margin: 0 0 12px;
  }
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
    margin-bottom: 10px;
  }
  .toolbar input {
    flex: 1;
    min-width: 200px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 12px;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    border-radius: 14px;
    padding: 3px 10px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .chip.active {
    border-color: var(--gold);
    color: var(--gold);
  }
  .swatch {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
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
  tr.selected {
    box-shadow: inset 3px 0 0 var(--gold);
  }
  tr.done .name {
    color: var(--muted);
  }
  td {
    padding: 7px 8px;
    vertical-align: middle;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .state {
    width: 22px;
  }
  .owned {
    font-size: 11px;
    color: #5fae6a;
    border: 1px solid #3f6b42;
    border-radius: 8px;
    padding: 0 6px;
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
  .num {
    text-align: right;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .small {
    font-size: 12px;
  }
  .more {
    margin: 12px auto;
    display: block;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 7px 14px;
    font: inherit;
    cursor: pointer;
  }
</style>
