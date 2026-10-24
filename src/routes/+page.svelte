<script lang="ts">
  import { onMount } from "svelte";
  import { tracker } from "$lib/tracker.svelte";
  import { achievement, ago, playtime } from "$lib/labels";
  import Now from "$lib/components/Now.svelte";
  import MapView from "$lib/components/MapView.svelte";
  import Sheet from "$lib/components/Sheet.svelte";
  import GlobalSearch from "$lib/components/GlobalSearch.svelte";
  import Notifications from "$lib/components/Notifications.svelte";
  import Welcome from "$lib/components/Welcome.svelte";
  import Builds from "$lib/components/Builds.svelte";
  import Stats from "$lib/components/Stats.svelte";
  import Explore from "$lib/components/Explore.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Progress from "$lib/components/Progress.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { prefs } from "$lib/prefs.svelte";
  import { ui } from "$lib/ui.svelte";

  let now = $state(Date.now());
  let prefsLoaded = $state(false);
  const view = $derived(ui.view);

  onMount(() => {
    prefs.load();
    prefsLoaded = true;
    tracker.start();
    mapdata.init();
    const timer = setInterval(() => (now = Date.now()), 1000);
    const onKey = (e: KeyboardEvent) => {
      const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement;
      if (e.key === "Escape") ui.sheet = null;
      else if (e.altKey && e.key === "ArrowLeft") {
        e.preventDefault();
        ui.back();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        document.getElementById("global-search")?.focus();
      } else if (!typing && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === "m") ui.go({ kind: "map" });
    };
    window.addEventListener("keydown", onKey);
    return () => {
      clearInterval(timer);
      window.removeEventListener("keydown", onKey);
    };
  });

  const snapshot = $derived(tracker.snapshot);
  const fresh = $derived(snapshot ? now - snapshot.loaded_at < 15 * 60 * 1000 : false);
</script>

