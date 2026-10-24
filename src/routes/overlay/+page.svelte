<script lang="ts">
  import { ArrowUp, Castle, Flame, Gem, Lock, MapPin, Skull, Swords, Timer } from "@lucide/svelte";
  // In-game overlay HUD: a transparent always-on-top window over the game
  // (borderless windowed). Click-through until Ctrl+Shift+I.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { journal } from "$lib/journal.svelte";
  import { analyse, sessions } from "$lib/history";
  import { zoneData } from "$lib/zoneData.svelte";
  import { formatDistance, heading } from "$lib/route";
  import { achievement, ago, playtime } from "$lib/labels";
  import { locked } from "$lib/accessCheck.svelte";
  import { prefs } from "$lib/prefs.svelte";
  import { spoiler } from "$lib/spoiler.svelte";

  type Corner = "tl" | "tr" | "bl" | "br";
  let interactive = $state(false);
  let corner = $state<Corner>("tr");
  let opacity = $state(0.9);
  let scale = $state(1);
  let collapsed = $state(false);

  function load() {
    try {
      const s = JSON.parse(localStorage.getItem("overlay") ?? "{}");
      corner = s.corner ?? corner;
      opacity = s.opacity ?? opacity;
      scale = s.scale ?? scale;
      collapsed = s.collapsed ?? collapsed;
    } catch {
      /* settings are a convenience */
    }
  }
  $effect(() => {
    const s = JSON.stringify({ corner, opacity, scale, collapsed });
    try {
      localStorage.setItem("overlay", s);
    } catch {
      /* ignore */
    }
  });

  onMount(() => {
    load();
    prefs.load();
    // The main window changes the preferences; pick them up here too.
    const onStorage = () => prefs.load();
    window.addEventListener("storage", onStorage);
    tracker.start();
    gamedata.load();
    journal.reload();
    const un = listen<boolean>("overlay://interactive", (e) => (interactive = e.payload));
    return () => {
      window.removeEventListener("storage", onStorage);
      un.then((f) => f());
    };
  });

  const me = $derived(tracker.snapshot?.map_position ?? null);
  const bosses = $derived(db.entries.filter((e) => e.category === "boss"));
  const analysis = $derived(analyse(journal.records, bosses, gamedata.positions, tracker.completed));

  // Closest boss not defeated yet, on the player's map.
  const nextBoss = $derived.by(() => {
    if (!me) return null;
    let best: { id: string; name: string; d: number; dir: string; climb: number } | null = null;
    for (const b of bosses) {
      if (tracker.completed.has(b.id)) continue;
      const p = gamedata.positions.get(b.id);
      if (!p || p.map !== me.map) continue;
      const d = Math.hypot(p.x - me.x, p.y - me.y);
      if (!best || d < best.d) best = { id: b.id, name: spoiler.name(b.id, gamedata.name(b)), d, dir: heading(p.x - me.x, p.y - me.y), climb: p.height - me.height };
    }
    return best;
  });
  const attempts = $derived(nextBoss ? (analysis.bosses.find((b) => b.id === nextBoss.id)?.attempts ?? 0) : 0);

  const nextLocked = $derived.by(() => {
    const b = nextBoss && bosses.find((e) => e.id === nextBoss.id);
    return b ? locked(b) : false;
  });
  /** Map angle of the boss, 0 = north, clockwise. */
  const angle = $derived.by(() => {
    const p = nextBoss && gamedata.positions.get(nextBoss.id);
    return p && me ? (Math.atan2(p.x - me.x, -(p.y - me.y)) * 180) / Math.PI : 0;
  });

  // The zone the player stands in, and what is left there.
  const zone = $derived(zoneData.current);
  const zoneDone = $derived.by(() => {
    if (!zone) return 0;
    const t = zone.total.boss + zone.total.grace + zone.total.item + zone.total.dungeon;
    const l = zone.left.boss + zone.left.grace + zone.left.item + zone.left.dungeon;
    return t ? Math.round(((t - l) / t) * 100) : 100;
  });

  const session = $derived(sessions(journal.records).at(-1) ?? null);
  const sessionBosses = $derived(session?.gained.filter((id) => id.startsWith("boss:")).length ?? 0);
  const sessionItems = $derived(session?.gained.filter((id) => id.startsWith("item:")).length ?? 0);
  const climb = (m: number) =>
    Math.abs(m) < 15 ? "" : m > 0 ? ` · ${Math.round(m)} m plus haut` : ` · ${Math.round(-m)} m plus bas`;
  /** "1 grâce", "3 grâces"; words ending in s do not change. */
  const plural = (n: number, word: string) => `${n} ${word}${n > 1 && !word.endsWith("s") ? "s" : ""}`;

  let now = $state(Date.now());
  $effect(() => {
    const t = setInterval(() => (now = Date.now()), 5000);
    return () => clearInterval(t);
  });

  async function close() {
    await invoke("set_overlay_interactive", { on: false });
    await invoke("toggle_overlay");
  }
