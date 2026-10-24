<script lang="ts">
  import { onMount, untrack } from "svelte";
  import L from "leaflet";
  import "leaflet/dist/leaflet.css";
  import { db, tracker } from "$lib/tracker.svelte";
  import { assetBase, gamedata } from "$lib/gamedata.svelte";
  import { DUNGEON_ICONS, formatDistance, planRoute } from "$lib/route";
  import { walkPath } from "$lib/pathfind";
  import { journal } from "$lib/journal.svelte";
  import { sessions } from "$lib/history";
  import { ui } from "$lib/ui.svelte";
  import { ITEM_CATEGORIES, categoryById, pickupCategory, pickupTitle } from "$lib/items";
  import { zoneData } from "$lib/zoneData.svelte";
  import { search, type SearchResult } from "$lib/search.svelte";
  import type { Entry, MapId, MapInfo, MarkerPos } from "$lib/types";
  import type { Snippet } from "svelte";
  import { ChevronDown, ChevronRight, ChevronsDownUp, ChevronsUpDown, LocateFixed, Search } from "@lucide/svelte";

  let { maps }: { maps: MapInfo[] } = $props();

  const MAP_NAMES: Record<MapId, string> = {
    M00: "Entre-terre",
    M01: "Souterrain",
    M10: "Royaume de l'Ombre",
    M11: "Souterrain (DLC)",
  };
  const KIND_COLORS: Record<string, string> = { grace: "#e8c46a", boss: "#e0574a" };
  const DONE_COLOR = "#5fae6a";

  let el: HTMLDivElement;
  let panel = $state<HTMLDivElement>();
  let map: L.Map;
  let tileLayer: L.TileLayer | null = null;
  let current = $state<MapId>("M00");
  // Layer choices, kept between launches.
  const LAYERS_KEY = "ertracker-map-layers";
  const IMPORTANT = ITEM_CATEGORIES.filter((c) => c.important).map((c) => c.id);
  const saved = (() => {
    try {
      return JSON.parse(localStorage.getItem(LAYERS_KEY) ?? "{}");
    } catch {
      return {};
    }
  })();
  let showGraces = $state<boolean>(saved.graces ?? true);
  let showBosses = $state<boolean>(saved.bosses ?? true);
  let missingOnly = $state<boolean>(saved.missingOnly ?? false);
  let showDungeons = $state<boolean>(saved.dungeons ?? true);
  let showExploration = $state<boolean>(saved.exploration ?? false);
  let showJourney = $state<boolean>(saved.journey ?? true);
  let showDeaths = $state<boolean>(saved.deaths ?? false);
  /** Item categories drawn on the map. */
  let itemCats = $state<string[]>(saved.items ?? IMPORTANT);
  let collapsed = $state<boolean>(saved.collapsed ?? false);
  let openSections = $state<Record<string, boolean>>(saved.open ?? { places: true, items: true, journey: false, legend: false });
  $effect(() => {
    const s = JSON.stringify({
      graces: showGraces,
      bosses: showBosses,
      missingOnly,
      dungeons: showDungeons,
      exploration: showExploration,
      journey: showJourney,
      deaths: showDeaths,
      items: itemCats,
      collapsed,
      open: openSections,
    });
    try {
      localStorage.setItem(LAYERS_KEY, s);
    } catch {
      /* layer choices are a convenience */
    }
  });
  const historyLayer = L.layerGroup();
  let shown = $state(0);
  let player: L.Marker | null = null;
  let centered = false;

  const markerLayer = L.layerGroup();
  const routeLayer = L.layerGroup();
  const info = $derived(maps.find((m) => m.id === current) ?? maps[0]);
  const visitedDungeons = $derived(new Set(zoneData.dungeons.filter((d) => d.done).map((d) => d.id)));

  const markers = $derived(
    db.entries.flatMap((entry) => {
      const pos = gamedata.positions.get(entry.id);
      return pos && (entry.category === "grace" || entry.category === "boss") ? [{ entry, pos }] : [];
    }),
  );

  // Done / total on the shown map, per layer and item category.
  const counts = $derived.by(() => {
    const done = tracker.completed;
    const c: Record<string, [number, number]> = { grace: [0, 0], boss: [0, 0], dungeon: [0, 0] };
    const add = (key: string, isDone: boolean) => {
      const n = (c[key] ??= [0, 0]);
      n[1]++;
      if (isDone) n[0]++;
    };
    for (const { entry, pos } of markers) if (pos.map === current) add(entry.category, done.has(entry.id));
    gamedata.places.forEach((p, i) => {
      if (p.map === current && DUNGEON_ICONS.has(p.icon)) add("dungeon", visitedDungeons.has(`place:${i}`));
    });
    for (const [id, p] of gamedata.items) if (p.map === current) add(pickupCategory(p), done.has(id));
    return c;
  });
  const left = (key: string) => {
    const [d, t] = counts[key] ?? [0, 0];
    return t - d;
  };
  const itemsLeft = $derived(ITEM_CATEGORIES.reduce((n, c) => n + (itemCats.includes(c.id) ? left(c.id) : 0), 0));

  function toggleCat(id: string) {
    itemCats = itemCats.includes(id) ? itemCats.filter((c) => c !== id) : [...itemCats, id];
  }
  const sameCats = (ids: string[]) => ids.length === itemCats.length && ids.every((id) => itemCats.includes(id));

  // Map search: fly to what was picked, making sure its layer is shown.
  let query = $state("");
  let searchOpen = $state(false);
  let selectedResult = $state(0);
  const results = $derived(search(query, 10));

  function pick(r: SearchResult | undefined) {
    if (!r) return;
    if (r.id.startsWith("boss:")) showBosses = true;
    else if (r.id.startsWith("grace:")) showGraces = true;
    else if (r.id.startsWith("place:")) showDungeons = true;
    else {
      const pickup = gamedata.items.get(r.id);
      const cat = pickup && pickupCategory(pickup);
      if (cat && !itemCats.includes(cat)) itemCats = [...itemCats, cat];
    }
    if (r.done) missingOnly = false;
    ui.sheet = r.id;
    const p = gamedata.positions.get(r.id);
    if (p) ui.flyTo = { map: p.map, x: p.x, y: p.y, label: r.title };
    query = "";
    searchOpen = false;
  }

  function onSearchKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") selectedResult = Math.min(selectedResult + 1, results.length - 1);
    else if (e.key === "ArrowUp") selectedResult = Math.max(selectedResult - 1, 0);
    else if (e.key === "Enter") pick(results[selectedResult]);
    else if (e.key === "Escape") {
      searchOpen = false;
      e.stopPropagation();
    } else return;
    e.preventDefault();
  }

  const toLatLng = (x: number, y: number) => map.unproject([x, y], info.max_zoom);

  onMount(() => {
    map = L.map(el, {
      crs: L.CRS.Simple,
      minZoom: 2,
      maxZoom: 8,
      zoomSnap: 0.5,
      zoomDelta: 0.5,
      preferCanvas: true,
      attributionControl: false,
    });
    historyLayer.addTo(map);
    markerLayer.addTo(map);
    routeLayer.addTo(map);
    journal.reload();
    // The side sheet changes the map's width.
    const resize = new ResizeObserver(() => map.invalidateSize());
    resize.observe(el);

    const startMap = tracker.snapshot?.map_position?.map;
    if (startMap && maps.some((m) => m.id === startMap)) current = startMap;
    return () => {
      resize.disconnect();
      map.remove();
    };
  });

  // Base tiles for the selected map.
  $effect(() => {
    const m = info;
    if (!map || !m) return;
    tileLayer?.remove();
    const bounds = L.latLngBounds(toLatLng(0, m.height), toLatLng(m.width, 0));
    tileLayer = L.tileLayer(`${assetBase}tiles/${m.id}/{z}/{x}/{y}.jpg`, {
      minNativeZoom: m.min_zoom,
      maxNativeZoom: m.max_zoom,
      minZoom: 2,
      maxZoom: 8,
      noWrap: true,
      bounds,
    }).addTo(map);
    tileLayer.bringToBack();
    map.setMaxBounds(bounds.pad(0.2));
    if (!untrack(() => centered)) map.fitBounds(bounds);
  });

  // Markers: rebuilt when the map, filters or progress change.
  $effect(() => {
    if (!map) return;
    const done = tracker.completed;
    const selected = ui.sheet;
    markerLayer.clearLayers();
    let count = 0;
    // Items first, so graces and bosses are drawn on top.
    const cats = new Set(itemCats);
    if (cats.size) {
      for (const [id, p] of gamedata.items) {
        if (p.map !== current) continue;
        const cat = pickupCategory(p);
        if (!cats.has(cat)) continue;
        const isDone = done.has(id);
        if (missingOnly && isDone) continue;
        const color = categoryById.get(cat)?.color ?? "#ccc";
        L.circleMarker(toLatLng(p.x, p.y), {
          radius: id === selected ? 7 : 4,
          color: id === selected ? "#5ab0ff" : "#14110b",
          weight: id === selected ? 3 : 1,
          fillColor: isDone ? DONE_COLOR : color,
          fillOpacity: isDone ? 0.45 : 0.95,
        })
          .bindTooltip(pickupTitle(p), { direction: "top", offset: [0, -4] })
          .on("click", () => (ui.sheet = id))
          .addTo(markerLayer);
        count++;
      }
    }
    for (const { entry, pos } of markers) {
      if (pos.map !== current) continue;
      if ((entry.category === "grace" && !showGraces) || (entry.category === "boss" && !showBosses)) continue;
      const isDone = done.has(entry.id);
      if (missingOnly && isDone) continue;
      const legendary = gamedata.bosses.get(entry.id)?.legendary ?? false;
      const circle = L.circleMarker(toLatLng(pos.x, pos.y), style(entry, isDone, legendary, entry.id === selected))
        .bindTooltip(gamedata.name(entry), { direction: "top", offset: [0, -6] });
      circle.on("click", () => (ui.sheet = entry.id));
      circle.addTo(markerLayer);
      count++;
    }
    if (showDungeons) {
      gamedata.places.forEach((p, i) => {
        if (p.map !== current || !DUNGEON_ICONS.has(p.icon)) return;
        const id = `place:${i}`;
        const visited = visitedDungeons.has(id);
        if (missingOnly && visited && id !== selected) return;
        L.marker(toLatLng(p.x, p.y), {
          icon: L.divIcon({ className: `dungeon-icon${visited ? " done" : ""}${id === selected ? " selected" : ""}`, iconSize: [12, 12] }),
        })
          .bindTooltip(p.name.fr ?? p.name.en, { direction: "top", offset: [0, -4] })
          .on("click", () => (ui.sheet = id))
          .addTo(markerLayer);
        count++;
      });
    }
    shown = count;
  });

  // Exploration (from what was picked up, discovered, defeated), journey and deaths (from the journal).
  $effect(() => {
    if (!map) return;
    historyLayer.clearLayers();
    const done = tracker.completed;
    const metres = (m: number) => m / 2 ** info.max_zoom;
    if (showExploration) {
      for (const [id, p] of gamedata.positions) {
        if (p.map !== current || !done.has(id)) continue;
        L.circle(toLatLng(p.x, p.y), { radius: metres(90), stroke: false, fillColor: "#f3d27a", fillOpacity: 0.13, interactive: false }).addTo(historyLayer);
      }
    }
    const all = sessions(journal.records);
    if (showJourney) {
      all.forEach((s, i) => {
        const recent = i === all.length - 1;
        const color = recent ? "#5ab0ff" : `hsl(${(i * 47) % 360} 55% 60%)`;
        let line: L.LatLng[] = [];
        let prev: { x: number; y: number } | null = null;
        const flush = () => {
          if (line.length > 1) L.polyline(line, { color, weight: recent ? 4 : 3, opacity: recent ? 0.95 : 0.6, interactive: false }).addTo(historyLayer);
          line = [];
        };
        for (const r of s.records) {
          const p = r.pos;
          if (!p || p.map !== current) {
            flush();
            prev = null;
            continue;
          }
          // Long jumps are fast travel, not walking.
          if (prev && Math.hypot(p.x - prev.x, p.y - prev.y) > 400) flush();
          line.push(toLatLng(p.x, p.y));
          prev = p;
        }
        flush();
      });
    }
    if (showDeaths) {
      for (let i = 1; i < journal.records.length; i++) {
        const r = journal.records[i];
        const n = r.deaths - journal.records[i - 1].deaths;
        const p = r.death_pos ?? r.pos;
        if (n <= 0 || !p || p.map !== current) continue;
        L.marker(toLatLng(p.x, p.y), { icon: L.divIcon({ className: "death-icon", html: "✕", iconSize: [14, 14] }) })
          .bindTooltip(`${n} mort${n > 1 ? "s" : ""} · ${new Date(r.t).toLocaleString("fr-FR")}`, { direction: "top" })
          .addTo(historyLayer);
      }
    }
  });

  // "You are here", from the last save.
  $effect(() => {
    const pos = tracker.snapshot?.map_position;
    if (!map) return;
    player?.remove();
    player = null;
    if (!pos || pos.map !== current) return;
    player = L.marker(toLatLng(pos.x, pos.y), {
      icon: L.divIcon({ className: "player-dot", iconSize: [18, 18] }),
      zIndexOffset: 1000,
      interactive: false,
    }).addTo(map);
    if (!centered && !ui.focus) {
      centered = true;
      map.setView(toLatLng(pos.x, pos.y), 5);
    }
  });

  // Focus requested from a boss sheet: centre on it and draw the way from the
  // closest discovered grace.
  $effect(() => {
    const id = ui.focus;
    const target = id ? gamedata.positions.get(id) : undefined;
    if (!map || !id || !target) return;
    untrack(() => {
      current = target.map;
      centered = true;
      ui.sheet = id;
      drawRoute(id, target);
      map.setView(toLatLng(target.x, target.y), 5.5);
      keepClearOfSheet();
      ui.focus = null;
    });
  });

  // A place picked from a route (grace, dungeon entrance): centre on it and flash it.
  const highlightLayer = L.layerGroup();
  $effect(() => {
    const fly = ui.flyTo;
    if (!map || !fly) return;
    untrack(() => {
      current = fly.map as MapId;
      centered = true;
      if (fly.route) {
        const target = gamedata.positions.get(fly.route);
        if (target) drawRoute(fly.route, target);
      }
      highlightLayer.clearLayers().addTo(map);
      const at = toLatLng(fly.x, fly.y);
      L.marker(at, { icon: L.divIcon({ className: "fly-ring", iconSize: [34, 34] }), interactive: false }).addTo(highlightLayer);
      if (fly.radius) {
        L.circle(at, {
          radius: fly.radius / 2 ** info.max_zoom,
          color: "#f3d27a",
          weight: 2,
          dashArray: "6 6",
          fillColor: "#f3d27a",
          fillOpacity: 0.08,
          interactive: false,
        }).addTo(highlightLayer);
      }
      L.tooltip({ permanent: true, direction: "top", className: "route-label", offset: [0, -16] })
        .setLatLng(at)
        .setContent(fly.label)
        .addTo(highlightLayer);
      if (fly.radius) map.fitBounds(L.latLngBounds(toLatLng(fly.x - fly.radius, fly.y + fly.radius), toLatLng(fly.x + fly.radius, fly.y - fly.radius)));
      else map.setView(at, 6);
      keepClearOfSheet();
      ui.flyTo = null;
    });
  });

  /** The layers panel and the side sheet cover the map: centre in the space left between them. */
  function keepClearOfSheet() {
    const w = el.clientWidth;
    const left = panel ? panel.offsetLeft + panel.offsetWidth : 0;
    const right = ui.sheet ? w - Math.min(420, w * 0.92) : w;
    if (right - left < 200) return;
    map.panBy([w / 2 - (left + right) / 2, 0], { animate: false });
  }

  let routeToken = 0;

  /** Walking route from the chosen start (see RouteCard) to a target, over the terrain data. */
  async function drawRoute(id: string, target: MarkerPos) {
    const token = ++routeToken;
    routeLayer.clearLayers();
    const plan = planRoute(target, gamedata.positions, tracker.completed, gamedata.places, (e) => gamedata.name(e), {
      goal: "la cible",
      exclude: id.startsWith("grace:") ? id : undefined,
    });
    const chosen = ui.routeStart[id];
    const me = tracker.snapshot?.map_position;
    const startPos =
      chosen === "me" && me?.map === target.map ? me : chosen ? gamedata.positions.get(chosen) : plan.warp?.pos;
    const startName =
      chosen === "me" ? "Ma position" : chosen ? gamedata.name(db.entries.find((e) => e.id === chosen)!) : plan.warp ? gamedata.name(plan.warp.grace) : null;
    const label = (at: L.LatLng, text: string, cls = "route-label") =>
      L.tooltip({ permanent: true, direction: "right", className: cls, offset: [10, 0] }).setLatLng(at).setContent(text).addTo(routeLayer);

    if (plan.gate && !chosen) {
      const at = toLatLng(plan.gate.pos.x, plan.gate.pos.y);
      L.circleMarker(at, { radius: 9, color: "#d39b45", weight: 3, fill: false, interactive: false }).addTo(routeLayer);
      label(at, `À débloquer : ${gamedata.name(plan.gate.grace)}`);
    }
    if (plan.entrance) {
      const at = toLatLng(plan.entrance.x, plan.entrance.y);
      L.marker(at, { icon: L.divIcon({ className: "entrance-icon", iconSize: [16, 16] }), interactive: false }).addTo(routeLayer);
      label(at, `Entrée : ${plan.entrance.name.fr ?? plan.entrance.name.en}`);
    }
    if (!startPos || startPos.map !== target.map) return;
    const from = toLatLng(startPos.x, startPos.y);
    L.circleMarker(from, { radius: 9, color: "#5ab0ff", weight: 2, fill: false, interactive: false }).addTo(routeLayer);
    if (startName) label(from, `Départ : ${startName}`);

    const walk = await walkPath(target.map, startPos, target);
    if (token !== routeToken) return;
    if (walk.found) {
      const line = walk.points.map(([x, y]) => toLatLng(x, y));
      L.polyline(line, { color: "#0b1a2a", weight: 7, opacity: 0.6, interactive: false }).addTo(routeLayer);
      L.polyline(line, { color: "#5ab0ff", weight: 4, interactive: false }).addTo(routeLayer);
      label(toLatLng(target.x, target.y), `${formatDistance(walk.length)} · ↑${Math.round(walk.ascent)} m ↓${Math.round(walk.descent)} m`);
    } else {
      const points = (plan.path.length >= 2 ? [startPos, ...plan.path.slice(1)] : [startPos, target]).map((p) => toLatLng(p.x, p.y));
      L.polyline(points, { color: "#5ab0ff", weight: 3, dashArray: "8 8", interactive: false }).addTo(routeLayer);
    }
  }

  function style(entry: Entry, done: boolean, legendary: boolean, selected: boolean): L.CircleMarkerOptions {
    const boss = entry.category === "boss";
    return {
      radius: boss ? (legendary ? 9 : 7) : 5,
      color: selected ? "#5ab0ff" : done ? DONE_COLOR : legendary ? "#f3d27a" : "#14110b",
      weight: selected ? 3 : legendary ? 2.5 : 1.5,
      fillColor: done ? DONE_COLOR : KIND_COLORS[entry.category],
      fillOpacity: done ? 0.55 : 0.95,
      opacity: done && !selected ? 0.7 : 1,
    };
  }

  function centerOnPlayer() {
    const pos = tracker.snapshot?.map_position;
    if (!pos) return;
    current = pos.map;
    map.setView(toLatLng(pos.x, pos.y), Math.max(map.getZoom(), 5));
  }
