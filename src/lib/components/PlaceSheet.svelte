<script lang="ts">
  // Sheet for a named place of the map (dungeon entrance): what is inside and how to get there.
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { pickupTitle, categoryById, pickupCategory } from "$lib/items";
  import { ui } from "$lib/ui.svelte";
  import RouteCard from "./RouteCard.svelte";
  import { PLACE_KINDS, placeContents } from "$lib/places";

  let { id }: { id: string } = $props();

  const place = $derived(gamedata.places[Number(id.split(":")[1])]);
  const pos = $derived(gamedata.positions.get(id));
  const name = $derived(place ? (place.name.fr ?? place.name.en) : "");
  const contents = $derived(
    place ? placeContents(place, db.entries, gamedata.positions, gamedata.items, tracker.completed) : null,
  );
  const block = $derived(contents?.block ?? null);
  const bosses = $derived(contents?.bosses ?? []);
  const graces = $derived(contents?.graces ?? []);
  const items = $derived(contents?.items ?? []);
  const itemsDone = $derived(items.filter(([iid]) => tracker.completed.has(iid)).length);
  const kind = $derived(place ? PLACE_KINDS[place.icon]?.[0] : undefined);

  const youtube = $derived(
    place ? `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring ${place.name.en} walkthrough`)}` : "",
  );
</script>

{#if !place}
  <p class="muted">Lieu introuvable.</p>
{:else}
  <header>
    <div class="badges">
      <span class="badge">{kind ? kind.charAt(0).toUpperCase() + kind.slice(1) : "Lieu"}</span>
      {#if contents}<span class="badge" class:ok={contents.done}>{contents.done ? "Terminé ✓" : contents.visited ? "Commencé" : "Pas encore visité"}</span>{/if}
    </div>
    <h2>{name}</h2>
    {#if place.name.fr && place.name.fr !== place.name.en}<div class="muted small">{place.name.en}</div>{/if}
  </header>

  {#if place}
    <section class="card">
      <h3>{block ? "À l'intérieur" : "Autour de l'entrée"}</h3>
      {#if bosses.length}
        <ul class="list">
          {#each bosses as b (b.id)}
            <li>
              <span class="box" class:checked={tracker.completed.has(b.id)}>{tracker.completed.has(b.id) ? "✓" : ""}</span>
              <button class="link" onclick={() => (ui.sheet = b.id)}>{gamedata.name(b)}</button>
              <span class="muted small">boss</span>
            </li>
          {/each}
        </ul>
      {/if}
      {#each graces as g (g.id)}
        <p class="muted small">Grâce : {gamedata.name(g)} {tracker.completed.has(g.id) ? "(découverte ✓)" : "(pas encore découverte)"}</p>
      {/each}
      {#if items.length}
        <h4>Objets ({itemsDone} / {items.length} ramassés)</h4>
        <ul class="list">
          {#each items as [iid, p] (iid)}
            <li>
              <span class="box" class:checked={tracker.completed.has(iid)}>{tracker.completed.has(iid) ? "✓" : ""}</span>
              <button class="link" onclick={() => (ui.sheet = iid)}>{pickupTitle(p)}</button>
              <span class="muted small">{categoryById.get(pickupCategory(p))?.name ?? ""}</span>
            </li>
          {/each}
        </ul>
      {/if}
      {#if !bosses.length && !items.length}
        <p class="muted small">Aucun boss ni objet suivi dans ce lieu.</p>
      {/if}
    </section>
  {/if}

  {#if pos}
    <RouteCard {id} {pos} goal="l'entrée" howTo={place.name.en} />
  {/if}

  <section class="links">
    <button onclick={() => openUrl(youtube)}>Vidéos ↗</button>
  </section>
{/if}

<style>
  h4 {
    margin: 12px 0 4px;
    font-size: 12px;
    color: var(--muted);
  }
  .list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .list li {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 3px 0;
  }
  .list .small {
    margin-left: auto;
  }
  .box {
    width: 15px;
    height: 15px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    display: grid;
    place-items: center;
    font-size: 10px;
    color: var(--bg);
    flex-shrink: 0;
  }
  .box.checked {
    background: #5fae6a;
    border-color: #5fae6a;
  }
  .link {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .link:hover {
    color: var(--gold);
  }
</style>
