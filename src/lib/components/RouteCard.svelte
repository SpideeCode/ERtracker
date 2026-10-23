<script lang="ts">
  // "Y aller" card shared by every sheet (boss, grace, item).
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { formatClimb, formatDistance, heading, nearestGraces, planRoute } from "$lib/route";
  import { ui } from "$lib/ui.svelte";
  import { walkPath, type WalkPath } from "$lib/pathfind";
  import type { MarkerPos } from "$lib/types";

  let {
    id,
    pos,
    goal,
    howTo,
  }: {
    id: string;
    pos: MarkerPos;
    /** "le boss", "la grâce", "l'objet" */
    goal: string;
    /** English name to search "how to get to …" for, when the target itself may be hard to reach. */
    howTo?: string;
  } = $props();

  const plan = $derived(
    planRoute(pos, gamedata.positions, tracker.completed, gamedata.places, (e) => gamedata.name(e), {
      goal,
      exclude: id.startsWith("grace:") ? id : undefined,
    }),
  );
  const others = $derived(nearestGraces(pos, gamedata.positions, tracker.completed, 5).filter((g) => g.grace.id !== id).slice(0, 4));

  const me = $derived(tracker.snapshot?.map_position);

  // Starting point of the walking route: any grace of this map (discovered or not) or the last save.
  const startOptions = $derived(
    nearestGraces(pos, gamedata.positions, tracker.completed, 25).filter((g) => g.grace.id !== id),
  );
  const defaultStart = $derived(plan.warp?.grace.id ?? startOptions[0]?.grace.id ?? "");
  let chosen = $state<string | null>(null);
  const start = $derived(chosen ?? ui.routeStart[id] ?? defaultStart);
  const startPos = $derived(
    start === "me"
      ? me && me.map === pos.map
        ? me
        : undefined
      : gamedata.positions.get(start),
  );
  let walk = $state<WalkPath | null>(null);
  $effect(() => {
    const from = startPos;
    walk = null;
    if (!from) return;
    let current = true;
    walkPath(pos.map, from, pos).then((w) => current && (walk = w));
    return () => (current = false);
  });

  function showRoute() {
    ui.routeStart = { ...ui.routeStart, [id]: start };
    ui.showOnMap(id);
  }
  const fromMe = $derived(
    me && me.map === pos.map ? { distance: Math.hypot(pos.x - me.x, pos.y - me.y), direction: heading(pos.x - me.x, pos.y - me.y) } : null,
  );

  // Ask the community: either for the grace that opens the area, or for the target itself.
  const help = $derived.by(() => {
    const target = plan.gate ? { en: plan.gate.grace.name, label: gamedata.name(plan.gate.grace) } : howTo ? { en: howTo, label: null } : null;
    if (!target) return null;
    return {
      label: target.label ? `Comment accéder à ${target.label} ?` : "Comment y accéder ?",
      url: `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring how to get to ${target.en}`)}`,
    };
  });
</script>

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
  {#if help}
    <button class="secondary" onclick={() => openUrl(help.url)}>{help.label} ↗</button>
  {/if}
  <div class="walk">
    <label>
      Partir de
      <select value={start} onchange={(e) => (chosen = e.currentTarget.value)}>
        {#if me && me.map === pos.map}<option value="me">Ma position (dernière sauvegarde)</option>{/if}
        {#each startOptions as g (g.grace.id)}
          <option value={g.grace.id}>{g.discovered ? "✓ " : ""}{gamedata.name(g.grace)} — {formatDistance(g.distance)}</option>
        {/each}
      </select>
    </label>
    {#if !startPos}
      <p class="muted small">Choisis un point de départ sur la même carte.</p>
    {:else if !walk}
      <p class="muted small">Calcul du trajet…</p>
    {:else if walk.found}
      <p>
        Trajet à pied : <strong>{formatDistance(walk.length)}</strong>
        <span class="muted">· ↑ {Math.round(walk.ascent)} m · ↓ {Math.round(walk.descent)} m</span>
      </p>
    {:else}
      <p class="muted small">Pas de chemin trouvé dans les données du terrain (ascenseur, échelle ou zone à part) : la carte montrera la direction à vol d'oiseau.</p>
    {/if}
  </div>
  <button class="primary" onclick={showRoute}>Afficher l'itinéraire sur la carte</button>

  <details>
    <summary>Grâces les plus proches</summary>
    <ul class="graces">
      {#each others as g (g.grace.id)}
        <li class:discovered={g.discovered}>
          <span class="dot" title={g.discovered ? "Découverte" : "Pas encore découverte"}></span>
          <div>
            <button class="place" title="Voir la fiche" onclick={() => (ui.sheet = g.grace.id)}>{gamedata.name(g.grace)}</button>
            <div class="muted small">{formatDistance(g.distance)} · {g.direction} · {formatClimb(g.climb)}</div>
          </div>
        </li>
      {/each}
    </ul>
    <p class="muted small">Point doré : grâce déjà découverte.</p>
  </details>
</section>

<style>
  .card {
    margin-top: 18px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
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
  .where {
    margin: 0 0 10px;
    color: var(--text-dim);
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
  .steps li.warning .num {
    background: #d39b45;
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
  .walk {
    border-top: 1px solid var(--border);
    margin-top: 6px;
    padding-top: 10px;
  }
  .walk label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  .walk select {
    width: 100%;
  }
  .walk p {
    margin: 8px 0;
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
  .secondary {
    width: 100%;
    margin: 0 0 8px;
    background: none;
    border: 1px solid #d39b45;
    color: #f0c27a;
    border-radius: 6px;
    padding: 7px 12px;
    font: inherit;
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
</style>
