<script lang="ts">
  import { db, tracker } from "$lib/tracker.svelte";
  import { achievement, ago, categoryName } from "$lib/labels";

  let { now, open }: { now: number; open: (category: string) => void } = $props();

  const stats = $derived(
    db.categories.map((c) => {
      const entries = db.entries.filter((e) => e.category === c.id);
      const done = entries.filter((e) => tracker.completed.has(e.id)).length;
      return { id: c.id, name: c.name.fr, done, total: entries.length };
    }),
  );
  const done = $derived(stats.reduce((n, s) => n + s.done, 0));
  const total = $derived(stats.reduce((n, s) => n + s.total, 0));
  const percent = (n: number, t: number) => (t ? Math.round((n / t) * 100) : 0);
</script>

<section class="overall">
  <div class="ring" style="--p: {percent(done, total)}">
    <span>{percent(done, total)} %</span>
  </div>
  <div>
    <h2>Progression globale</h2>
    <p class="muted">{done} / {total} éléments suivis</p>
  </div>
</section>

<section class="grid">
  {#each stats as s (s.id)}
    <button class="card" onclick={() => open(s.id)}>
      <div class="row">
        <span>{s.name}</span>
        <span class="count">{s.done} / {s.total}</span>
      </div>
      <div class="bar"><div style="width: {percent(s.done, s.total)}%"></div></div>
    </button>
  {/each}
</section>

<section>
  <h3>Activité en direct</h3>
  {#if tracker.log.length === 0}
    <p class="muted">Rien pour l'instant. Joue : chaque sauvegarde auto du jeu mettra cette liste à jour.</p>
  {:else}
    <ul class="log">
      {#each tracker.log as item (item.at + item.id)}
        <li class:lost={!item.gained}>
          <span class="tag">{item.gained ? achievement(item.category) : "Annulé"}</span>
          <span>{item.label}</span>
          <span class="muted small">{categoryName(item.category)} · {ago(item.at, now)}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .overall {
    display: flex;
    align-items: center;
    gap: 24px;
    margin-bottom: 24px;
  }
  .ring {
    --size: 96px;
    width: var(--size);
    height: var(--size);
    border-radius: 50%;
    background: conic-gradient(var(--gold) calc(var(--p) * 1%), var(--surface-2) 0);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .ring span {
    width: calc(var(--size) - 16px);
    height: calc(var(--size) - 16px);
    border-radius: 50%;
    background: var(--bg);
    display: grid;
    place-items: center;
    font-size: 20px;
    font-weight: 600;
  }
  h2 {
    margin: 0 0 4px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
    margin-bottom: 28px;
  }
  .card {
    text-align: left;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 14px;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  .card:hover {
    border-color: var(--gold-dim);
  }
  .row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .count {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: var(--gold);
  }
  .log {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .log li {
    display: flex;
    gap: 12px;
    align-items: baseline;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .log li.lost {
    opacity: 0.6;
  }
  .tag {
    color: var(--gold);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    min-width: 170px;
  }
  .small {
    font-size: 12px;
    margin-left: auto;
  }
</style>
