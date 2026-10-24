<script lang="ts">
  import { Lock } from "@lucide/svelte";
  // "Explorer": zones with things left, without spoilers unless asked.
  import { db } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { pickupTitle } from "$lib/items";
  import { progress, type Zone } from "$lib/zones";
  import { zoneData } from "$lib/zoneData.svelte";
  import { formatDistance } from "$lib/route";
  import { ui } from "$lib/ui.svelte";
  import { PLACE_KINDS } from "$lib/places";

  const MAPS: Record<string, string> = { M00: "Entre-terre", M01: "Souterrain", M10: "Royaume de l'Ombre", M11: "Souterrain (DLC)" };

  let filter = $state<"suggest" | "started" | "all">("suggest");
  let revealed = $state<Set<string>>(new Set());

  /** "2 catacombes et 1 grotte" */
  const plural = (kind: string, n: number) => {
    const entry = Object.values(PLACE_KINDS).find(([one]) => one === kind);
    return `${n} ${n > 1 ? (entry?.[1] ?? kind + "s") : kind}`;
  };
  const listKinds = (counts: Record<string, number>) => {
    const parts = Object.entries(counts).map(([k, n]) => plural(k, n));
    return parts.length > 1 ? `${parts.slice(0, -1).join(", ")} et ${parts[parts.length - 1]}` : (parts[0] ?? "");
  };

  // Per region: dungeons never visited, the easiest things to miss.
  const byRegion = $derived.by(() => {
    const regions = new Map<string, Record<string, number>>();
    for (const z of zoneData.zones) {
      if (!z.left.dungeon) continue;
      const r = regions.get(z.region) ?? {};
      for (const [k, n] of Object.entries(z.dungeonsLeft)) r[k] = (r[k] ?? 0) + n;
      regions.set(z.region, r);
    }
    return [...regions].sort((a, b) => Object.values(b[1]).reduce((x, y) => x + y, 0) - Object.values(a[1]).reduce((x, y) => x + y, 0));
  });

  const regionName = (en: string) => gamedata.regions[en] ?? en;

  const shown = $derived.by(() => {
    if (filter === "started") return zoneData.ranked.filter((r) => r.z.visited).sort((a, b) => b.z.score - a.z.score);
    if (filter === "all") return [...zoneData.ranked].sort((a, b) => b.z.score - a.z.score);
    return zoneData.suggestions.slice(0, 12);
  });

  function show(z: Zone) {
    ui.showPlace({ map: z.map, x: z.x, y: z.y, label: zoneData.name(z), radius: z.radius });
  }

  function toggle(key: string) {
    const next = new Set(revealed);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    revealed = next;
  }

  const label = (id: string) => {
    if (id.startsWith("place:")) {
      const p = gamedata.places[Number(id.slice(6))];
      return p ? (p.name.fr ?? p.name.en) : id;
    }
    const e = db.entries.find((x) => x.id === id);
    if (e) return gamedata.name(e);
    const p = gamedata.items.get(id);
    return p ? pickupTitle(p) : id;
  };
</script>

<h2>Explorer</h2>
<p class="muted">
  Les zones où il te reste le plus à découvrir, sans rien dévoiler : seulement combien de boss, de grâces et d'objets
  notables restent à trouver.
</p>

{#if !gamedata.loaded}
  <p class="notice">Prépare les cartes (onglet Carte) pour activer les suggestions.</p>
{/if}

{#if byRegion.length}
  <section class="missed">
    <h3>Donjons jamais visités</h3>
    <ul>
      {#each byRegion as [region, counts] (region)}
        <li><strong>{regionName(region)}</strong> : {listKinds(counts)}</li>
      {/each}
    </ul>
  </section>
{/if}

<div class="segmented filters">
  <button class:active={filter === "suggest"} onclick={() => (filter = "suggest")}>Suggestions</button>
  <button class:active={filter === "started"} onclick={() => (filter = "started")}>Zones commencées</button>
  <button class:active={filter === "all"} onclick={() => (filter = "all")}>Toutes</button>
</div>

<div class="zones">
  {#each shown as { z, distance, locked: isLocked } (z.key)}
    {@const name = zoneData.name(z)}
    <article class="zone" class:locked={isLocked}>
      <header>
        <div>
          <h3>{name}</h3>
          <div class="muted small">
            {MAPS[z.map]}{distance !== null ? ` · à ${formatDistance(distance)} de toi` : ""}
            {z.visited ? " · déjà visitée" : " · jamais visitée"}
          </div>
        </div>
        {#if isLocked}<span class="lock" title="Accès à débloquer"><Lock size={13} /></span>{/if}
      </header>
      <div class="bar"><div style="width: {Math.round(progress(z) * 100)}%"></div></div>
      <ul class="left">
        {#if z.left.boss}<li><strong>{z.left.boss}</strong> boss</li>{/if}
        {#if z.left.grace}<li><strong>{z.left.grace}</strong> grâce{z.left.grace > 1 ? "s" : ""}</li>{/if}
        {#if z.left.item}<li><strong>{z.left.item}</strong> objet{z.left.item > 1 ? "s" : ""} notable{z.left.item > 1 ? "s" : ""}</li>{/if}
        {#if z.left.dungeon}<li class="dungeons">{listKinds(z.dungeonsLeft)} à visiter</li>{/if}
      </ul>
      <div class="actions">
        <button class="primary" onclick={() => show(z)}>Voir la zone</button>
        <button class="ghost" onclick={() => toggle(z.key)}>{revealed.has(z.key) ? "Masquer" : "Révéler (spoil)"}</button>
      </div>
      {#if revealed.has(z.key)}
        <ul class="spoil">
          {#each z.contents.filter((c) => !c.done) as c (c.id)}
            <li><button onclick={() => (ui.sheet = c.id)}>{label(c.id)}</button><span class="muted small">{c.kind === "boss" ? "boss" : c.kind === "grace" ? "grâce" : c.kind === "dungeon" ? c.subtype : "objet"}</span></li>
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
  .missed {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    margin-top: 12px;
  }
  .missed h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
    font-family: inherit;
    margin-bottom: 6px;
  }
  .missed ul {
    margin: 0;
    padding: 0;
    list-style: none;
    columns: 2 260px;
  }
  .missed li {
    padding: 2px 0;
  }
  .dungeons {
    color: #9fb4c8;
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
    font-family: var(--font-title);
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
