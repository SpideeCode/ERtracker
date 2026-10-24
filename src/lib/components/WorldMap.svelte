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
  import type { Entry, MapId, MapInfo, MarkerPos } from "$lib/types";

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
  let map: L.Map;
  let tileLayer: L.TileLayer | null = null;
  let current = $state<MapId>("M00");
  let showGraces = $state(true);
  let showBosses = $state(true);
  let missingOnly = $state(false);
  let showDungeons = $state(true);
  let showExploration = $state(false);
  let showJourney = $state(true);
  let showDeaths = $state(false);
  const historyLayer = L.layerGroup();
  /** "important", "none", "all" or an item category id. */
  let itemFilter = $state("important");
  let shown = $state(0);
  let player: L.Marker | null = null;
  let centered = false;

  const markerLayer = L.layerGroup();
  const routeLayer = L.layerGroup();
  const info = $derived(maps.find((m) => m.id === current) ?? maps[0]);

  const markers = $derived(
    db.entries.flatMap((entry) => {
      const pos = gamedata.positions.get(entry.id);
      return pos && (entry.category === "grace" || entry.category === "boss") ? [{ entry, pos }] : [];
    }),
  );

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
    if (itemFilter !== "none") {
      for (const [id, p] of gamedata.items) {
        if (p.map !== current) continue;
        const cat = pickupCategory(p);
        if (itemFilter === "important" ? !categoryById.get(cat)?.important : itemFilter !== "all" && itemFilter !== cat) continue;
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
        L.marker(toLatLng(p.x, p.y), {
          icon: L.divIcon({ className: id === selected ? "dungeon-icon selected" : "dungeon-icon", iconSize: [12, 12] }),
        })
          .bindTooltip(p.name.fr ?? p.name.en, { direction: "top", offset: [0, -4] })
          .on("click", () => (ui.sheet = id))
          .addTo(markerLayer);
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
      L.tooltip({ permanent: true, direction: "top", className: "route-label", offset: [0, -16] })
        .setLatLng(at)
        .setContent(fly.label)
        .addTo(highlightLayer);
      map.setView(at, 6);
      keepClearOfSheet();
      ui.flyTo = null;
    });
  });

  /** The boss sheet covers the right of the map: shift so the centre stays visible. */
  function keepClearOfSheet() {
    if (ui.sheet) map.panBy([Math.min(210, el.clientWidth * 0.23), 0], { animate: false });
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

  <div class="panel">
    <div class="tabs">
      {#each maps as m (m.id)}
        <button class:active={current === m.id} onclick={() => (current = m.id)}>{MAP_NAMES[m.id]}</button>
      {/each}
    </div>
    <div class="filters">
      <label><input type="checkbox" bind:checked={showGraces} /> <span class="dot grace"></span> Grâces</label>
      <label><input type="checkbox" bind:checked={showBosses} /> <span class="dot boss"></span> Boss</label>
      <label><input type="checkbox" bind:checked={showDungeons} /> <span class="dot dungeon"></span> Donjons</label>
      <label><input type="checkbox" bind:checked={missingOnly} /> Manquants seulement</label>
    </div>
    <div class="filters">
      <label><input type="checkbox" bind:checked={showExploration} /> Exploration</label>
      <label><input type="checkbox" bind:checked={showJourney} /> Mon voyage</label>
      <label><input type="checkbox" bind:checked={showDeaths} /> Morts</label>
    </div>
    <div class="filters">
      <label>
        Objets
        <select bind:value={itemFilter}>
          <option value="important">Essentiels</option>
          <option value="none">Aucun</option>
          <option value="all">Tous</option>
          {#each ITEM_CATEGORIES as c (c.id)}
            <option value={c.id}>{c.name}</option>
          {/each}
        </select>
      </label>
    </div>
    <div class="foot">
      <span class="muted">{shown} repères · clique un repère pour sa fiche</span>
      {#if tracker.snapshot?.map_position}
        <button class="link" onclick={centerOnPlayer}>◎ Ma position</button>
      {/if}
    </div>
  </div>
</div>

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
    gap: 10px;
    max-width: calc(100% - 24px);
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
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
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 13px;
  }
  .filters label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
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
  :global(.dungeon-icon.selected) {
    border: 2px solid #5ab0ff;
    box-shadow: 0 0 8px #5ab0ff;
  }
  .foot {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 12px;
  }
  .link {
    background: none;
    border: none;
    color: var(--gold);
    cursor: pointer;
    font: inherit;
    padding: 0;
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
