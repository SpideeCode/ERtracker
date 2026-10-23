<script lang="ts">
  import { entriesById, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import RouteCard from "./RouteCard.svelte";

  let { id }: { id: string } = $props();

  const entry = $derived(entriesById.get(id)!);
  const pos = $derived(gamedata.positions.get(id));
  const discovered = $derived(tracker.completed.has(id));
  const name = $derived(gamedata.name(entry));
</script>

<header>
  <div class="badges">
    <span class="badge" class:ok={discovered}>{discovered ? "Découverte ✓" : "Pas encore découverte"}</span>
    <span class="badge">Site de grâce</span>
  </div>
  <h2>{name}</h2>
  {#if name !== entry.name}<div class="muted small">{entry.name}</div>{/if}
  {#if gamedata.region(entry)}<div class="meta"><span>{gamedata.region(entry)}</span></div>{/if}
</header>

{#if discovered}
  <p class="muted">Tu peux t'y téléporter depuis n'importe quelle grâce.</p>
{:else if pos}
  <RouteCard {id} {pos} goal="la grâce" howTo={entry.name} />
{/if}
