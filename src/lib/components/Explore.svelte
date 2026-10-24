<script lang="ts">
  // "À explorer": zones with things left, without spoilers unless asked.
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { catalog } from "$lib/catalog.svelte";
  import { categoryById, pickupCategory, pickupTitle } from "$lib/items";
  import { locked } from "$lib/accessCheck.svelte";
  import { buildZones, progress, zoneName, type Zone } from "$lib/zones";
  import { formatDistance } from "$lib/route";
  import { ui } from "$lib/ui.svelte";

  const MAPS: Record<string, string> = { M00: "Entre-terre", M01: "Souterrain", M10: "Royaume de l'Ombre", M11: "Souterrain (DLC)" };

  let filter = $state<"suggest" | "started" | "all">("suggest");
  let revealed = $state<Set<string>>(new Set());

  const zones = $derived(
    buildZones({
      bosses: db.entries.filter((e) => e.category === "boss"),
      graces: db.entries.filter((e) => e.category === "grace"),
      items: [...gamedata.items],
      positions: gamedata.positions,
      completed: tracker.completed,
      important: (p) => !!categoryById.get(pickupCategory(p))?.important,
      rarity: (p) => Math.max(...p.items.map((i) => catalog.rarity(i.cat, i.id))),
    }),
  );

  const me = $derived(tracker.snapshot?.map_position ?? null);
  const regionName = (en: string) => gamedata.regions[en] ?? en;
  const lockedZone = (z: Zone) => {
    const boss = db.entries.find((e) => e.category === "boss" && e.region === z.region);
    return boss ? locked(boss) && !z.visited : false;
  };

  const shown = $derived.by(() => {
    const list = zones
      .filter((z) => z.score > 0)
      .map((z) => ({ z, distance: me && me.map === z.map ? Math.hypot(z.x - me.x, z.y - me.y) : null, locked: lockedZone(z) }));
    if (filter === "started") return list.filter((r) => r.z.visited).sort((a, b) => b.z.score - a.z.score);
    if (filter === "all") return list.sort((a, b) => b.z.score - a.z.score);
    // Suggestions: zones you can reach, where you already set foot or that are close, with the most left.
    return list
      .filter((r) => !r.locked)
      .sort((a, b) => rank(b) - rank(a))
      .slice(0, 12);
  });

  function rank(r: { z: Zone; distance: number | null }) {
    const near = r.distance === null ? 0.6 : Math.max(0.3, 1 - r.distance / 4000);
    const started = r.z.visited ? 1.3 : 1;
    return r.z.score * near * started;
  }

  function show(z: Zone) {
    ui.showPlace({ map: z.map, x: z.x, y: z.y, label: `${zoneName(z, regionName(z.region))}`, radius: z.radius });
  }

  function toggle(key: string) {
    const next = new Set(revealed);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    revealed = next;
  }

  const label = (id: string) => {
    const e = db.entries.find((x) => x.id === id);
    if (e) return gamedata.name(e);
    const p = gamedata.items.get(id);
    return p ? pickupTitle(p) : id;
  };
</script>

<h2>À explorer</h2>
<p class="muted">
  Les zones où il te reste le plus à découvrir, sans rien dévoiler : seulement combien de boss, de grâces et d'objets
  notables restent à trouver.
</p>

{#if !gamedata.loaded}
  <p class="notice">Prépare les cartes (onglet Carte) pour activer les suggestions.</p>
{/if}

<div class="segmented filters">
  <button class:active={filter === "suggest"} onclick={() => (filter = "suggest")}>Suggestions</button>
  <button class:active={filter === "started"} onclick={() => (filter = "started")}>Zones commencées</button>
  <button class:active={filter === "all"} onclick={() => (filter = "all")}>Toutes</button>
</div>

<div class="zones">
  {#each shown as { z, distance, locked: isLocked } (z.key)}
    {@const name = zoneName(z, regionName(z.region))}
    <article class="zone" class:locked={isLocked}>
      <header>
        <div>
          <h3>{name}</h3>
          <div class="muted small">
            {MAPS[z.map]}{distance !== null ? ` · à ${formatDistance(distance)} de toi` : ""}
            {z.visited ? " · déjà visitée" : " · jamais visitée"}
          </div>
        </div>
        {#if isLocked}<span class="lock" title="Accès à débloquer">🔒</span>{/if}
      </header>
      <div class="bar"><div style="width: {Math.round(progress(z) * 100)}%"></div></div>
      <ul class="left">
        {#if z.left.boss}<li><strong>{z.left.boss}</strong> boss</li>{/if}
        {#if z.left.grace}<li><strong>{z.left.grace}</strong> grâce{z.left.grace > 1 ? "s" : ""}</li>{/if}
        {#if z.left.item}<li><strong>{z.left.item}</strong> objet{z.left.item > 1 ? "s" : ""} notable{z.left.item > 1 ? "s" : ""}</li>{/if}
      </ul>
      <div class="actions">
        <button class="primary" onclick={() => show(z)}>Voir la zone</button>
        <button class="ghost" onclick={() => toggle(z.key)}>{revealed.has(z.key) ? "Masquer" : "Révéler (spoil)"}</button>
      </div>
      {#if revealed.has(z.key)}
        <ul class="spoil">
          {#each z.contents.filter((c) => !c.done) as c (c.id)}
            <li><button onclick={() => (ui.sheet = c.id)}>{label(c.id)}</button><span class="muted small">{c.kind === "boss" ? "boss" : c.kind === "grace" ? "grâce" : "objet"}</span></li>
          {/each}
        </ul>
      {/if}
    </article>
  {:else}
    <p class="muted">Rien à suggérer avec ce filtre.</p>
  {/each}
</div>

<style>
  h2 {
    margin: 0 0 6px;
  }
  .notice {
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .filters {
    display: inline-flex;
    margin: 12px 0 14px;
  }
  .zones {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 12px;
  }
  .zone {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .zone.locked {
    opacity: 0.7;
  }
  header {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  h3 {
    margin: 0;
    font-family: Georgia, serif;
    font-weight: 500;
    color: var(--gold);
  }
  .small {
    font-size: 12px;
  }
  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: var(--gold);
  }
  .left {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .primary {
    background: var(--gold);
    color: #1a1508;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .ghost {
    background: none;
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    border-radius: 6px;
    padding: 6px 12px;
    font: inherit;
    cursor: pointer;
  }
  .spoil {
    list-style: none;
    padding: 8px 0 0;
    margin: 0;
    border-top: 1px solid var(--border);
  }
  .spoil li {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    padding: 2px 0;
  }
  .spoil button {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .spoil button:hover {
    color: var(--gold);
  }
</style>
