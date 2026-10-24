<script lang="ts">
  import { Lock, LockOpen } from "@lucide/svelte";
  // "Conditions d'accès" for a boss behind a quest, key item or story progress.
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { accessRules } from "$lib/accessCheck.svelte";
  import { ui } from "$lib/ui.svelte";
  import type { Entry } from "$lib/types";

  let { entry }: { entry: Entry } = $props();

  const rules = $derived(accessRules(entry));

  const help = $derived(
    `https://www.youtube.com/results?search_query=${encodeURIComponent(`Elden Ring how to unlock ${entry.name.split(" (")[0]}`)}`,
  );
</script>

{#each rules as rule}
  <section class="access" class:met={rule.met}>
    <h3 class="with-icon">{#if rule.met}<LockOpen size={13} /> Accès débloqué{:else}<Lock size={13} /> Conditions d'accès{/if}</h3>
    <p>{rule.text}</p>
    {#if rule.checks.length}
      <ul>
        {#each rule.checks as c}
          <li class:done={c.done}><span class="box">{c.done ? "✓" : ""}</span>{c.label}</li>
        {/each}
      </ul>
    {/if}
    {#if rule.quest}
      <div class="quest">
        <div>
          Quête : <strong>{rule.quest.npc}</strong> —
          {rule.quest.step < 0 ? "pas commencée" : `étape ${rule.quest.step + 1} / ${rule.quest.total}`}
        </div>
        {#if rule.quest.description}<div class="muted small">« {rule.quest.description} »</div>{/if}
        <button class="link" onclick={() => ui.openQuest(rule.quest!.id)}>Voir la quête →</button>
      </div>
    {/if}
    {#if !rule.met}
      <button class="help" onclick={() => openUrl(help)}>Comment débloquer l'accès ? ↗</button>
    {/if}
  </section>
{/each}

<style>
  .access {
    margin-top: 18px;
    border: 1px solid #d39b45;
    background: rgba(211, 155, 69, 0.08);
    border-radius: 8px;
    padding: 12px 14px;
  }
  .access.met {
    border-color: #3f6b42;
    background: rgba(95, 174, 106, 0.06);
  }
  h3 {
    color: #f0c27a !important;
  }
  .access.met h3 {
    color: #8fd893 !important;
  }
  p {
    margin: 0 0 8px;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0 0 8px;
  }
  li {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 2px 0;
  }
  li.done {
    color: var(--muted);
  }
  .box {
    width: 15px;
    height: 15px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    display: grid;
    place-items: center;
    font-size: 10px;
    color: var(--bg);
    flex-shrink: 0;
  }
  li.done .box {
    background: #5fae6a;
    border-color: #5fae6a;
  }
  .quest {
    border-top: 1px solid var(--border);
    padding-top: 8px;
    margin-bottom: 6px;
  }
  .small {
    font-size: 12px;
  }
  .link {
    background: none;
    border: none;
    padding: 0;
    margin-top: 4px;
    color: var(--gold);
    font: inherit;
    cursor: pointer;
  }
  .help {
    width: 100%;
    margin-top: 6px;
    background: none;
    border: 1px solid #d39b45;
    color: #f0c27a;
    border-radius: 6px;
    padding: 6px 12px;
    font: inherit;
    cursor: pointer;
  }
</style>
