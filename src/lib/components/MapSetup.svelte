<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { MapProgress, MapStatus } from "$lib/types";

  let { status, onready }: { status: MapStatus; onready: () => void } = $props();

  // Editable copies, seeded once from the status.
  let gameDir = $state((() => status.game_dir ?? "")());
  let progress = $state<MapProgress | null>(null);
  let error = $state<string | null>(null);
  let running = $state((() => status.running)());

  onMount(() => {
    const unlisten = listen<MapProgress>("maps://progress", (e) => {
      progress = e.payload;
      if (e.payload.finished) {
        running = false;
        if (e.payload.error) error = e.payload.error;
        else onready();
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  });

  async function start() {
    error = null;
    progress = null;
    try {
      await invoke("extract_maps", { gameDir });
      running = true;
    } catch (e) {
      error = String(e);
    }
  }

  const percent = $derived(progress && progress.total ? Math.round((progress.done / progress.total) * 100) : 0);
</script>

<div class="setup">
  <h2>{status.outdated ? "Mettre à jour les cartes" : "Préparer les cartes"}</h2>
  <p class="muted">
    {#if status.outdated}
      Le jeu a été mis à jour depuis la dernière préparation. Relance-la pour avoir les cartes et les positions à jour.
    {:else}
      Les cartes et les positions des grâces et des boss sont lues directement dans ton installation d'Elden Ring.
      C'est fait une seule fois (environ 30 secondes, ~80 Mo) et rien n'est envoyé sur internet.
    {/if}
  </p>

  <label>
    Dossier du jeu
    <input bind:value={gameDir} placeholder="C:\Program Files (x86)\Steam\steamapps\common\ELDEN RING" disabled={running} />
  </label>
  {#if !status.game_dir && !gameDir}
    <p class="muted small">Jeu non trouvé automatiquement : colle le chemin du dossier « ELDEN RING » (celui qui contient « Game »).</p>
  {/if}

  {#if running}
    <div class="bar"><div style="width: {percent}%"></div></div>
    <p class="muted small">
      {progress?.step === "tiles" ? `Tuiles de carte : ${progress.done} / ${progress.total}` : "Lecture des données du jeu…"}
    </p>
  {:else}
    <button class="primary" onclick={start} disabled={!gameDir}>
      {status.outdated ? "Mettre à jour" : "Préparer les cartes"}
    </button>
  {/if}

  {#if error}
    <div class="error">Échec : {error}</div>
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
