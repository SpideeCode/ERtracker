<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { entriesById, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import RouteCard from "./RouteCard.svelte";
  import AccessCard from "./AccessCard.svelte";

  let { id }: { id: string } = $props();

  const DAMAGE = ["Standard", "Tranchant", "Contondant", "Perforant", "Magie", "Feu", "Foudre", "Sacré"];
  const STATUS = ["Poison", "Putréfaction", "Saignement", "Mort", "Givre", "Sommeil", "Folie"];

  const entry = $derived(entriesById.get(id)!);
  const sheet = $derived(gamedata.bosses.get(id));
  const pos = $derived(gamedata.positions.get(id));
  const done = $derived(tracker.completed.has(id));
  const englishBase = $derived(entry.name.split(" (")[0]);
  const name = $derived(gamedata.name(entry));

  // Damage types and status effects worth using, and the ones to avoid.
  const weak = $derived.by(() => {
    if (!sheet?.absorption) return [];
    const min = Math.min(...sheet.absorption);
    const types = min < 20 ? sheet.absorption.flatMap((v, i) => (v === min ? [DAMAGE[i]] : [])) : [];
    const status = (sheet.resistance ?? [])
      .flatMap((v, i) => (v < 200 && i !== 3 ? [{ name: STATUS[i], v }] : []))
      .sort((a, b) => a.v - b.v)
      .slice(0, 2)
      .map((s) => s.name);
    return [...types, ...status];
  });
  const strong = $derived.by(() => {
    if (!sheet) return [];
    const types = (sheet.absorption ?? []).flatMap((v, i) => (v >= 40 ? [DAMAGE[i]] : []));
    const immune = (sheet.resistance ?? []).flatMap((v, i) => (v >= 999 && i !== 3 ? [STATUS[i]] : []));
    return [...types, ...immune.map((s) => `${s} (immunisé)`)];
  });

  function absorptionClass(v: number) {
    if (v < 0) return "weak";
    if (v <= 10) return "low";
    if (v >= 40) return "high";
    return "";
  }

  const fextralife = $derived(`https://eldenring.wiki.fextralife.com/${encodeURIComponent(englishBase).replace(/%20/g, "+")}`);
  const youtube = $derived(
    `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring ${englishBase} emplacement guide`)}`,
  );
</script>

<header>
  <div class="badges">
    <span class="badge" class:ok={done}>{done ? "Vaincu ✓" : "À vaincre"}</span>
    {#if sheet?.legendary}<span class="badge gold">★ Boss légendaire</span>{/if}
  </div>
  <h2>{name}</h2>
  {#if name !== entry.name}<div class="muted small">{entry.name}</div>{/if}
  <div class="meta">
    {#if gamedata.region(entry)}<span>{gamedata.region(entry)}</span>{/if}
    {#if sheet?.runes}<span>{sheet.runes.toLocaleString("fr-FR")} runes</span>{/if}
  </div>
  {#if sheet?.remembrance}
    <div class="remembrance">Donne : <strong>{sheet.remembrance.fr ?? sheet.remembrance.en}</strong></div>
  {/if}
</header>

{#if !gamedata.loaded || (!sheet && mapdata.running)}
  <p class="notice">
    {mapdata.running ? "Mise à jour des données du jeu en cours…" : "Prépare les cartes (onglet Carte) pour voir l'itinéraire et les défenses."}
  </p>
{/if}

<AccessCard {entry} />

{#if pos}
  <RouteCard {id} {pos} goal="le boss" />
{/if}

{#if sheet?.absorption}
  <section class="card">
    <h3>Combat</h3>
    {#if weak.length}
      <div class="chips-row">
        <span class="label">Points faibles</span>
        <div class="chips">{#each weak as w}<span class="chip good">{w}</span>{/each}</div>
      </div>
    {/if}
    {#if strong.length}
      <div class="chips-row">
        <span class="label">À éviter</span>
        <div class="chips">{#each strong as s}<span class="chip bad">{s}</span>{/each}</div>
      </div>
    {/if}

    <details>
      <summary>Détail des défenses</summary>
      <div class="grid">
        {#each sheet.absorption as v, i}
          <div class="cell {absorptionClass(v)}"><span>{DAMAGE[i]}</span><strong>{v} %</strong></div>
        {/each}
      </div>
      <p class="muted small">Dégâts absorbés : plus c'est bas, plus le boss y est sensible.</p>
      {#if sheet.resistance}
        <div class="grid">
          {#each sheet.resistance as v, i}
            <div class="cell {v >= 999 ? 'high' : v < 200 ? 'low' : ''}">
              <span>{STATUS[i]}</span><strong>{v >= 999 ? "Immunisé" : v}</strong>
            </div>
          {/each}
        </div>
        <p class="muted small">Seuil d'accumulation : plus il est bas, plus l'effet se déclenche vite.</p>
      {/if}
    </details>
  </section>
{/if}

<section class="links">
  <button onclick={() => openUrl(fextralife)}>Stratégie (wiki) ↗</button>
  <button onclick={() => openUrl(youtube)}>Vidéos ↗</button>
</section>

<style>
  .remembrance {
    margin-top: 6px;
    color: var(--text-dim);
  }
  details {
    margin-top: 12px;
  }
  summary {
    cursor: pointer;
    color: var(--text-dim);
    font-size: 13px;
  }
  .chips-row {
    display: flex;
    gap: 10px;
    align-items: baseline;
    margin-bottom: 8px;
  }
  .label {
    flex-shrink: 0;
    width: 92px;
    font-size: 12px;
    color: var(--muted);
  }
  .chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .chip {
    font-size: 12px;
    padding: 2px 9px;
    border-radius: 10px;
    border: 1px solid;
  }
  .chip.good {
    color: #8fd893;
    border-color: #3f6b42;
    background: rgba(95, 174, 106, 0.1);
  }
  .chip.bad {
    color: var(--text-dim);
    border-color: var(--border-strong);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    margin-top: 10px;
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
    color: var(--muted);
  }
</style>
