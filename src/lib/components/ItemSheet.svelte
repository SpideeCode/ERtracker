<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { categoryById, itemCategory, itemLabel, pickupCategory } from "$lib/items";
  import RouteCard from "./RouteCard.svelte";
  import EquipmentDetails from "./EquipmentDetails.svelte";

  let { id }: { id: string } = $props();

  const pickup = $derived(gamedata.items.get(id));
  const done = $derived(tracker.completed.has(id));
  const first = $derived(pickup?.items[0]);
  const category = $derived(pickup ? categoryById.get(pickupCategory(pickup)) : undefined);

  const KINDS: Record<number, string> = { 1: "Objet", 2: "Arme", 3: "Armure", 4: "Talisman", 5: "Cendre de guerre" };

  const wiki = $derived(
    first ? `https://eldenring.wiki.fextralife.com/${encodeURIComponent(first.name.en).replace(/%20/g, "+")}` : "",
  );
  const youtube = $derived(
    first ? `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring ${first.name.en} location`)}` : "",
  );
</script>

{#if !pickup || !first}
  <p class="muted">Objet introuvable dans les données extraites.</p>
{:else}
  <header>
    <div class="badges">
      <span class="badge" class:ok={done}>{done ? "Récupéré ✓" : "À récupérer"}</span>
      {#if category}<span class="badge" style="border-color: {category.color}; color: {category.color}">{category.name}</span>{/if}
    </div>
    <h2>{itemLabel(first)}{first.count > 1 ? ` ×${first.count}` : ""}</h2>
    {#if first.name.fr && first.name.fr !== first.name.en}<div class="muted small">{first.name.en}</div>{/if}
  </header>

  {#if pickup.items.length > 1}
    <section class="card">
      <h3>Contenu ({pickup.items.length} objets)</h3>
      <ul class="contents">
        {#each pickup.items as item}
          <li>
            <span>{itemLabel(item)}{item.count > 1 ? ` ×${item.count}` : ""}</span>
            <span class="muted small">{categoryById.get(itemCategory(item))?.name ?? KINDS[item.cat]}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#each pickup.items.slice(0, 4) as item}
    <section class="card">
      <h3>{pickup.items.length > 1 ? itemLabel(item) : "Détails"}</h3>
      <EquipmentDetails cat={item.cat} id={item.id} />
    </section>
  {/each}

  <RouteCard {id} pos={{ ...pickup, kind: "item" }} goal="l'objet" />

  <section class="links">
    <button onclick={() => openUrl(wiki)}>Wiki ↗</button>
    <button onclick={() => openUrl(youtube)}>Vidéos ↗</button>
  </section>
{/if}

<style>
  .contents {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .contents li {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 4px 0;
    border-bottom: 1px solid var(--border);
  }
</style>
