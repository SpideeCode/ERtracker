<script lang="ts">
  // Home: what matters right now. Where you are, the next boss, where to go, this session.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { MapPin, Swords, Telescope, Map as MapIcon, Layers, BookOpen, Skull, Trophy } from "@lucide/svelte";
  import { db, describe, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { zoneData } from "$lib/zoneData.svelte";
  import { journal } from "$lib/journal.svelte";
  import { analyse, formatDuration, sessions } from "$lib/history";
  import { locked } from "$lib/accessCheck.svelte";
  import { formatDistance, heading } from "$lib/route";
  import { achievement, ago } from "$lib/labels";
  import { ui } from "$lib/ui.svelte";
  import type { Zone } from "$lib/zones";

  let { now }: { now: number } = $props();

  onMount(() => journal.reload());

  const MAPS: Record<string, string> = { M00: "Entre-terre", M01: "Souterrain", M10: "Royaume de l'Ombre", M11: "Souterrain (DLC)" };
  const me = $derived(tracker.snapshot?.map_position ?? null);
  const bosses = $derived(db.entries.filter((e) => e.category === "boss"));
  const analysis = $derived(analyse(journal.records, bosses, gamedata.positions, tracker.completed));

  // Closest boss you can reach and have not beaten (locked ones come last).
  const nextBoss = $derived.by(() => {
    if (!me) return null;
    const list = bosses.flatMap((b) => {
      const p = gamedata.positions.get(b.id);
      if (tracker.completed.has(b.id) || !p || p.map !== me.map) return [];
      const d = Math.hypot(p.x - me.x, p.y - me.y);
      return [{ b, d, dir: heading(p.x - me.x, p.y - me.y), locked: locked(b) }];
    });
    list.sort((a, b) => Number(a.locked) - Number(b.locked) || a.d - b.d);
    return list[0] ?? null;
  });
  const nextSheet = $derived(nextBoss ? gamedata.bosses.get(nextBoss.b.id) : undefined);
  const nextTries = $derived(nextBoss ? (analysis.bosses.find((s) => s.id === nextBoss.b.id)?.attempts ?? 0) : 0);

  const zone = $derived(zoneData.current);
  const suggestions = $derived(zoneData.suggestions.filter((r) => r.z.key !== zone?.key).slice(0, 3));

  const session = $derived(sessions(journal.records).at(-1) ?? null);
  const sessionLive = $derived(session ? now - session.end < 30 * 60 * 1000 : false);
  const unlocked = $derived(
    (session?.gained ?? [])
      .map((id) => ({ id, ...describe(id) }))
      .filter((u) => !["item:material", "item:rune", "item:consumable", "item:upgrade", "quest"].includes(u.category))
      .reverse()
      .slice(0, 8),
  );

  // Overall progress: bosses, graces, items, dungeons visited.
  const stats = $derived.by(() => {
    const count = (ids: string[]) => ({ done: ids.filter((id) => tracker.completed.has(id)).length, total: ids.length });
    const d = zoneData.dungeons;
    return [
      { label: "Boss", ...count(bosses.map((b) => b.id)), go: () => ui.progress("bosses") },
      { label: "Grâces", ...count(db.entries.filter((e) => e.category === "grace").map((e) => e.id)), go: () => ui.progress("graces") },
      { label: "Objets", ...count([...gamedata.items.keys()]), go: () => ui.progress("items") },
      { label: "Donjons visités", done: d.filter((x) => x.done).length, total: d.length, go: () => ui.go({ kind: "explore" }) },
    ];
  });
  const overall = $derived.by(() => {
    const done = stats.reduce((n, s) => n + s.done, 0);
    const total = stats.reduce((n, s) => n + s.total, 0);
    return total ? Math.round((done / total) * 100) : 0;
  });

  const showZone = (z: Zone) => ui.showPlace({ map: z.map, x: z.x, y: z.y, label: zoneData.name(z), radius: z.radius });
  const leftText = (z: Zone) =>
    [
      z.left.boss && `${z.left.boss} boss`,
      z.left.grace && `${z.left.grace} grâce${z.left.grace > 1 ? "s" : ""}`,
      z.left.item && `${z.left.item} objet${z.left.item > 1 ? "s" : ""}`,
      z.left.dungeon && `${z.left.dungeon} donjon${z.left.dungeon > 1 ? "s" : ""}`,
    ]
      .filter(Boolean)
      .join(" · ");

  function centerOnMe() {
    if (!me) return;
    ui.showPlace({ map: me.map, x: me.x, y: me.y, label: "Ma position (dernière sauvegarde)" });
  }
</script>

<div class="grid">
  <!-- Where you are -->
  <section class="card hero">
    <div class="kicker"><MapPin size={14} /> Où tu es</div>
    {#if zone}
      <h2>{zoneData.name(zone)}</h2>
      <p class="muted">{MAPS[zone.map]}{tracker.snapshot ? ` · position lue ${ago(tracker.snapshot.loaded_at, now)}` : ""}</p>
      {#if zone.score}
        <p class="left">Il reste ici : <strong>{leftText(zone)}</strong></p>
      {:else}
        <p class="left done">Zone terminée ✓</p>
      {/if}
      <div class="actions">
        <button class="primary" onclick={() => showZone(zone)}><MapIcon size={16} /> Voir la zone</button>
        <button class="ghost" onclick={centerOnMe}>Ma position</button>
      </div>
    {:else if !gamedata.loaded}
      <p class="muted">Prépare les cartes (onglet Carte) pour savoir où tu es et ce qu'il reste autour.</p>
    {:else}
      <p class="muted">Ta position apparaîtra à la prochaine sauvegarde du jeu.</p>
    {/if}
  </section>

  <!-- Next boss -->
  <section class="card">
    <div class="kicker"><Swords size={14} /> Prochain boss</div>
    {#if nextBoss}
      <h3>{gamedata.name(nextBoss.b)}{nextSheet?.legendary ? " ★" : ""}{nextBoss.locked ? " 🔒" : ""}</h3>
      <p class="muted">
        {formatDistance(nextBoss.d)} vers le {nextBoss.dir}{nextSheet?.runes ? ` · ${nextSheet.runes.toLocaleString("fr-FR")} runes` : ""}{nextTries
          ? ` · ${nextTries} essai${nextTries > 1 ? "s" : ""}`
          : ""}
      </p>
      <div class="actions">
        <button class="primary" onclick={() => ui.showOnMap(nextBoss.b.id)}>Y aller</button>
        <button class="ghost" onclick={() => (ui.sheet = nextBoss.b.id)}>Fiche</button>
      </div>
    {:else}
      <p class="muted">Aucun boss restant sur cette carte.</p>
    {/if}
  </section>

  <!-- This session -->
  <section class="card">
    <div class="kicker"><BookOpen size={14} /> {sessionLive ? "Session en cours" : "Dernière session"}</div>
    {#if session}
      <div class="numbers">
        <div><strong>{formatDuration(session.played * 1000 || session.end - session.start)}</strong><span class="muted small">jouées</span></div>
        <div><strong><Skull size={14} /> {session.deaths}</strong><span class="muted small">morts</span></div>
        <div><strong>{session.gained.filter((g) => g.startsWith("boss:")).length}</strong><span class="muted small">boss</span></div>
      </div>
      {#if unlocked.length}
        <ul class="unlocked">
          {#each unlocked as u (u.id)}
            <li><button onclick={() => (ui.sheet = u.id)}><span class="tag">{achievement(u.category)}</span> {u.label}</button></li>
          {/each}
        </ul>
      {/if}
    {:else}
      <p class="muted">Le journal démarre : joue avec ERtracker ouvert et ta session apparaîtra ici.</p>
    {/if}
    <button class="link" onclick={() => ui.go({ kind: "journey" })}>Mon aventure →</button>
  </section>

  <!-- Where to go -->
  <section class="card wide">
    <div class="kicker"><Telescope size={14} /> Où aller ensuite</div>
    {#if suggestions.length}
      <div class="suggestions">
        {#each suggestions as s (s.z.key)}
          <button class="suggestion" onclick={() => showZone(s.z)}>
            <strong>{zoneData.name(s.z)}</strong>
            <span class="muted small">{s.distance !== null ? `à ${formatDistance(s.distance)}` : MAPS[s.z.map]}{s.z.visited ? " · commencée" : ""}</span>
            <span class="small">{leftText(s.z)}</span>
          </button>
        {/each}
      </div>
      <button class="link" onclick={() => ui.go({ kind: "explore" })}>Toutes les zones et les donjons ratés →</button>
    {:else}
      <p class="muted">Rien à suggérer pour l'instant.</p>
    {/if}
  </section>

  <!-- Progress -->
  <section class="card">
    <div class="kicker"><Trophy size={14} /> Progression</div>
    <div class="overall">
      <div class="ring" style="--p: {overall}"><span>{overall} %</span></div>
      <ul class="bars">
        {#each stats as s}
          <li>
            <button onclick={s.go}>
              <span>{s.label}</span>
              <span class="muted small">{s.done} / {s.total}</span>
              <div class="bar"><div style="width: {s.total ? (s.done / s.total) * 100 : 0}%"></div></div>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <!-- Quick actions -->
  <section class="card quick">
    <div class="kicker"><Layers size={14} /> Raccourcis</div>
    <button class="ghost" onclick={() => ui.go({ kind: "map" })}><MapIcon size={16} /> Carte <kbd>M</kbd></button>
    <button class="ghost" onclick={() => document.getElementById("global-search")?.focus()}>Rechercher <kbd>Ctrl K</kbd></button>
    <button class="ghost" onclick={() => invoke("toggle_overlay")}>Overlay en jeu <kbd>Ctrl Maj O</kbd></button>
    <button class="ghost" onclick={() => ui.go({ kind: "builds" })}><Swords size={16} /> Conseiller de build</button>
  </section>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 14px;
    max-width: 1200px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .hero {
    grid-column: span 2;
    background: linear-gradient(135deg, rgba(201, 169, 91, 0.12), transparent 60%), var(--surface);
    border-color: var(--gold-dim);
  }
  .wide {
    grid-column: span 2;
  }
  @media (max-width: 760px) {
    .hero,
    .wide {
      grid-column: auto;
    }
  }
  .kicker {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
  }
  h2,
  h3 {
    margin: 0;
    font-family: Georgia, serif;
    font-weight: 500;
  }
  h2 {
    font-size: 28px;
    color: var(--text);
  }
  h3 {
    font-size: 20px;
  }
  p {
    margin: 0;
  }
  .left strong {
    color: var(--gold);
  }
  .done {
    color: #6fbf73;
  }
  .small {
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }
  .primary,
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border-radius: 7px;
    padding: 7px 14px;
    font: inherit;
    cursor: pointer;
  }
  .primary {
    background: var(--gold);
    color: #1a1508;
    border: none;
    font-weight: 600;
  }
  .ghost {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
  }
  .ghost:hover {
    border-color: var(--gold-dim);
  }
  .link {
    align-self: flex-start;
    background: none;
    border: none;
    padding: 0;
    color: var(--gold);
    font: inherit;
    cursor: pointer;
  }
  .suggestions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 8px;
  }
  .suggestion {
    text-align: left;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
    color: inherit;
    font: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .suggestion:hover {
    border-color: var(--gold-dim);
  }
  .numbers {
    display: flex;
    gap: 24px;
  }
  .numbers div {
    display: flex;
    flex-direction: column;
  }
  .numbers strong {
    font-size: 20px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .unlocked {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .unlocked button {
    background: none;
    border: none;
    padding: 3px 0;
    color: inherit;
    font: inherit;
    cursor: pointer;
    text-align: left;
  }
  .unlocked button:hover {
    color: var(--gold);
  }
  .tag {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--gold);
    margin-right: 6px;
  }
  .overall {
    display: flex;
    gap: 16px;
    align-items: center;
  }
  .ring {
    --size: 84px;
    width: var(--size);
    height: var(--size);
    border-radius: 50%;
    background: conic-gradient(var(--gold) calc(var(--p) * 1%), var(--surface-2) 0);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .ring span {
    width: calc(var(--size) - 14px);
    height: calc(var(--size) - 14px);
    border-radius: 50%;
    background: var(--surface);
    display: grid;
    place-items: center;
    font-weight: 600;
  }
  .bars {
    list-style: none;
    padding: 0;
    margin: 0;
    flex: 1;
  }
  .bars button {
    width: 100%;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 8px;
    background: none;
    border: none;
    padding: 3px 0;
    color: inherit;
    font: inherit;
    cursor: pointer;
    text-align: left;
  }
  .bar {
    grid-column: 1 / -1;
    height: 5px;
    border-radius: 3px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: var(--gold);
  }
  .quick {
    gap: 6px;
  }
  .quick .ghost {
    justify-content: flex-start;
  }
  kbd {
    margin-left: auto;
    font-size: 11px;
    color: var(--muted);
    background: var(--bg);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 0 5px;
  }
</style>
