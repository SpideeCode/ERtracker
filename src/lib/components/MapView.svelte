<script lang="ts">
  import { mapdata } from "$lib/mapdata.svelte";
  import MapSetup from "./MapSetup.svelte";
  import WorldMap from "./WorldMap.svelte";
</script>

{#if !mapdata.status}
  <p class="muted pad">Chargement…</p>
{:else if !mapdata.status.manifest}
  <div class="pad"><MapSetup /></div>
{:else}
  {#key mapdata.status.manifest.game_build}
    <WorldMap maps={mapdata.status.manifest.maps} />
  {/key}
{/if}

<style>
  .pad {
    padding: 20px 28px;
  }
</style>
