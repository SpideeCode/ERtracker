<script lang="ts">
  import { onMount } from "svelte";
  import { db, tracker } from "$lib/tracker.svelte";
  import { achievement, ago, playtime } from "$lib/labels";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import CategoryList from "$lib/components/CategoryList.svelte";
  import Quests from "$lib/components/Quests.svelte";
  import MapView from "$lib/components/MapView.svelte";

  type View = { kind: "dashboard" } | { kind: "category"; id: string } | { kind: "quests" } | { kind: "map" };

  let view = $state<View>({ kind: "dashboard" });
  let now = $state(Date.now());

  onMount(() => {
    tracker.start();
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const snapshot = $derived(tracker.snapshot);
  const fresh = $derived(snapshot ? now - snapshot.loaded_at < 15 * 60 * 1000 : false);

  function folderLabel(path: string) {
    const parts = path.split(/[\\/]/);
    return `${parts.at(-2)} · ${parts.at(-1)}`;
  }

  const open = (id: string) => (view = { kind: "category", id });
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

    <div class="live" class:on={fresh} title={snapshot?.save_path}>
      <span class="dot"></span>
      {#if snapshot}
        LIVE · lu {ago(snapshot.loaded_at, now)}
      {:else}
        En attente…
      {/if}
    </div>
  </header>

  <nav>
    <button class:active={view.kind === "dashboard"} onclick={() => (view = { kind: "dashboard" })}>Tableau de bord</button>
    <button class:active={view.kind === "map"} onclick={() => (view = { kind: "map" })}>Carte</button>
    <div class="sep"></div>
    {#each db.categories as c (c.id)}
      <button class:active={view.kind === "category" && view.id === c.id} onclick={() => open(c.id)}>
        {c.name.fr}
      </button>
    {/each}
    <button class:active={view.kind === "quests"} onclick={() => (view = { kind: "quests" })}>Quêtes PNJ</button>
  </nav>

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
    {:else if view.kind === "quests"}
      <Quests />
    {:else if view.kind === "map"}
      <MapView />
    {/if}
  </main>

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
    grid-template-columns: 200px 1fr;
    grid-template-rows: auto 1fr;
    height: 100vh;
  }
  header {
    grid-column: 1 / -1;
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
  nav {
    border-right: 1px solid var(--border);
    padding: 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
  }
  nav button {
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
  .sep {
    height: 1px;
    background: var(--border);
    margin: 8px 4px;
  }
  main {
    overflow-y: auto;
    padding: 20px 28px;
    position: relative;
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
  .toasts {
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