</script>

<div class="wrap">
  <div class="map" bind:this={el}></div>

  <div class="panel" class:collapsed bind:this={panel}>
    <div class="head">
      <div class="tabs">
        {#each maps as m (m.id)}
          <button class:active={current === m.id} onclick={() => (current = m.id)}>{MAP_NAMES[m.id]}</button>
        {/each}
      </div>
      <button class="icon" title={collapsed ? "Afficher les calques" : "Replier"} onclick={() => (collapsed = !collapsed)}>
        {#if collapsed}<ChevronsUpDown size={16} />{:else}<ChevronsDownUp size={16} />{/if}
      </button>
    </div>

    <div class="search">
      <Search size={14} />
      <input
        type="search"
        placeholder="Trouver sur la carte…"
        bind:value={query}
        onfocus={() => (searchOpen = true)}
        oninput={() => {
          searchOpen = true;
          selectedResult = 0;
        }}
        onkeydown={onSearchKey}
        onblur={() => setTimeout(() => (searchOpen = false), 150)}
      />
      {#if searchOpen && results.length}
        <ul class="results">
          {#each results as r, i (r.id)}
            <li>
              <button class:selected={i === selectedResult} onmousedown={() => pick(r)} onmouseenter={() => (selectedResult = i)}>
                <span class="swatch" style="background: {r.color}"></span>
                <span class="title" class:done={r.done}>{r.title}{r.done ? " ✓" : ""}</span>
                <span class="kind">{r.kind}</span>
              </button>
            </li>
          {/each}
        </ul>
      {:else if searchOpen && query.trim().length >= 2}
        <div class="results empty">Aucun résultat</div>
      {/if}
    </div>

    {#if !collapsed}
      <div class="sections">
        {@render section("places", "Lieux", placesBody)}
        {@render section("items", itemCats.length ? `Objets · ${itemsLeft} à prendre` : "Objets", itemsBody)}
        {@render section("journey", "Mon parcours", journeyBody)}
        {@render section("legend", "Légende", legendBody)}
      </div>
      <div class="foot muted">{shown} repères · clique un repère pour sa fiche</div>
    {/if}
  </div>

  {#if tracker.snapshot?.map_position}
    <button class="locate" title="Centrer sur ma position" onclick={centerOnPlayer}><LocateFixed size={18} /></button>
  {/if}
</div>

{#snippet section(id: string, title: string, body: Snippet)}
  <section>
    <button class="section-title" onclick={() => (openSections = { ...openSections, [id]: !openSections[id] })}>
      {#if openSections[id]}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
      {title}
    </button>
    {#if openSections[id]}<div class="body">{@render body()}</div>{/if}
  </section>
{/snippet}

{#snippet tally(key: string)}
  {@const n = counts[key] ?? [0, 0]}
  <span class="count">{n[0]}/{n[1]}</span>
{/snippet}

{#snippet placesBody()}
  <div class="chips">
    <button class="chip" class:on={showGraces} onclick={() => (showGraces = !showGraces)}>
      <span class="dot grace"></span> Grâces {@render tally("grace")}
    </button>
    <button class="chip" class:on={showBosses} onclick={() => (showBosses = !showBosses)}>
      <span class="dot boss"></span> Boss {@render tally("boss")}
    </button>
    <button class="chip" class:on={showDungeons} onclick={() => (showDungeons = !showDungeons)}>
      <span class="dot dungeon"></span> Donjons {@render tally("dungeon")}
    </button>
  </div>
  <label class="check"><input type="checkbox" bind:checked={missingOnly} /> Masquer ce qui est fait</label>
{/snippet}

{#snippet itemsBody()}
  <div class="presets">
    <button class:on={sameCats(IMPORTANT)} onclick={() => (itemCats = IMPORTANT)}>Essentiels</button>
    <button class:on={itemCats.length === ITEM_CATEGORIES.length} onclick={() => (itemCats = ITEM_CATEGORIES.map((c) => c.id))}>Tous</button>
    <button class:on={!itemCats.length} onclick={() => (itemCats = [])}>Aucun</button>
  </div>
  <div class="cats">
    {#each ITEM_CATEGORIES as c (c.id)}
      <button class="cat" class:on={itemCats.includes(c.id)} onclick={() => toggleCat(c.id)}>
        <span class="dot" style="background: {c.color}"></span>
        <span class="name">{c.name}</span>
        <span class="count">{left(c.id) || "✓"}</span>
      </button>
    {/each}
  </div>
{/snippet}

{#snippet journeyBody()}
  <div class="chips">
    <button class="chip" class:on={showJourney} onclick={() => (showJourney = !showJourney)}><span class="line journey"></span> Mon voyage</button>
    <button class="chip" class:on={showExploration} onclick={() => (showExploration = !showExploration)}><span class="dot explored"></span> Exploré</button>
    <button class="chip" class:on={showDeaths} onclick={() => (showDeaths = !showDeaths)}><span class="cross">✕</span> Morts</button>
  </div>
  <p class="hint muted">Le voyage et les morts viennent du journal : ils s'enregistrent quand l'appli est ouverte pendant que tu joues.</p>
{/snippet}

{#snippet legendBody()}
  <ul class="legend">
    <li><span class="dot boss"></span> Boss à vaincre · <span class="dot boss legendary"></span> légendaire</li>
    <li><span class="dot grace"></span> Grâce à découvrir</li>
    <li><span class="dot dungeon"></span> Donjon · <span class="dot dungeon visited"></span> déjà visité</li>
    <li><span class="dot done"></span> Fait : vaincu, découvert ou ramassé</li>
    <li><span class="dot me"></span> Toi, à la dernière sauvegarde</li>
    <li><span class="line route"></span> Itinéraire demandé depuis une fiche</li>
  </ul>
{/snippet}

<style>
  .wrap {
    position: relative;
    height: 100%;
  }
  .map {
    position: absolute;
    inset: 0;
    background: #0e0d0b;
  }
  .panel {
    position: absolute;
    top: 12px;
    left: 12px;
    z-index: 1000;
    background: rgba(23, 21, 15, 0.94);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 310px;
    max-width: calc(100% - 24px);
    max-height: calc(100% - 24px);
    box-sizing: border-box;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  }
  .head {
    display: flex;
    align-items: flex-start;
    gap: 6px;
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    flex: 1;
  }
  .icon {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    padding: 2px;
    display: flex;
  }
  .icon:hover {
    color: var(--gold);
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 8px;
  }
  .search:focus-within {
    border-color: var(--gold);
  }
  .search input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    padding: 6px 0;
  }
  .results {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 10;
    list-style: none;
    margin: 0;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    max-height: 50vh;
    overflow-y: auto;
  }
  .results.empty {
    padding: 8px 10px;
    font-size: 13px;
  }
  .results button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    background: none;
    border: none;
    color: var(--text);
    padding: 5px 6px;
    border-radius: 6px;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .results button.selected {
    background: var(--surface-2);
  }
  .swatch {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .results .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .results .title.done {
    color: var(--muted);
  }
  .kind {
    font-size: 11px;
    color: var(--muted);
  }
  .sections {
    overflow-y: auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    margin: 0 -4px;
    padding: 0 4px;
  }
  section {
    border-top: 1px solid var(--border);
    padding: 6px 0;
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    background: none;
    border: none;
    padding: 2px 0;
    color: var(--gold);
    font: inherit;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    cursor: pointer;
    text-align: left;
  }
  .body {
    padding-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip,
  .presets button,
  .cat {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: 1px solid var(--border);
    color: var(--muted);
    border-radius: 14px;
    padding: 3px 9px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .chip.on,
  .presets button.on,
  .cat.on {
    background: var(--surface-2);
    border-color: var(--border-strong);
    color: var(--text);
  }
  .chip:not(.on) .dot,
  .cat:not(.on) .dot {
    opacity: 0.35;
  }
  .count {
    color: var(--muted);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
  .presets {
    display: flex;
    gap: 4px;
  }
  .presets button.on {
    border-color: var(--gold);
    color: var(--gold);
  }
  .cats {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cat {
    border-color: transparent;
    border-radius: 6px;
    padding: 3px 6px;
    text-align: left;
  }
  .cat .name {
    flex: 1;
  }
  .cat.on {
    border-color: transparent;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-dim);
    cursor: pointer;
  }
  .hint {
    margin: 0;
    font-size: 11px;
  }
  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12px;
    color: var(--text-dim);
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .line {
    width: 14px;
    height: 3px;
    border-radius: 2px;
    display: inline-block;
    flex-shrink: 0;
  }
  .line.journey,
  .line.route {
    background: #5ab0ff;
  }
  .cross {
    color: #e0574a;
    font-weight: 700;
    font-size: 11px;
  }
  .locate {
    position: absolute;
    right: 14px;
    bottom: 24px;
    z-index: 1000;
    width: 38px;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(23, 21, 15, 0.94);
    border: 1px solid var(--border-strong);
    border-radius: 50%;
    color: #5ab0ff;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.45);
  }
  .locate:hover {
    border-color: #5ab0ff;
  }
  .tabs button {
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-dim);
    border-radius: 14px;
    padding: 3px 10px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .tabs button.active {
    background: var(--gold);
    border-color: var(--gold);
    color: #1a1508;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
  }
  .dot.legendary {
    box-shadow: 0 0 0 2px #f3d27a;
  }
  .dot.done {
    background: #5fae6a;
  }
  .dot.me {
    background: #5ab0ff;
    box-shadow: 0 0 0 2px #fff;
  }
  .dot.explored {
    background: rgba(243, 210, 122, 0.45);
  }
  .dot.grace {
    background: #e8c46a;
  }
  .dot.boss {
    background: #e0574a;
  }
  .dot.dungeon {
    background: #9fb4c8;
    border-radius: 2px;
    transform: rotate(45deg) scale(0.8);
  }
  :global(.dungeon-icon) {
    background: #9fb4c8;
    border: 1px solid #14110b;
    transform: rotate(45deg);
    cursor: pointer;
  }
  .dot.dungeon.visited,
  :global(.dungeon-icon.done) {
    background: #5fae6a;
    opacity: 0.75;
  }
  :global(.dungeon-icon.selected) {
    border: 2px solid #5ab0ff;
    box-shadow: 0 0 8px #5ab0ff;
  }
  .foot {
    font-size: 11px;
  }
  :global(.player-dot) {
    border-radius: 50%;
    background: #5ab0ff;
    border: 3px solid #fff;
    box-shadow: 0 0 0 4px rgba(90, 176, 255, 0.35), 0 0 12px #5ab0ff;
  }
  :global(.leaflet-container) {
    font: inherit;
  }
  :global(.leaflet-popup-content-wrapper),
  :global(.leaflet-popup-tip) {
    background: var(--surface);
    color: var(--text);
  }
  :global(.leaflet-tooltip) {
    background: var(--surface);
    color: var(--text);
    border-color: var(--border-strong);
  }
  :global(.fly-ring) {
    border: 3px solid #f3d27a;
    border-radius: 50%;
    box-shadow: 0 0 14px #f3d27a;
    animation: fly-pulse 1.2s ease-out 3;
  }
  @keyframes fly-pulse {
    from {
      transform: scale(2.2);
      opacity: 0;
    }
  }
  :global(.death-icon) {
    color: #e0574a;
    font-weight: 700;
    font-size: 13px;
    line-height: 14px;
    text-align: center;
    text-shadow: 0 0 3px #000;
  }
  :global(.entrance-icon) {
    background: #5ab0ff;
    border: 2px solid #fff;
    transform: rotate(45deg);
    box-shadow: 0 0 10px #5ab0ff;
  }
  :global(.leaflet-tooltip.route-label) {
    border-color: #5ab0ff;
    font-weight: 600;
  }
</style>
