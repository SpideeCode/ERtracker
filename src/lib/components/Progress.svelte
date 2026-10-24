<script lang="ts">
  // Progression: one page, tabs for bosses, items, graces, NPC quests and small collections.
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { ui, type ProgressTab } from "$lib/ui.svelte";
  import BossList from "./BossList.svelte";
  import ItemList from "./ItemList.svelte";
  import CategoryList from "./CategoryList.svelte";
  import Quests from "./Quests.svelte";
  import ProgressOverview from "./ProgressOverview.svelte";
  import { achievements } from "$lib/achievements.svelte";

  let { tab }: { tab: ProgressTab } = $props();

  const COLLECTIONS = ["great_rune", "cookbook", "bell_bearing", "whetblade", "map_reveal"];
  let collection = $state("great_rune");

  const count = (category: string) => {
    const list = db.entries.filter((e) => e.category === category);
    return `${list.filter((e) => tracker.completed.has(e.id)).length}/${list.length}`;
  };
  const items = $derived(`${[...gamedata.items.keys()].filter((id) => tracker.completed.has(id)).length}/${gamedata.items.size}`);

  const tabs: { id: ProgressTab; label: string; count: () => string }[] = [
    { id: "overview", label: "Aperçu", count: () => achievements.rank.title },
    { id: "bosses", label: "Boss", count: () => count("boss") },
    { id: "items", label: "Objets", count: () => items },
    { id: "graces", label: "Grâces", count: () => count("grace") },
    { id: "quests", label: "Quêtes PNJ", count: () => `${db.quests.length}` },
    { id: "collections", label: "Collections", count: () => "" },
  ];
</script>

<div class="tabs">
  {#each tabs as t (t.id)}
    <button class:active={tab === t.id} onclick={() => ui.progress(t.id)}>
      {t.label}
      {#if t.count()}<span class="count">{t.count()}</span>{/if}
    </button>
  {/each}
</div>

{#if tab === "overview"}
  <ProgressOverview />
{:else if tab === "bosses"}
  <BossList />
{:else if tab === "items"}
  <ItemList />
{:else if tab === "graces"}
  {#key "grace"}<CategoryList category="grace" />{/key}
{:else if tab === "quests"}
  <Quests />
{:else}
  <div class="sub">
    {#each COLLECTIONS as c}
      {@const cat = db.categories.find((x) => x.id === c)}
      <button class="chip" class:active={collection === c} onclick={() => (collection = c)}>
        {cat?.name.fr ?? c} <span class="muted">{count(c)}</span>
      </button>
    {/each}
  </div>
  {#key collection}<CategoryList category={collection} />{/key}
{/if}

<style>
  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 18px;
  }
  .tabs button {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-dim);
    padding: 8px 14px;
    font: inherit;
    cursor: pointer;
    display: flex;
    gap: 8px;
    align-items: baseline;
  }
  .tabs button:hover {
    color: var(--text);
  }
  .tabs button.active {
    color: var(--gold);
    border-bottom-color: var(--gold);
  }
  .count {
    font-size: 12px;
    color: var(--muted);
  }
  .sub {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 14px;
  }
  .chip {
    background: var(--surface);
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    border-radius: 14px;
    padding: 4px 12px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .chip.active {
    border-color: var(--gold);
    color: var(--gold);
  }
</style>
