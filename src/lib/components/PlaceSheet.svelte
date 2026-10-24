<script lang="ts">
  // Sheet for a named place of the map (dungeon entrance): what is inside and how to get there.
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { pickupTitle, categoryById, pickupCategory } from "$lib/items";
  import { ui } from "$lib/ui.svelte";
  import RouteCard from "./RouteCard.svelte";

  let { id }: { id: string } = $props();

  /** Map icon -> kind of place. */
  const KINDS: Record<number, string> = {
    4: "Catacombes",
    13: "Grotte",
    14: "Tunnel",
    15: "Puits",
    16: "Tombe de héros",
    23: "Tour divine",
    230: "Catacombes",
    231: "Geôle",
    232: "Forge en ruine",
    233: "Grotte",
    234: "Grotte",
  };

  const place = $derived(gamedata.places[Number(id.split(":")[1])]);
  const pos = $derived(gamedata.positions.get(id));
  const name = $derived(place ? (place.name.fr ?? place.name.en) : "");
  const isOpen = (block: string) => [60, 61, 12].includes(Number(block.slice(1, 3)));

  // The dungeon's own map block: the place's block, or the block of the grace named like it inside.
  const block = $derived.by(() => {
    if (!place) return null;
    if (!isOpen(place.block)) return place.block;
    const grace = db.entries.find((e) => e.category === "grace" && e.name === place.name.en);
    const gpos = grace ? gamedata.positions.get(grace.id) : undefined;
    return gpos && !isOpen(gpos.block) ? gpos.block : null;
  });

  // Caves drawn inside an open-world tile have no block of their own: take what is around the entrance.
  const NEARBY = 80;
  const inside = (b: string | undefined, x: number, y: number, map: string) =>
    block ? b === block || !!b?.startsWith(block) : !!place && map === place.map && Math.hypot(x - place.x, y - place.y) < NEARBY;
  const atPlace = (id: string) => {
    const p = gamedata.positions.get(id);
    return !!p && inside(p.block, p.x, p.y, p.map);
  };
  const bosses = $derived(place ? db.entries.filter((e) => e.category === "boss" && atPlace(e.id)) : []);
  const graces = $derived(place && block ? db.entries.filter((e) => e.category === "grace" && atPlace(e.id)) : []);
  const items = $derived(place ? [...gamedata.items].filter(([, p]) => inside(p.block, p.x, p.y, p.map)) : []);
  const itemsDone = $derived(items.filter(([iid]) => tracker.completed.has(iid)).length);

  const youtube = $derived(
    place ? `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring ${place.name.en} walkthrough`)}` : "",
  );
</script>

{#if !place}
  <p class="muted">Lieu introuvable.</p>
{:else}
  <header>
    <div class="badges"><span class="badge">{KINDS[place.icon] ?? "Lieu"}</span></div>
    <h2>{name}</h2>
    {#if place.name.fr && place.name.fr !== place.name.en}<div class="muted small">{place.name.en}</div>{/if}
  </header>

  {#if place}
    <section class="card">
      <h3>{block ? "À l'intérieur" : `Autour de l'entrée (${NEARBY} m)`}</h3>
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
