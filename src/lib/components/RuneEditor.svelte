<script lang="ts">
  import { TriangleAlert } from "@lucide/svelte";
  // Sets the held runes in the save file, with the game closed.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { tracker } from "$lib/tracker.svelte";

  const MAX = 999_999_999;

  let running = $state<boolean | null>(null);
  let amount = $state(0);
  let understood = $state(false);
  let busy = $state(false);
  let result = $state<{ backup: string; offset: number; before: number; after: number } | null>(null);
  let error = $state<string | null>(null);

  const character = $derived(tracker.character);
  const hex = (n: number) => "0x" + n.toString(16).toUpperCase().padStart(8, "0");

  onMount(() => {
    const check = async () => (running = await invoke<boolean>("game_running"));
    check();
    const timer = setInterval(check, 3000);
    return () => clearInterval(timer);
  });

  $effect(() => {
    if (character && amount === 0) amount = character.runes;
  });

  const add = (n: number) => (amount = Math.min(MAX, (character?.runes ?? 0) + n));

  async function write() {
    if (!character) return;
    busy = true;
    error = null;
    result = null;
    try {
      result = await invoke("set_runes", { slot: character.slot, value: Math.max(0, Math.min(MAX, Math.floor(amount))) });
      understood = false;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>


<section class="warning">
  <h3 class="with-icon"><TriangleAlert size={14} /> À lire avant de modifier</h3>
  <ul>
    <li>
      <strong>Ferme complètement Elden Ring.</strong> Le jeu garde la sauvegarde en mémoire et l'écraserait à sa prochaine
      sauvegarde automatique. L'app refuse d'écrire tant que le jeu tourne.
    </li>
    <li>
      <strong>Joue hors ligne avec une sauvegarde modifiée.</strong> Lance le jeu en mode hors ligne (option « Lancer hors
      ligne » au démarrage, ou Steam en mode hors ligne). En ligne, des valeurs anormales peuvent entraîner une sanction de
      FromSoftware (« soft ban » : tu ne joues plus qu'avec d'autres joueurs signalés), même après coup.
    </li>
    <li>
      <strong>Une copie de secours est faite à chaque modification</strong>, à côté de ta sauvegarde
      (<code>ER0000.sl2.ertracker-….bak</code>). Pour revenir en arrière : jeu fermé, renomme cette copie en
      <code>ER0000.sl2</code>.
    </li>
    <li>
      Si la synchronisation Steam Cloud est active, Steam peut remplacer la sauvegarde modifiée par celle du cloud.
    </li>
  </ul>
</section>

<section class="card">
  <div class="row">
    <span>Jeu</span>
    {#if running === null}
      <span class="muted">vérification…</span>
    {:else if running}
      <span class="bad">● Lancé — ferme-le pour pouvoir écrire</span>
    {:else}
      <span class="good">● Fermé</span>
    {/if}
  </div>
  {#if character}
    <div class="row"><span>Personnage</span><strong>{character.name} (emplacement {character.slot + 1})</strong></div>
    <div class="row"><span>Runes actuelles</span><strong>{character.runes.toLocaleString("fr-FR")}</strong></div>
    <div class="row">
      <span>Adresse dans la sauvegarde</span>
      <code>{hex(character.runes_offset)}</code>
    </div>
    <p class="muted small">
      Entier non signé 32 bits (little-endian) dans <code>ER0000.sl2</code>, slot {character.slot + 1}. Ce n'est pas une
      adresse mémoire Cheat Engine : celle-ci change à chaque lancement du jeu.
    </p>
  {/if}
</section>

{#if character}
  <section class="card">
    <h3>Nouveau total</h3>
    <div class="amount">
      <input type="number" min="0" max={MAX} step="1000" bind:value={amount} />
      <span class="muted">runes</span>
    </div>
    <div class="quick">
      <button onclick={() => add(100_000)}>+100 000</button>
      <button onclick={() => add(1_000_000)}>+1 000 000</button>
      <button onclick={() => add(10_000_000)}>+10 000 000</button>
      <button onclick={() => (amount = MAX)}>Maximum</button>
    </div>
    <label class="check">
      <input type="checkbox" bind:checked={understood} />
      J'ai fermé le jeu, je jouerai hors ligne et j'accepte les risques.
    </label>
    <button class="primary" disabled={!understood || running !== false || busy} onclick={write}>
      {busy ? "Écriture…" : `Écrire ${Math.min(MAX, Math.max(0, Math.floor(amount))).toLocaleString("fr-FR")} runes dans la sauvegarde`}
    </button>

    {#if result}
      <p class="good">
        ✓ {result.before.toLocaleString("fr-FR")} → {result.after.toLocaleString("fr-FR")} runes à l'adresse {hex(result.offset)}.
      </p>
      <p class="muted small">Copie de secours : <code>{result.backup}</code></p>
    {/if}
    {#if error}<p class="bad">Échec : {error}</p>{/if}
  </section>
{/if}

<style>
  h3 {
    margin: 0 0 8px;
    font-size: 14px;
  }
  .warning {
    border: 1px solid #b0503f;
    background: rgba(176, 80, 63, 0.12);
    border-radius: 8px;
    padding: 12px 16px;
    max-width: 760px;
  }
  .warning ul {
    margin: 0;
    padding-left: 18px;
  }
  .warning li {
    margin: 6px 0;
  }
  .card {
    margin-top: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    max-width: 760px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 4px 0;
  }
  .good {
    color: #6fbf73;
  }
  .bad {
    color: var(--danger);
  }
  .small {
    font-size: 12px;
  }
  code {
    background: var(--surface-2);
    padding: 1px 6px;
    border-radius: 4px;
  }
  .amount {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .amount input {
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 7px 10px;
    font: inherit;
    font-size: 16px;
    width: 220px;
  }
  .quick {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin: 10px 0;
  }
  .quick button {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 5px 10px;
    font: inherit;
    cursor: pointer;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 8px 0 12px;
    cursor: pointer;
  }
</style>
