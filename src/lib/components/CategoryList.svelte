<script lang="ts">
  import { db, tracker } from "$lib/tracker.svelte";
  import { categoryName } from "$lib/labels";
  import { gamedata } from "$lib/gamedata.svelte";

  let { category }: { category: string } = $props();

  let filter = $state<"all" | "todo" | "done">("todo");
  let search = $state("");

  const entries = $derived(db.entries.filter((e) => e.category === category));
  const doneCount = $derived(entries.filter((e) => tracker.completed.has(e.id)).length);

  const groups = $derived.by(() => {
    const q = search.trim().toLowerCase();
    const byRegion = new Map<string, typeof entries>();
    for (const e of entries) {
      const done = tracker.completed.has(e.id);
      if ((filter === "todo" && done) || (filter === "done" && !done)) continue;
      const name = gamedata.name(e);
      const region = gamedata.region(e) ?? "Autre";
      if (q && !name.toLowerCase().includes(q) && !e.name.toLowerCase().includes(q) && !region.toLowerCase().includes(q)) continue;
      byRegion.set(region, [...(byRegion.get(region) ?? []), e]);
    }
    return [...byRegion.entries()].sort(([a], [b]) => a.localeCompare(b));
  });
</script>

<header>
  <h2>{categoryName(category)}</h2>
  <span class="muted">{doneCount} / {entries.length}</span>
</header>

<div class="toolbar">
  <input type="search" placeholder="Rechercher…" bind:value={search} />
  <div class="segmented">
    <button class:active={filter === "todo"} onclick={() => (filter = "todo")}>À faire</button>
    <button class:active={filter === "done"} onclick={() => (filter = "done")}>Fait</button>
    <button class:active={filter === "all"} onclick={() => (filter = "all")}>Tout</button>
  </div>
</div>

{#each groups as [region, items] (region)}
  <section>
    <h3>{region} <span class="muted">({items.length})</span></h3>
    <ul>
      {#each items as e (e.id)}
        {@const done = tracker.completed.has(e.id)}
        <li class:done>
          <span class="check">{done ? "✓" : ""}</span>
          <span>{gamedata.name(e)}</span>
        </li>
      {/each}
    </ul>
  </section>
{:else}
  <p class="muted">Rien à afficher avec ces filtres.</p>
{/each}

<style>
  header {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  h2 {
    margin: 0 0 12px;
  }
  .toolbar {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 16px;
    position: sticky;
    top: 0;
    background: var(--bg);
    padding: 8px 0;
    z-index: 1;
  }
  input {
    flex: 1;
    min-width: 180px;
  }
  h3 {
    font-size: 14px;
    color: var(--gold);
    margin: 18px 0 6px;
    font-weight: 600;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    columns: 2 280px;
    column-gap: 24px;
  }
  li {
    display: flex;
    gap: 10px;
    padding: 4px 0;
    break-inside: avoid;
  }
  li.done {
    color: var(--muted);
  }
  .check {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    display: grid;
    place-items: center;
    font-size: 12px;
    color: var(--bg);
    margin-top: 3px;
  }
  li.done .check {
    background: var(--gold);
    border-color: var(--gold);
  }
</style>
