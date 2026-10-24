<script lang="ts">
  import { onMount } from "svelte";
  import { db, tracker } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { journal } from "$lib/journal.svelte";
  import { analyse, formatDuration, sessions } from "$lib/history";
  import { nearestGraces } from "$lib/route";
  import { playtime } from "$lib/labels";
  import { ui } from "$lib/ui.svelte";

  onMount(() => journal.reload());

  const records = $derived(journal.records);
  const bossEntries = $derived(db.entries.filter((e) => e.category === "boss"));
  const analysis = $derived(analyse(records, bossEntries, gamedata.positions, tracker.completed));
  const allSessions = $derived(sessions(records).reverse());
  const character = $derived(tracker.character);
  const nameOf = (id: string) => {
    const e = db.entries.find((x) => x.id === id);
    return e ? gamedata.name(e) : id;
  };
  const date = (t: number) =>
    new Date(t).toLocaleString("fr-FR", { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });

  // Deaths away from bosses, grouped by the closest grace.
  const spots = $derived.by(() => {
    const byGrace = new Map<string, number>();
    for (const d of analysis.deaths) {
      if (d.boss) continue;
      const g = nearestGraces({ ...d.pos, flag: 0, kind: "grace", block: "m60_00_00" }, gamedata.positions, tracker.completed, 1)[0];
      const key = g ? gamedata.name(g.grace) : "Ailleurs";
      byGrace.set(key, (byGrace.get(key) ?? 0) + d.count);
    }
    return [...byGrace].sort((a, b) => b[1] - a[1]).slice(0, 8);
  });
</script>

<h2>Mes stats</h2>

<section class="tiles">
  <div class="tile"><span class="muted">Morts au total</span><strong>{character?.deaths.toLocaleString("fr-FR") ?? "—"}</strong></div>
  <div class="tile"><span class="muted">Temps de jeu</span><strong>{character ? playtime(character.seconds_played) : "—"}</strong></div>
  <div class="tile"><span class="muted">Morts enregistrées</span><strong>{analysis.recordedDeaths}</strong></div>
  <div class="tile"><span class="muted">Sessions enregistrées</span><strong>{allSessions.length}</strong></div>
</section>

{#if records.length < 2}
  <p class="notice">
    Le journal démarre : à partir de maintenant, chaque sauvegarde du jeu est notée (morts, position, boss, objets).
    Joue avec ERtracker ouvert et reviens ici : tu verras tes essais par boss, tes sessions et ton voyage sur la carte.
    Le jeu ne garde aucun historique, donc ce qui s'est passé avant ne peut pas être reconstitué.
  </p>
{:else}
  <p class="muted small">Enregistré depuis le {date(records[0].t)}.</p>
{/if}

{#if analysis.bosses.length}
  <section>
    <h3>Essais par boss</h3>
    <table>
      <thead><tr><th>Boss</th><th class="num">Morts</th><th>Statut</th><th class="num">Durée</th></tr></thead>
      <tbody>
        {#each analysis.bosses as b (b.id)}
          <tr onclick={() => (ui.sheet = b.id)}>
            <td>{nameOf(b.id)}</td>
            <td class="num"><strong>{b.attempts}</strong></td>
            <td>
              {#if b.defeatedAt}
                <span class="good">Vaincu {b.attempts ? `après ${b.attempts} mort${b.attempts > 1 ? "s" : ""}` : "du premier coup"}</span>
              {:else}
                <span class="muted">en cours</span>
              {/if}
            </td>
            <td class="num muted">{formatDuration((b.defeatedAt ?? b.lastTry) - b.firstTry)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
{/if}

{#if spots.length}
  <section>
    <h3>Où tu meurs le plus (hors boss)</h3>
    <ul class="bars">
      {#each spots as [where, n]}
        <li>
          <span>{where}</span>
          <div class="bar"><div style="width: {(n / spots[0][1]) * 100}%"></div></div>
          <span class="num">{n}</span>
        </li>
      {/each}
    </ul>
  </section>
{/if}

{#if allSessions.length}
  <section>
    <h3>Sessions</h3>
    <table>
      <thead><tr><th>Début</th><th class="num">Durée</th><th class="num">Morts</th><th>Débloqué</th></tr></thead>
      <tbody>
        {#each allSessions as s (s.start)}
          {@const bosses = s.gained.filter((g) => g.startsWith("boss:"))}
          {@const others = s.gained.length - bosses.length}
          <tr class="plain">
            <td>{date(s.start)}</td>
            <td class="num">{formatDuration(s.played * 1000 || s.end - s.start)}</td>
            <td class="num">{s.deaths}</td>
            <td>
              {#each bosses as b}<button class="chip" onclick={() => (ui.sheet = b)}>{nameOf(b)}</button>{/each}
              {#if others}<span class="muted small">+{others} autres</span>{/if}
              {#if s.levels}<span class="muted small"> · +{s.levels} niveau{s.levels > 1 ? "x" : ""}</span>{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    <button class="link" onclick={() => ui.go({ kind: "map" })}>Voir mon voyage sur la carte →</button>
  </section>
{/if}

<style>
  h2 {
    margin: 0 0 12px;
  }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
    margin: 24px 0 8px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 10px;
  }
  .tile {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
  }
  .tile strong {
    font-size: 22px;
  }
  .notice {
    margin-top: 16px;
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .small {
    font-size: 12px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    font-size: 12px;
    color: var(--muted);
    font-weight: 500;
    padding: 4px 8px;
  }
  td {
    padding: 6px 8px;
    border-top: 1px solid var(--border);
  }
  tr:not(.plain) {
    cursor: pointer;
  }
  tr:not(.plain):hover {
    background: var(--surface);
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .good {
    color: #6fbf73;
  }
  .bars {
    list-style: none;
    padding: 0;
    margin: 0;
    max-width: 640px;
  }
  .bars li {
    display: grid;
    grid-template-columns: 220px 1fr 40px;
    gap: 10px;
    align-items: center;
    padding: 3px 0;
  }
  .bar {
    height: 8px;
    background: var(--surface-2);
    border-radius: 4px;
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: #b0503f;
  }
  .chip {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 10px;
    padding: 0 8px;
    margin: 0 4px 2px 0;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .link {
    margin-top: 10px;
    background: none;
    border: none;
    color: var(--gold);
    font: inherit;
    cursor: pointer;
    padding: 0;
  }
</style>
