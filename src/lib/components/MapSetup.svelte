<script lang="ts">
  import { mapdata } from "$lib/mapdata.svelte";

  // Editable copy, seeded once from the detected folder.
  let gameDir = $state((() => mapdata.status?.game_dir ?? "")());
</script>

<div class="setup">
  <h2>Préparer les cartes</h2>
  <p class="muted">
    Les cartes, les positions des grâces et des boss, leurs fiches et les noms en français sont lus directement dans
    ton installation d'Elden Ring. C'est fait une seule fois (environ 45 secondes, ~80 Mo) et rien n'est envoyé sur
    internet.
  </p>

  <label>
    Dossier du jeu
    <input bind:value={gameDir} placeholder="C:\Program Files (x86)\Steam\steamapps\common\ELDEN RING" disabled={mapdata.running} />
  </label>
  {#if !mapdata.status?.game_dir && !gameDir}
    <p class="muted small">Jeu non trouvé automatiquement : colle le chemin du dossier « ELDEN RING » (celui qui contient « Game »).</p>
  {/if}

  {#if mapdata.running}
    <div class="bar"><div style="width: {mapdata.percent}%"></div></div>
    <p class="muted small">{mapdata.step()}</p>
  {:else}
    <button class="primary" onclick={() => mapdata.start(gameDir)} disabled={!gameDir}>Préparer les cartes</button>
  {/if}

  {#if mapdata.error}
    <div class="error">Échec : {mapdata.error}</div>
  {/if}
</div>

<style>
  .setup {
    max-width: 560px;
    margin: 40px auto;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 24px 28px;
  }
  h2 {
    margin: 0 0 8px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 18px 0 8px;
    color: var(--text-dim);
  }
  input {
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 8px 10px;
    font: inherit;
  }
  .small {
    font-size: 12px;
  }
  .primary {
    margin-top: 12px;
    background: var(--gold);
    color: #1a1508;
    border: none;
    border-radius: 6px;
    padding: 9px 18px;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .primary:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .bar {
    margin-top: 16px;
    height: 8px;
    border-radius: 4px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: var(--gold);
    transition: width 0.2s;
  }
  .error {
    margin-top: 14px;
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: 8px 10px;
    border-radius: 6px;
  }
</style>
