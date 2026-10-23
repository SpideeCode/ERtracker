<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import L from "leaflet";
  import "leaflet/dist/leaflet.css";
  import { entriesById, tracker } from "$lib/tracker.svelte";
  import type { Entry, MapId, MapInfo, MarkerPos } from "$lib/types";

  let { maps }: { maps: MapInfo[] } = $props();

  const MAP_NAMES: Record<MapId, string> = {
    M00: "Entre-terre",
    M01: "Souterrain",
    M10: "Royaume de l'Ombre",
    M11: "Souterrain (DLC)",
  };
  const KIND_COLORS = { grace: "#e8c46a", boss: "#e0574a" } as const;
  const DONE_COLOR = "#5fae6a";

  // http://ertiles.localhost/ on Windows, ertiles://localhost/ elsewhere.
  const base = convertFileSrc("", "ertiles");

  let el: HTMLDivElement;
  let map: L.Map;
  let tileLayer: L.TileLayer | null = null;
  let current = $state<MapId>("M00");
  let showGraces = $state(true);
  let showBosses = $state(true);
  let missingOnly = $state(false);
  let markers = $state<(MarkerPos & { entry: Entry })[]>([]);
  let shown = $state(0);
  let player: L.Marker | null = null;
  let centeredOnPlayer = false;

  const layer = L.layerGroup();
  const info = $derived(maps.find((m) => m.id === current) ?? maps[0]);

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
    layer.addTo(map);

    fetch(base + "markers.json")
      .then((r) => r.json())
      .then((all: MarkerPos[]) => {
        markers = all.flatMap((m) => {
          const entry = entriesById.get(`${m.kind}:${m.flag}`);
          return entry ? [{ ...m, entry }] : [];
        });
      });

    const startMap = tracker.snapshot?.map_position?.map;
    if (startMap && maps.some((m) => m.id === startMap)) current = startMap;
    return () => map.remove();
  });

  // Base tiles for the selected map.
  $effect(() => {
    const m = info;
    if (!map || !m) return;
    tileLayer?.remove();
    const bounds = L.latLngBounds(toLatLng(0, m.height), toLatLng(m.width, 0));
    tileLayer = L.tileLayer(`${base}tiles/${m.id}/{z}/{x}/{y}.jpg`, {
      minNativeZoom: m.min_zoom,
      maxNativeZoom: m.max_zoom,
      minZoom: 2,
      maxZoom: 8,
      noWrap: true,
      bounds,
    }).addTo(map);
    tileLayer.bringToBack();
    map.setMaxBounds(bounds.pad(0.2));
    if (!centeredOnPlayer) map.fitBounds(bounds);
  });

  // Markers: rebuilt when the map or filters change, restyled when progress changes.
  $effect(() => {
    if (!map) return;
    const done = tracker.completed;
    layer.clearLayers();
    let count = 0;
    for (const m of markers) {
      if (m.map !== current) continue;
      if ((m.kind === "grace" && !showGraces) || (m.kind === "boss" && !showBosses)) continue;
      const isDone = done.has(m.entry.id);
      if (missingOnly && isDone) continue;
      const circle = L.circleMarker(toLatLng(m.x, m.y), style(m.kind, isDone))
        .bindTooltip(m.entry.name, { direction: "top", offset: [0, -6] })
        .bindPopup(popup(m.entry, isDone));
      circle.addTo(layer);
      count++;
    }
    shown = count;
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
    if (!centeredOnPlayer) {
      centeredOnPlayer = true;
      map.setView(toLatLng(pos.x, pos.y), 5);
    }
  });

  function style(kind: MarkerPos["kind"], done: boolean): L.CircleMarkerOptions {
    return {
      radius: kind === "boss" ? 7 : 5,
      color: done ? DONE_COLOR : "#14110b",
      weight: 1.5,
      fillColor: done ? DONE_COLOR : KIND_COLORS[kind],
      fillOpacity: done ? 0.55 : 0.95,
      opacity: done ? 0.7 : 1,
    };
  }

  function popup(entry: Entry, done: boolean) {
    const div = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = entry.name;
    const meta = document.createElement("div");
    meta.textContent = `${entry.region ?? ""}${entry.region ? " · " : ""}${done ? "Fait ✓" : "À faire"}`;
    div.append(title, meta);
    return div;
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
      <label><input type="checkbox" bind:checked={missingOnly} /> Manquants seulement</label>
    </div>
    <div class="foot">
      <span class="muted">{shown} repères affichés</span>
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
</style>
