<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { MapStatus } from "$lib/types";
  import { gamedata } from "$lib/gamedata.svelte";
  import MapSetup from "./MapSetup.svelte";
  import WorldMap from "./WorldMap.svelte";

  let status = $state<MapStatus | null>(null);
  let updating = $state(false);

  async function refresh() {
    status = await invoke<MapStatus>("map_status", { gameDir: null });
    updating = false;
    if (status.manifest) await gamedata.load();
  }

  onMount(refresh);
</script>

{#if !status}
  <p class="muted pad">Chargement…</p>
{:else if !status.manifest || updating || status.running}
  <div class="pad"><MapSetup {status} onready={refresh} /></div>
{:else}
  {#if status.outdated}
    <div class="notice">
      Nouvelle version du jeu ou de l'app : mets à jour les cartes pour avoir toutes les données.
      <button onclick={() => (updating = true)}>Mettre à jour</button>
    </div>
  {/if}
  <WorldMap maps={status.manifest.maps} />
{/if}

<style>
  .pad {
    padding: 20px 28px;
  }
  .notice {
    position: absolute;
    z-index: 1001;
    right: 12px;
    top: 12px;
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 13px;
  }
  .notice button {
    margin-left: 8px;
    background: none;
    border: none;
    color: var(--gold);
    cursor: pointer;
    font: inherit;
  }
</style>
