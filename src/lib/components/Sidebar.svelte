<script lang="ts">
  import { Compass, Map, Trophy, Telescope, BookOpen, Swords, Settings, PanelLeftClose, PanelLeftOpen } from "@lucide/svelte";
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { prefs } from "$lib/prefs.svelte";
  import { ui, type View } from "$lib/ui.svelte";

  const view = $derived(ui.view);

  // Overall progress shown next to "Progression": bosses, graces and item pickups.
  const progress = $derived.by(() => {
    const ids = [
      ...db.entries.filter((e) => e.category === "boss" || e.category === "grace").map((e) => e.id),
      ...gamedata.items.keys(),
    ];
    const done = ids.filter((id) => tracker.completed.has(id)).length;
    return ids.length ? Math.round((done / ids.length) * 100) : 0;
  });

  const items: { kind: View["kind"]; label: string; icon: typeof Compass; go: View }[] = [
    { kind: "now", label: "Maintenant", icon: Compass, go: { kind: "now" } },
    { kind: "map", label: "Carte", icon: Map, go: { kind: "map" } },
    { kind: "progress", label: "Progression", icon: Trophy, go: { kind: "progress", tab: "bosses" } },
    { kind: "explore", label: "Explorer", icon: Telescope, go: { kind: "explore" } },
    { kind: "journey", label: "Mon aventure", icon: BookOpen, go: { kind: "journey" } },
    { kind: "builds", label: "Builds", icon: Swords, go: { kind: "builds" } },
  ];

  function toggle() {
    prefs.sidebarCollapsed = !prefs.sidebarCollapsed;
    prefs.save();
  }
</script>

<nav class:collapsed={prefs.sidebarCollapsed}>
  {#each items as item (item.kind)}
    {@const Icon = item.icon}
    <button class:active={view.kind === item.kind} onclick={() => ui.go(item.go)} title={item.label}>
      <Icon size={18} strokeWidth={1.75} />
      <span class="label">{item.label}</span>
      {#if item.kind === "progress" && gamedata.loaded}<span class="count">{progress} %</span>{/if}
    </button>
  {/each}

  <div class="spacer"></div>

  <button class:active={view.kind === "settings"} onclick={() => ui.go({ kind: "settings", tab: "general" })} title="Réglages et outils">
    <Settings size={18} strokeWidth={1.75} />
    <span class="label">Réglages</span>
  </button>
  <button class="collapse" onclick={toggle} title={prefs.sidebarCollapsed ? "Déplier le menu" : "Replier le menu"}>
    {#if prefs.sidebarCollapsed}<PanelLeftOpen size={18} strokeWidth={1.75} />{:else}<PanelLeftClose size={18} strokeWidth={1.75} />{/if}
    <span class="label">Replier</span>
  </button>
</nav>

<style>
  nav {
    grid-area: nav;
    width: 190px;
    border-right: 1px solid var(--border);
    padding: 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
    transition: width 0.15s;
  }
  nav.collapsed {
    width: 52px;
  }
  button {
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-dim);
    padding: 9px 10px;
    border-radius: 8px;
    font: inherit;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
  }
  button :global(svg) {
    flex-shrink: 0;
  }
  button:hover {
    background: var(--surface);
    color: var(--text);
  }
  button.active {
    background: var(--surface-2);
    color: var(--gold);
  }
  .label {
    flex: 1;
  }
  .collapsed .label,
  .collapsed .count {
    display: none;
  }
  .count {
    font-size: 12px;
    color: var(--muted);
  }
  .spacer {
    flex: 1;
  }
  .collapse {
    color: var(--muted);
  }
</style>