<div class="app" class:collapsed={prefs.sidebarCollapsed}>
  <header>
    <div class="brand">ERtracker</div>

    {#if snapshot}
      <select
        class="character"
        value={snapshot.slot}
        onchange={(e) => tracker.select(null, Number(e.currentTarget.value))}
        title="Personnage suivi"
      >
        {#each snapshot.characters as c (c.slot)}
          <option value={c.slot}>{c.name} · niv. {c.level} · {playtime(c.seconds_played)}</option>
        {/each}
      </select>
    {/if}

    <GlobalSearch />

    {#if mapdata.running && mapdata.status?.manifest}
      <div class="sync" title={mapdata.step()}>
        Mise à jour des données… {mapdata.percent} %
        <div class="sync-bar"><div style="width: {mapdata.percent}%"></div></div>
      </div>
    {/if}

    <div class="live" class:on={fresh} title={snapshot?.save_path}>
      <span class="dot"></span>
      {#if snapshot}
        LIVE · {ago(snapshot.loaded_at, now)}
      {:else}
        En attente…
      {/if}
    </div>
    <Notifications {now} />
  </header>

  <Sidebar />

  <div class="content">
    <main class:full={view.kind === "map"}>
      {#if tracker.error}
        <div class="error">Impossible de lire la sauvegarde : {tracker.error}</div>
      {/if}

      {#if !snapshot && !tracker.error}
        <p class="muted">
          Recherche de ta sauvegarde Elden Ring dans %APPDATA%\EldenRing… Si rien n'apparaît, lance le jeu une fois
          pour créer un personnage.
        </p>
      {:else if view.kind === "now"}
        <Now {now} />
      {:else if view.kind === "map"}
        <MapView />
      {:else if view.kind === "progress"}
        <Progress tab={view.tab} />
      {:else if view.kind === "explore"}
        <Explore />
      {:else if view.kind === "journey"}
        <Stats />
      {:else if view.kind === "builds"}
        <Builds />
      {:else if view.kind === "settings"}
        <Settings tab={view.tab} />
      {/if}
    </main>

    {#if ui.sheet}
      <div class="drawer">
        {#key ui.sheet}
          <Sheet id={ui.sheet} />
        {/key}
      </div>
    {/if}
  </div>

  {#if prefsLoaded && !prefs.welcomed}<Welcome />{/if}

  <div class="toasts">
    {#each tracker.toasts as t (t.key)}
      <button class="toast" onclick={() => ui.open(t.id)} title="Ouvrir la fiche">
        <div class="kicker">{achievement(t.category)}</div>
        <div>{t.label}</div>
      </button>
    {/each}
  </div>
</div>

<style>
  :global(:root) {
    --bg: #0e0d0b;
    --surface: #17150f;
    --surface-2: #26221a;
    --border: #2b261c;
    --border-strong: #4a4232;
    --text: #e8e0cc;
    --text-dim: #a59c86;
    --muted: #8a8170;
    --gold: #c9a95b;
    --gold-dim: #7d6a3c;
    --danger: #d16a5a;
    --font-title: "Cinzel", Georgia, serif;
    color-scheme: dark;
  }
  :global(main h2) {
    font-family: var(--font-title);
    font-weight: 600;
    letter-spacing: 0.02em;
  }
  /* Shared buttons: primary for the main action of a card, ghost for the others. */
  :global(button.primary),
  :global(button.ghost) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 7px;
    padding: 7px 14px;
    font: inherit;
    cursor: pointer;
  }
  :global(button.primary) {
    background: var(--gold);
    border: 1px solid var(--gold);
    color: #1a1508;
    font-weight: 600;
  }
  :global(button.primary:hover) {
    filter: brightness(1.08);
  }
  :global(button.ghost) {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
  }
  :global(button.ghost:hover) {
    border-color: var(--gold-dim);
  }
  :global(button.primary:disabled),
  :global(button.ghost:disabled) {
    opacity: 0.45;
    cursor: default;
    filter: none;
  }
  :global(.with-icon) {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  :global(.lock) {
    display: inline-flex;
    vertical-align: middle;
    margin-left: 6px;
    color: #d39b45;
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--text);
    font: 14px/1.5 "Segoe UI", system-ui, sans-serif;
  }
  :global(.muted) {
    color: var(--muted);
  }
  :global(input[type="search"]),
  :global(select) {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 6px 10px;
    font: inherit;
  }
  :global(.segmented) {
    display: flex;
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    overflow: hidden;
  }
  :global(.segmented button) {
    background: var(--surface);
    color: var(--text-dim);
    border: none;
    padding: 6px 12px;
    font: inherit;
    cursor: pointer;
  }
  :global(.segmented button.active) {
    background: var(--surface-2);
    color: var(--gold);
  }

  .app {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: 52px minmax(0, 1fr);
    grid-template-areas: "header header" "nav content";
    height: 100vh;
  }
  header {
    grid-area: header;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .character {
    max-width: 280px;
  }
  .brand {
    font-family: var(--font-title);
    font-size: 18px;
    letter-spacing: 0.08em;
    color: var(--gold);
    margin-right: 12px;
  }
  .live {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    font-size: 13px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--muted);
  }
  .live.on .dot {
    background: #6fbf73;
    box-shadow: 0 0 8px #6fbf73;
  }
  .sync {
    font-size: 12px;
    color: var(--gold);
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 200px;
  }
  .sync-bar {
    height: 3px;
    background: var(--surface-2);
    border-radius: 2px;
    overflow: hidden;
  }
  .sync-bar div {
    height: 100%;
    background: var(--gold);
    transition: width 0.2s;
  }
  .content {
    grid-area: content;
    position: relative;
    min-height: 0;
    overflow: hidden;
  }
  main {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    padding: 20px 28px;
  }
  main.full {
    padding: 0;
    overflow: hidden;
  }
  .error {
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: 10px 12px;
    border-radius: 6px;
    margin-bottom: 16px;
  }
  .drawer {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(420px, 92%);
    z-index: 1500;
    box-shadow: -12px 0 32px rgba(0, 0, 0, 0.55);
    animation: drawer 0.18s ease-out;
  }
  @keyframes drawer {
    from {
      transform: translateX(24px);
      opacity: 0;
    }
  }
  .toasts {
    z-index: 2000;
    position: fixed;
    right: 16px;
    bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }
  .toast {
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-left: 3px solid var(--gold);
    padding: 10px 14px;
    border-radius: 6px;
    min-width: 240px;
    animation: slide 0.25s ease-out;
    pointer-events: auto;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .kicker {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--gold);
  }
  @keyframes slide {
    from {
      transform: translateY(8px);
      opacity: 0;
    }
  }
</style>
