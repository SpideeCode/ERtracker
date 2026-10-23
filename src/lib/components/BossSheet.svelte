<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { entriesById, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { formatClimb, formatDistance, nearestGraces } from "$lib/route";
  import { ui } from "$lib/ui.svelte";

  let { id }: { id: string } = $props();

  const DAMAGE = ["Standard", "Tranchant", "Contondant", "Perforant", "Magie", "Feu", "Foudre", "Sacré"];
  const STATUS = ["Poison", "Putréfaction", "Saignement", "Mort", "Givre", "Sommeil", "Folie"];

  const entry = $derived(entriesById.get(id)!);
  const sheet = $derived(gamedata.bosses.get(id));
  const pos = $derived(gamedata.positions.get(id));
  const done = $derived(tracker.completed.has(id));
  const englishBase = $derived(entry.name.split(" (")[0]);
  const name = $derived(gamedata.name(entry));

  const graces = $derived(pos ? nearestGraces(pos, gamedata.positions, tracker.completed, 4) : []);
  // Closest grace the player can actually travel to.
  const warp = $derived(
    pos ? nearestGraces(pos, gamedata.positions, tracker.completed, 500).find((g) => g.discovered) : undefined,
  );

  const weakest = $derived.by(() => {
    if (!sheet?.absorption) return [];
    const min = Math.min(...sheet.absorption);
    const types = min < 20 ? sheet.absorption.flatMap((v, i) => (v === min ? [DAMAGE[i]] : [])) : [];
    const status = (sheet.resistance ?? []).flatMap((v, i) => (v < 200 && i !== 3 ? [{ name: STATUS[i], v }] : []));
    status.sort((a, b) => a.v - b.v);
    return [...types, ...status.slice(0, 2).map((s) => s.name)];
  });

  function absorptionClass(v: number) {
    if (v < 0) return "weak";
    if (v <= 10) return "low";
    if (v >= 50) return "immune";
    if (v >= 35) return "high";
    return "";
  }

  const fextralife = $derived(`https://eldenring.wiki.fextralife.com/${encodeURIComponent(englishBase).replace(/%20/g, "+")}`);
  const youtube = $derived(
    `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring ${englishBase} emplacement guide`)}`,
  );
</script>

<aside class="sheet">
  <button class="close" onclick={() => (ui.sheet = null)} aria-label="Fermer">×</button>

  <header>
    <div class="badges">
      {#if sheet?.legendary}<span class="badge gold">Boss légendaire</span>{/if}
      <span class="badge" class:ok={done}>{done ? "Vaincu ✓" : "À vaincre"}</span>
    </div>
    <h2>{name}</h2>
    {#if name !== entry.name}<div class="muted">{entry.name}</div>{/if}
    {#if gamedata.region(entry)}<div class="region">{gamedata.region(entry)}</div>{/if}
  </header>

  <section class="facts">
    {#if sheet?.runes}
      <div><span class="muted">Runes</span><strong>{sheet.runes.toLocaleString("fr-FR")}</strong></div>
    {/if}
    {#if sheet?.remembrance}
      <div><span class="muted">Souvenir</span><strong>{sheet.remembrance.fr ?? sheet.remembrance.en}</strong></div>
    {/if}
  </section>

  <section>
    <h3>Comment y aller</h3>
    {#if !pos}
      <p class="muted">Position inconnue pour ce boss.</p>
    {:else}
      {#if warp}
        <p class="tip">
          Voyage rapide vers <strong>{gamedata.name(warp.grace)}</strong>, puis {formatDistance(warp.distance)} vers le
          <strong>{warp.direction}</strong> ({formatClimb(warp.climb)}).
        </p>
      {:else}
        <p class="tip">Aucune grâce découverte sur cette carte : il faudra y aller à pied.</p>
      {/if}
      <ul class="graces">
        {#each graces as g (g.grace.id)}
          <li class:discovered={g.discovered}>
            <span class="dot"></span>
            <span class="gname">{gamedata.name(g.grace)}</span>
            <span class="muted small">{formatDistance(g.distance)} · {g.direction} · {formatClimb(g.climb)}</span>
          </li>
        {/each}
      </ul>
      <button class="primary" onclick={() => ui.showOnMap(id)}>Voir sur la carte</button>
    {/if}
  </section>

  {#if sheet?.absorption}
    <section>
      <h3>Défenses</h3>
      {#if weakest.length}
        <p class="tip">Points faibles : <strong>{weakest.join(", ")}</strong></p>
      {/if}
      <div class="grid">
        {#each sheet.absorption as v, i}
          <div class="cell {absorptionClass(v)}">
            <span>{DAMAGE[i]}</span>
            <strong>{v} %</strong>
          </div>
        {/each}
      </div>
      <p class="muted small">Pourcentage de dégâts absorbés : plus c'est bas, plus le boss y est sensible.</p>
    </section>
  {/if}

  {#if sheet?.resistance}
    <section>
      <h3>Altérations d'état</h3>
      <div class="grid">
        {#each sheet.resistance as v, i}
          <div class="cell {v >= 999 ? 'immune' : v < 200 ? 'low' : ''}">
            <span>{STATUS[i]}</span>
            <strong>{v >= 999 ? "Immunisé" : v}</strong>
          </div>
        {/each}
      </div>
      <p class="muted small">Seuil d'accumulation de base : plus il est bas, plus l'effet se déclenche vite.</p>
    </section>
  {/if}

  <section class="links">
    <button onclick={() => openUrl(fextralife)}>Guide et stratégie (wiki) ↗</button>
    <button onclick={() => openUrl(youtube)}>Vidéos ↗</button>
  </section>
</aside>

<style>
  .sheet {
    position: relative;
    height: 100%;
    overflow-y: auto;
    padding: 18px 20px 24px;
    background: var(--surface);
    border-left: 1px solid var(--border-strong);
    box-sizing: border-box;
  }
  .close {
    position: absolute;
    top: 8px;
    right: 10px;
    background: none;
    border: none;
    color: var(--muted);
    font-size: 22px;
    cursor: pointer;
  }
  h2 {
    margin: 8px 0 2px;
    font-family: Georgia, serif;
    font-weight: 500;
    color: var(--gold);
  }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin: 0 0 8px;
  }
  section {
    margin-top: 20px;
  }
  .region {
    margin-top: 4px;
  }
  .badges {
    display: flex;
    gap: 6px;
  }
  .badge {
    font-size: 11px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 1px 8px;
    color: var(--text-dim);
  }
  .badge.gold {
    border-color: var(--gold);
    color: var(--gold);
  }
  .badge.ok {
    border-color: #5fae6a;
    color: #5fae6a;
  }
  .facts {
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
  }
  .facts div {
    display: flex;
    flex-direction: column;
  }
  .tip {
    background: var(--surface-2);
    border-left: 3px solid var(--gold);
    padding: 8px 10px;
    border-radius: 4px;
    margin: 0 0 10px;
  }
  .graces {
    list-style: none;
    padding: 0;
    margin: 0 0 12px;
  }
  .graces li {
    display: grid;
    grid-template-columns: 12px 1fr;
    column-gap: 8px;
    padding: 4px 0;
  }
  .graces .small {
    grid-column: 2;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    border: 1px solid var(--muted);
    margin-top: 6px;
  }
  .discovered .dot {
    background: var(--gold);
    border-color: var(--gold);
  }
  li:not(.discovered) .gname {
    color: var(--text-dim);
  }
  .small {
    font-size: 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .cell {
    background: var(--surface-2);
    border-radius: 6px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    font-size: 12px;
    border: 1px solid transparent;
  }
  .cell strong {
    font-size: 14px;
  }
  .cell.weak {
    border-color: #6fbf73;
    color: #8fd893;
  }
  .cell.low {
    border-color: #4d6b4f;
  }
  .cell.high {
    color: var(--text-dim);
  }
  .cell.immune {
    color: var(--muted);
    opacity: 0.7;
  }
  .primary {
    background: var(--gold);
    color: #1a1508;
    border: none;
    border-radius: 6px;
    padding: 7px 14px;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .links button {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 6px 12px;
    font: inherit;
    cursor: pointer;
  }
</style>
