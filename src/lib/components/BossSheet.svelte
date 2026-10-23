<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { entriesById, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { formatClimb, formatDistance, heading, nearestGraces, planRoute } from "$lib/route";
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

  const plan = $derived(
    pos ? planRoute(pos, gamedata.positions, tracker.completed, gamedata.places, (e) => gamedata.name(e)) : null,
  );
  const others = $derived(pos ? nearestGraces(pos, gamedata.positions, tracker.completed, 4) : []);

  const me = $derived(tracker.snapshot?.map_position);
  const fromMe = $derived(
    me && pos && me.map === pos.map
      ? { distance: Math.hypot(pos.x - me.x, pos.y - me.y), direction: heading(pos.x - me.x, pos.y - me.y) }
      : null,
  );

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

<aside class="sheet">
  <button class="close" onclick={() => (ui.sheet = null)} title="Fermer (Échap)">×</button>

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

  {#if pos && plan}
    <section class="card">
      <h3>Y aller</h3>
      {#if plan.dungeon}
        <p class="where">Dans un donjon : <strong>{plan.dungeon}</strong></p>
      {/if}
      <ol class="steps">
        {#each plan.steps as step, i}
          <li class:warning={step.warning}>
            <span class="num">{step.warning ? "!" : plan.steps.slice(0, i).filter((s) => !s.warning).length + 1}</span>
            <div>
              {#each step.parts as part}{#if typeof part === "string"}{part}{:else if part.place}<button
                    class="place"
                    title="Voir sur la carte"
                    onclick={() => part.place && ui.showPlace({ ...part.place, label: part.strong, route: id })}
                  >{part.strong}</button>{:else}<strong>{part.strong}</strong>{/if}{/each}
              {#if step.note}<div class="muted small">{step.note}</div>{/if}
            </div>
          </li>
        {/each}
      </ol>
      {#if fromMe}
        <p class="muted small">Depuis ta dernière sauvegarde : {formatDistance(fromMe.distance)} vers le {fromMe.direction}.</p>
      {/if}
      <button class="primary" onclick={() => ui.showOnMap(id)}>Afficher l'itinéraire sur la carte</button>

      <details>
        <summary>Grâces les plus proches</summary>
        <ul class="graces">
          {#each others as g (g.grace.id)}
            <li class:discovered={g.discovered}>
              <span class="dot" title={g.discovered ? "Découverte" : "Pas encore découverte"}></span>
              <div>
                <button
                  class="place"
                  title="Voir sur la carte"
                  onclick={() => ui.showPlace({ map: g.pos.map, x: g.pos.x, y: g.pos.y, label: gamedata.name(g.grace), route: id })}
                >{gamedata.name(g.grace)}</button>
                <div class="muted small">{formatDistance(g.distance)} · {g.direction} · {formatClimb(g.climb)}</div>
              </div>
            </li>
          {/each}
        </ul>
        <p class="muted small">Point doré : grâce déjà découverte.</p>
      </details>
    </section>
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
</aside>

<style>
  .sheet {
    position: relative;
    height: 100%;
    overflow-y: auto;
    padding: 20px 20px 28px;
    background: var(--surface);
    border-left: 1px solid var(--border-strong);
    box-sizing: border-box;
  }
  .close {
    position: absolute;
    top: 10px;
    right: 12px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .close:hover {
    color: var(--text);
  }
  h2 {
    margin: 10px 40px 2px 0;
    font-family: Georgia, serif;
    font-weight: 500;
    font-size: 24px;
    line-height: 1.2;
    color: var(--gold);
  }
  h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin: 0 0 10px;
  }
  .small {
    font-size: 12px;
  }
  .meta {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
    color: var(--text-dim);
  }
  .meta span + span::before {
    content: "·";
    margin-right: 6px;
    color: var(--muted);
  }
  .remembrance {
    margin-top: 6px;
    color: var(--text-dim);
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
  .notice {
    margin-top: 16px;
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 8px 10px;
  }
  .card {
    margin-top: 18px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
  }
  .steps {
    list-style: none;
    padding: 0;
    margin: 0 0 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .steps li {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }
  .place {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-weight: 700;
    color: var(--gold);
    cursor: pointer;
    text-align: left;
    text-decoration: underline dotted var(--gold-dim);
    text-underline-offset: 3px;
  }
  .place:hover {
    color: #f3d27a;
    text-decoration-style: solid;
  }
  .graces .place {
    font-weight: 400;
    color: inherit;
  }
  .where {
    margin: 0 0 10px;
    color: var(--text-dim);
  }
  .steps li.warning .num {
    background: #d39b45;
  }
  .steps li.warning {
    color: #f0c27a;
  }
  .num {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--gold);
    color: #1a1508;
    font-weight: 700;
    font-size: 12px;
    display: grid;
    place-items: center;
  }
  .primary {
    width: 100%;
    margin-top: 4px;
    background: var(--gold);
    color: #1a1508;
    border: none;
    border-radius: 6px;
    padding: 9px 14px;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  details {
    margin-top: 12px;
  }
  summary {
    cursor: pointer;
    color: var(--text-dim);
    font-size: 13px;
  }
  .graces {
    list-style: none;
    padding: 0;
    margin: 8px 0 4px;
  }
  .graces li {
    display: flex;
    gap: 8px;
    padding: 4px 0;
  }
  .dot {
    flex-shrink: 0;
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
  .graces li:not(.discovered) {
    color: var(--text-dim);
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
  .links {
    display: flex;
    gap: 8px;
    margin-top: 18px;
  }
  .links button {
    flex: 1;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 7px 10px;
    font: inherit;
    cursor: pointer;
  }
  .links button:hover {
    border-color: var(--gold-dim);
  }
</style>
