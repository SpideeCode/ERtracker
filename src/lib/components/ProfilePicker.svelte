<script lang="ts">
  // "How do you play?": three profiles, with NG+ suggested when the save says so.
  import { Sparkles, Trophy, Swords, Check } from "@lucide/svelte";
  import { PROFILES, prefs, type Profile } from "$lib/prefs.svelte";
  import { tracker } from "$lib/tracker.svelte";

  const ICONS: Record<Profile, typeof Sparkles> = { discover: Sparkles, complete: Trophy, build: Swords };
  const cycle = $derived(tracker.snapshot?.cycle ?? 0);
  const suggested = $derived<Profile | null>(cycle > 0 ? "build" : null);
</script>

<div class="profiles">
  {#each PROFILES as p (p.id)}
    {@const Icon = ICONS[p.id]}
    <button class="profile" class:on={prefs.profile === p.id} onclick={() => prefs.setProfile(p.id)}>
      <span class="icon"><Icon size={20} /></span>
      <span class="text">
        <strong>{p.title}</strong>
        {#if suggested === p.id}<span class="hint">Ta sauvegarde est en NG+{cycle}</span>{/if}
        <span class="muted">{p.text}</span>
      </span>
      {#if prefs.profile === p.id}<Check size={18} class="check" />{/if}
    </button>
  {/each}
</div>

<style>
  .profiles {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .profile {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    width: 100%;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .profile:hover {
    border-color: var(--gold-dim);
  }
  .profile.on {
    border-color: var(--gold);
    background: rgba(201, 169, 91, 0.08);
  }
  .icon {
    color: var(--gold);
    margin-top: 2px;
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .text .muted {
    font-size: 12px;
  }
  .hint {
    font-size: 11px;
    color: #6fbf73;
  }
  .profile :global(.check) {
    color: var(--gold);
  }
</style>
