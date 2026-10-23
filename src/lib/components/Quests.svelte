<script lang="ts">
  import { db, tracker } from "$lib/tracker.svelte";
  import { ui } from "$lib/ui.svelte";

  let search = $state("");
  let openId = $state<string | null>(null);
  $effect(() => {
    if (ui.questOpen) {
      openId = ui.questOpen;
      ui.questOpen = null;
    }
  });

  /** Index of the furthest step whose flags all match, or -1 when the quest has not started. */
  function currentStep(questId: string, steps: number): number {
    for (let i = steps - 1; i >= 0; i--) {
      if (tracker.completed.has(`${questId}#${i}`)) return i;
    }
    return -1;
  }

  const quests = $derived(
    db.quests
      .filter((q) => q.npc.toLowerCase().includes(search.trim().toLowerCase()))
      .map((q) => ({ ...q, current: currentStep(q.id, q.steps.length) })),
  );
</script>

<h2>Quêtes PNJ</h2>
<p class="muted">
  L'étape en surbrillance est la plus avancée qui correspond à l'état de ta sauvegarde. Les données de quêtes
  sont communautaires : elles peuvent être imprécises pour certains PNJ.
</p>
<input type="search" placeholder="Rechercher un PNJ…" bind:value={search} />

<ul class="quests">
  {#each quests as q (q.id)}
    <li>
      <button class="head" onclick={() => (openId = openId === q.id ? null : q.id)}>
        <span>{q.npc}</span>
        <span class="muted">
          {q.current < 0 ? "Pas commencée" : `Étape ${q.current + 1} / ${q.steps.length}`}
        </span>
      </button>
      {#if openId === q.id}
        <ol>
          {#each q.steps as step, i}
            <li class:past={i < q.current} class:current={i === q.current}>
              <div>{step.description}</div>
              {#if step.location}<div class="muted small">{step.location}</div>{/if}
            </li>
          {/each}
        </ol>
      {/if}
    </li>
  {/each}
</ul>

<style>
  h2 {
    margin: 0 0 6px;
  }
  input {
    width: 100%;
    margin: 12px 0;
  }
  .quests {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .quests > li {
    border-bottom: 1px solid var(--border);
  }
  .head {
    width: 100%;
    display: flex;
    justify-content: space-between;
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    padding: 10px 4px;
    cursor: pointer;
    text-align: left;
  }
  .head:hover {
    color: var(--gold);
  }
  ol {
    margin: 0 0 12px;
    padding-left: 28px;
  }
  ol li {
    padding: 6px 0;
    color: var(--muted);
  }
  ol li.past {
    color: var(--text-dim);
    text-decoration: line-through;
    text-decoration-color: var(--border-strong);
  }
  ol li.current {
    color: var(--text);
    border-left: 3px solid var(--gold);
    padding-left: 10px;
    margin-left: -13px;
  }
  .small {
    font-size: 12px;
  }
</style>
