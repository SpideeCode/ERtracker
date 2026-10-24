<script lang="ts">
  import { tracker } from "$lib/tracker.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { prefs } from "$lib/prefs.svelte";
  import { playtime } from "$lib/labels";
  import { ui, type SettingsTab } from "$lib/ui.svelte";
  import OverlayPanel from "./OverlayPanel.svelte";
  import RuneEditor from "./RuneEditor.svelte";

  let { tab }: { tab: SettingsTab } = $props();

  const snapshot = $derived(tracker.snapshot);
  const folderLabel = (path: string) => {
    const parts = path.split(/[\\/]/);
    return `${parts.at(-2)} · ${parts.at(-1)}`;
  };
  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "general", label: "Général" },
    { id: "overlay", label: "Overlay en jeu" },
    { id: "runes", label: "Runes" },
  ];
</script>

<h2>Réglages et outils</h2>
<div class="tabs">
  {#each tabs as t (t.id)}
    <button class:active={tab === t.id} onclick={() => ui.go({ kind: "settings", tab: t.id })}>{t.label}</button>
  {/each}
</div>

{#if tab === "general"}
  <section class="card">
    <h3>Personnage suivi</h3>
    {#if snapshot}
      <label>
        Personnage
        <select value={snapshot.slot} onchange={(e) => tracker.select(null, Number(e.currentTarget.value))}>
          {#each snapshot.characters as c (c.slot)}
            <option value={c.slot}>{c.name} · niv. {c.level} · {playtime(c.seconds_played)}</option>
          {/each}
        </select>
      </label>
    {/if}
    {#if tracker.saves.length}
      <label>
        Fichier de sauvegarde
        <select value={snapshot?.save_path} onchange={(e) => tracker.select(e.currentTarget.value, null)}>
          {#each tracker.saves as s (s.path)}
            <option value={s.path}>{folderLabel(s.path)}{s.seamless_coop ? " (Seamless Coop)" : ""}</option>
          {/each}
        </select>
      </label>
    {/if}
    {#if snapshot}<p class="muted small">{snapshot.save_path}</p>{/if}
  </section>

  <section class="card">
    <h3>Préférences</h3>
    <label class="check">
      <input type="checkbox" bind:checked={prefs.spoilerFree} onchange={() => prefs.save()} />
      <span>
        <strong>Mode sans spoil</strong>
        <span class="muted small">Ce que tu n'as pas encore trouvé (boss, grâces, objets) reste sans nom sur la carte, dans les listes et l'overlay ; sa fiche est voilée et la recherche ne le propose qu'une fois son nom complet tapé.</span>
      </span>
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={prefs.sidebarCollapsed} onchange={() => prefs.save()} />
      <span><strong>Menu replié</strong> <span class="muted small">Seulement les icônes, pour laisser plus de place.</span></span>
    </label>
    <button
      class="ghost"
      onclick={() => {
        prefs.welcomed = false;
        prefs.save();
      }}>Revoir la présentation</button
    >
  </section>

  <section class="card">
    <h3>Données du jeu</h3>
    <p class="muted small">
      Cartes, positions, fiches et noms français extraits de ton installation d'Elden Ring
      {mapdata.status?.game_dir ? `(${mapdata.status.game_dir})` : ""}.
    </p>
    {#if mapdata.running}
      <p>Mise à jour en cours… {mapdata.percent} % — {mapdata.step()}</p>
    {:else if mapdata.status?.game_dir}
      <button class="ghost" onclick={() => mapdata.status?.game_dir && mapdata.start(mapdata.status.game_dir)}>Relancer l'extraction</button>
    {/if}
    {#if mapdata.error}<p class="error">Échec : {mapdata.error}</p>{/if}
  </section>
{:else if tab === "overlay"}
  <OverlayPanel />
{:else}
  <RuneEditor />
{/if}

<style>
  h2 {
    margin: 0 0 12px;
  }
  h3 {
    margin: 0 0 10px;
    font-size: 14px;
  }
  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 18px;
  }
  .tabs button {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-dim);
    padding: 8px 14px;
    font: inherit;
    cursor: pointer;
  }
  .tabs button.active {
    color: var(--gold);
    border-bottom-color: var(--gold);
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    max-width: 760px;
    margin-bottom: 14px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
    color: var(--text-dim);
    font-size: 13px;
  }
  label.check {
    flex-direction: row;
    gap: 10px;
    align-items: flex-start;
    cursor: pointer;
  }
  label.check span {
    display: flex;
    flex-direction: column;
    color: var(--text);
  }
  .small {
    font-size: 12px;
  }
  .error {
    color: var(--danger);
  }
</style>
