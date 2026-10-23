<script lang="ts">
  import { onMount } from "svelte";
  import { db, tracker } from "$lib/tracker.svelte";
  import { achievement, ago, playtime } from "$lib/labels";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import CategoryList from "$lib/components/CategoryList.svelte";
  import Quests from "$lib/components/Quests.svelte";
  import MapView from "$lib/components/MapView.svelte";
  import BossList from "$lib/components/BossList.svelte";
  import Sheet from "$lib/components/Sheet.svelte";
  import ItemList from "$lib/components/ItemList.svelte";
  import GlobalSearch from "$lib/components/GlobalSearch.svelte";
  import Builds from "$lib/components/Builds.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { ui } from "$lib/ui.svelte";

  let now = $state(Date.now());
  const view = $derived(ui.view);

  onMount(() => {
    tracker.start();
    mapdata.init();
    const timer = setInterval(() => (now = Date.now()), 1000);
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && (ui.sheet = null);
    window.addEventListener("keydown", onKey);
    return () => {
      clearInterval(timer);
      window.removeEventListener("keydown", onKey);
    };
  });

  const snapshot = $derived(tracker.snapshot);
  const fresh = $derived(snapshot ? now - snapshot.loaded_at < 15 * 60 * 1000 : false);

  const counts = $derived(
    Object.fromEntries(
      db.categories.map((c) => {
        const entries = db.entries.filter((e) => e.category === c.id);
        return [c.id, `${entries.filter((e) => tracker.completed.has(e.id)).length}/${entries.length}`];
      }),
    ),
  );

  const itemCount = $derived(
    `${[...gamedata.items.keys()].filter((id) => tracker.completed.has(id)).length}/${gamedata.items.size}`,
  );

  function folderLabel(path: string) {
    const parts = path.split(/[\\/]/);
    return `${parts.at(-2)} · ${parts.at(-1)}`;
  }

  const open = (id: string) => (ui.view = id === "boss" ? { kind: "bosses" } : { kind: "category", id });
  const isActive = (id: string) => (view.kind === "category" && view.id === id) || (view.kind === "bosses" && id === "boss");
</script>

<div class="app">
  <header>
    <div class="brand">ERtracker</div>

    {#if snapshot}
      <select
        value={snapshot.slot}
        onchange={(e) => tracker.select(null, Number(e.currentTarget.value))}
        title="Personnage"
      >
        {#each snapshot.characters as c (c.slot)}
          <option value={c.slot}>{c.name} · niv. {c.level} · {playtime(c.seconds_played)}</option>
        {/each}
      </select>
    {/if}

    {#if tracker.saves.length > 1}
      <select
        value={snapshot?.save_path}
        onchange={(e) => tracker.select(e.currentTarget.value, null)}
        title="Fichier de sauvegarde"
      >
        {#each tracker.saves as s (s.path)}
          <option value={s.path}>{folderLabel(s.path)}</option>
        {/each}
      </select>
    {/if}

    <GlobalSearch />

    {#if mapdata.running && mapdata.status?.manifest}
      <div class="sync" title={mapdata.step()}>
        Mise à jour des données du jeu… {mapdata.percent} %
        <div class="sync-bar"><div style="width: {mapdata.percent}%"></div></div>
      </div>
    {/if}

    <div class="live" class:on={fresh} title={snapshot?.save_path}>
      <span class="dot"></span>
      {#if snapshot}
        LIVE · sauvegarde lue {ago(snapshot.loaded_at, now)}
      {:else}
        En attente…
      {/if}
    </div>
  </header>

  <nav>
    <div class="group">Suivi</div>
    <button class:active={view.kind === "dashboard"} onclick={() => (ui.view = { kind: "dashboard" })}>
      <span>Tableau de bord</span>
    </button>
    <button class:active={view.kind === "map"} onclick={() => (ui.view = { kind: "map" })}>
      <span>Carte</span>
    </button>

    <button class:active={view.kind === "builds"} onclick={() => (ui.view = { kind: "builds" })}>
      <span>Conseiller de build</span>
    </button>

    <div class="group">Progression</div>
    {#each db.categories as c (c.id)}
      <button class:active={isActive(c.id)} onclick={() => open(c.id)}>
        <span>{c.name.fr}</span>
        <span class="count">{counts[c.id]}</span>
      </button>
    {/each}
    <button class:active={view.kind === "items"} onclick={() => (ui.view = { kind: "items" })}>
      <span>Objets</span>
      {#if gamedata.items.size}<span class="count">{itemCount}</span>{/if}
    </button>
    <button class:active={view.kind === "quests"} onclick={() => (ui.view = { kind: "quests" })}>
      <span>Quêtes PNJ</span>
    </button>
  </nav>

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
      {:else if view.kind === "dashboard"}
        <Dashboard {now} {open} />
      {:else if view.kind === "category"}
        {#key view.id}
          <CategoryList category={view.id} />
        {/key}
      {:else if view.kind === "bosses"}
        <BossList />
      {:else if view.kind === "items"}
        <ItemList />
      {:else if view.kind === "builds"}
        <Builds />
      {:else if view.kind === "quests"}
        <Quests />
      {:else if view.kind === "map"}
        <MapView />
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

  <div class="toasts">
    {#each tracker.toasts as t (t.key)}
      <div class="toast">
        <div class="kicker">{achievement(t.category)}</div>
        <div>{t.label}</div>
      </div>
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
    color-scheme: dark;
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
    grid-template-columns: 200px minmax(0, 1fr);
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
  .brand {
    font-family: Georgia, "Times New Roman", serif;
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
  nav {
    grid-area: nav;
    border-right: 1px solid var(--border);
    padding: 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
  }
  .group {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    padding: 12px 10px 4px;
  }
  .group:first-child {
    padding-top: 0;
  }
  nav button {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-dim);
    padding: 7px 10px;
    border-radius: 6px;
    font: inherit;
    cursor: pointer;
  }
  nav button:hover {
    background: var(--surface);
  }
  nav button.active {
    background: var(--surface-2);
    color: var(--gold);
  }
  .count {
    font-size: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  nav button.active .count {
    color: var(--gold-dim);
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