</script>

<div class="screen" class:interactive>
  <div class="hud {corner}" style="opacity: {opacity}; transform: scale({scale})">
    <div class="title">
      <span class="brand">ERtracker</span>
      {#if tracker.character}<span class="muted">{tracker.character.name} · niveau {tracker.character.level}</span>{/if}
      {#if interactive}<button class="mini" onclick={() => (collapsed = !collapsed)}>{collapsed ? "▾" : "▴"}</button>{/if}
    </div>
    {#if tracker.snapshot}
      <div class="sync"><span class="pulse"></span> Sauvegarde lue {ago(tracker.snapshot.loaded_at, now)}</div>
    {/if}

    {#if !collapsed}
      {#if zone}
        <div class="block">
          <div class="label"><MapPin size={12} /> Tu es ici</div>
          <div class="value">{zoneData.name(zone)}</div>
          {#if zone.score}
            <div class="bar" title="{zoneDone} % de la zone fait"><div style="width: {zoneDone}%"></div></div>
            <div class="left">
              <span class="muted">Il reste ici :</span>
              {#if zone.left.boss}<span><Swords size={12} /> {plural(zone.left.boss, "boss")}</span>{/if}
              {#if zone.left.grace}<span><Flame size={12} /> {plural(zone.left.grace, "grâce")}</span>{/if}
              {#if zone.left.item}<span><Gem size={12} /> {plural(zone.left.item, "objet")}</span>{/if}
              {#if zone.left.dungeon}<span><Castle size={12} /> {plural(zone.left.dungeon, "donjon")}</span>{/if}
            </div>
          {:else}
            <div class="done">Zone terminée : rien d'important à trouver ici ✓</div>
          {/if}
        </div>
      {/if}

      {#if nextBoss}
        <div class="block">
          <div class="label"><Swords size={12} /> Boss pas encore vaincu le plus proche</div>
          <div class="boss">
            <span class="arrow" style="transform: rotate({angle}deg)" title="Direction sur la carte (nord en haut)"><ArrowUp size={18} /></span>
            <div>
              <div class="value">{nextBoss.name}</div>
              <div class="muted">{formatDistance(nextBoss.d)} vers le {nextBoss.dir}{climb(nextBoss.climb)}</div>
            </div>
          </div>
          {#if nextLocked}<div class="warn"><Lock size={12} /> Accès à débloquer d'abord (voir sa fiche)</div>{/if}
          {#if attempts}<div class="muted">Tu y es mort {plural(attempts, "fois")} jusqu'ici</div>{/if}
        </div>
      {/if}

      <div class="block">
        <div class="label"><Timer size={12} /> Cette session</div>
        {#if session}
          <div class="stats">
            <span><strong>{session.played < 3600 ? `${Math.floor(session.played / 60)} min` : playtime(session.played)}</strong> de jeu</span>
            <span><strong>{session.deaths}</strong> mort{session.deaths > 1 ? "s" : ""}</span>
            <span><strong>{sessionBosses}</strong> boss</span>
            <span><strong>{sessionItems}</strong> objet{sessionItems > 1 ? "s" : ""}</span>
          </div>
        {:else}
          <div class="muted">Commence dès la prochaine sauvegarde du jeu.</div>
        {/if}
        <div class="muted total"><Skull size={12} /> {tracker.character?.deaths.toLocaleString("fr-FR") ?? "—"} morts depuis le début</div>
      </div>
      {#if !interactive}<div class="hint">Ctrl+Maj+I pour régler · Ctrl+Maj+O pour masquer</div>{/if}
    {/if}

    {#if interactive}
      <div class="settings">
        <div class="corners">
          {#each [["tl", "↖"], ["tr", "↗"], ["bl", "↙"], ["br", "↘"]] as [c, icon]}
            <button class:active={corner === c} onclick={() => (corner = c as Corner)}>{icon}</button>
          {/each}
        </div>
        <label>Opacité <input type="range" min="0.3" max="1" step="0.05" bind:value={opacity} /></label>
        <label>Taille <input type="range" min="0.7" max="1.6" step="0.05" bind:value={scale} /></label>
        <button class="mini wide" onclick={close}>Fermer l'overlay</button>
      </div>
      <div class="hint">Ctrl+Maj+I : revenir au jeu · Ctrl+Maj+O : masquer</div>
    {/if}
  </div>

  <div class="toasts {corner}">
    {#each tracker.toasts as t (t.key)}
      <div class="toast">
        <div class="kicker">{achievement(t.category)}</div>
        <div class="toast-name">{t.label}</div>
      </div>
    {/each}
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    background: transparent !important;
    overflow: hidden;
    font: 14px/1.4 "Segoe UI", system-ui, sans-serif;
    color: #ece4cf;
    user-select: none;
  }
  .screen {
    position: fixed;
    inset: 0;
  }
  .screen.interactive {
    background: rgba(0, 0, 0, 0.25);
  }
  .hud {
    position: absolute;
    min-width: 300px;
    max-width: 360px;
    background: rgba(14, 13, 11, 0.82);
    border: 1px solid rgba(201, 169, 91, 0.45);
    border-radius: 10px;
    padding: 10px 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    text-shadow: 0 1px 2px #000;
  }
  .tl {
    top: 16px;
    left: 16px;
    transform-origin: top left;
  }
  .tr {
    top: 16px;
    right: 16px;
    transform-origin: top right;
  }
  .bl {
    bottom: 16px;
    left: 16px;
    transform-origin: bottom left;
  }
  .br {
    bottom: 16px;
    right: 16px;
    transform-origin: bottom right;
  }
  .title {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 6px;
  }
  .brand {
    font-family: "Cinzel", Georgia, serif;
    color: #c9a95b;
    letter-spacing: 0.06em;
  }
  .muted {
    color: #a59c86;
    font-size: 12px;
  }
  .block {
    border-top: 1px solid rgba(201, 169, 91, 0.2);
    padding: 6px 0;
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #c9a95b;
  }
  .value {
    font-size: 15px;
    font-weight: 600;
  }
  .sync {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #a59c86;
    margin: -2px 0 4px;
  }
  .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #6fbf73;
    box-shadow: 0 0 6px #6fbf73;
  }
  .label {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: rgba(201, 169, 91, 0.2);
    margin: 5px 0 4px;
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: #c9a95b;
  }
  .left span {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .boss {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .arrow {
    flex-shrink: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(201, 169, 91, 0.6);
    border-radius: 50%;
    color: #f3d27a;
  }
  .warn {
    display: flex;
    align-items: center;
    gap: 5px;
    color: #f0c27a;
    font-size: 12px;
    margin-top: 3px;
  }
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
  }
  .stats strong {
    color: #f3d27a;
  }
  .total {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-top: 3px;
  }
  .left {
    display: flex;
    gap: 2px 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .done {
    color: #6fbf73;
  }
  .deaths {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }
  .settings {
    border-top: 1px solid rgba(201, 169, 91, 0.3);
    margin-top: 6px;
    padding-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .settings label {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
  }
  .corners {
    display: flex;
    gap: 4px;
  }
  .corners button,
  .mini {
    background: #26221a;
    border: 1px solid #4a4232;
    color: #ece4cf;
    border-radius: 6px;
    padding: 2px 8px;
    font: inherit;
    cursor: pointer;
  }
  .corners button.active {
    border-color: #c9a95b;
    color: #c9a95b;
  }
  .mini {
    margin-left: auto;
  }
  .wide {
    margin: 0;
  }
  .hint {
    margin-top: 6px;
    font-size: 11px;
    color: #a59c86;
  }
  .toasts {
    position: absolute;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }
  .toasts.tr,
  .toasts.br {
    right: 16px;
  }
  .toasts.tl,
  .toasts.bl {
    left: 16px;
  }
  .toasts.tr,
  .toasts.tl {
    top: auto;
    bottom: 80px;
  }
  .toasts.br,
  .toasts.bl {
    bottom: auto;
    top: 80px;
  }
  .toast {
    background: rgba(14, 13, 11, 0.88);
    border: 1px solid #7d6a3c;
    border-left: 3px solid #c9a95b;
    padding: 10px 16px;
    border-radius: 6px;
    min-width: 300px;
    animation: slide 0.3s ease-out;
  }
  .kicker {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #c9a95b;
  }
  .toast-name {
    font-family: "Cinzel", Georgia, serif;
    font-size: 18px;
  }
  @keyframes slide {
    from {
      transform: translateY(10px);
      opacity: 0;
    }
  }
</style>
