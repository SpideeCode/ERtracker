<script lang="ts">
  // Progression at a glance, game-like: rank, next goals, region medals, achievements.
  import { Crown, Target, Medal, Award, Lock } from "@lucide/svelte";
  import { achievements, POINTS, type Tier } from "$lib/achievements.svelte";
  import { regionScene } from "$lib/art";
  import { ui } from "$lib/ui.svelte";

  const rank = $derived(achievements.rank);
  const c = $derived(achievements.counts);
  const unlocked = $derived(achievements.list.filter((a) => a.unlocked).length);
  let showAll = $state(false);

  const TIER_NAMES: Record<Tier, string> = { bronze: "Bronze", silver: "Argent", gold: "Or" };
  const ways = $derived(
    [
      c.legendary < c.legendaryTotal && `un boss légendaire (+${POINTS.legendary})`,
      c.greatRunes < c.greatRunesTotal && `une Grande rune (+${POINTS.greatRune})`,
      c.quests < c.questsTotal && `une quête (+${POINTS.quest})`,
      c.bosses < c.bossesTotal && `un boss (+${POINTS.boss})`,
      c.dungeons < c.dungeonsTotal && `un donjon (+${POINTS.dungeon})`,
    ].filter(Boolean),
  );
</script>

<div class="grid">
  <section class="card rank">
    <div class="crest"><Crown size={34} /><span>{rank.level}</span></div>
    <div class="info">
      <div class="kicker">Ton rang</div>
      <h2>{rank.title}</h2>
      <div class="xp"><div style="width: {Math.round(rank.progress * 100)}%"></div></div>
      {#if rank.next}
        <p class="muted small">
          {achievements.score.toLocaleString("fr-FR")} points · encore <strong>{rank.missing}</strong> pour devenir
          <strong>{rank.next.title}</strong>{ways.length ? ` : par exemple ${ways.slice(0, 3).join(", ")}` : ""}.
        </p>
      {:else}
        <p class="muted small">Rang maximum. Il ne reste que la gloire.</p>
      {/if}
    </div>
  </section>

  <section class="card goals">
    <div class="kicker"><Target size={14} /> Prochains objectifs</div>
    {#each achievements.next as a (a.id)}
      <div class="goal">
        <div class="goal-head">
          <strong>{a.title}</strong>
          <span class="muted small">{a.done}/{a.goal}</span>
        </div>
        <div class="muted small">{a.text}</div>
        <div class="bar {a.tier}"><div style="width: {(a.done / a.goal) * 100}%"></div></div>
      </div>
    {:else}
      <p class="muted">Lance-toi : chaque boss, grâce et objet compte.</p>
    {/each}
  </section>
</div>

<section class="block">
  <div class="kicker"><Medal size={14} /> Régions · bronze à 50 %, argent à 80 %, or à 100 %</div>
  <div class="regions">
    {#each achievements.regions as r (r.region)}
      <button
        class="region"
        style="--scene: url({regionScene(r.region)})"
        onclick={() => ui.showPlace({ map: r.map, x: r.x, y: r.y, label: r.name })}
        title="Voir sur la carte"
      >
        <span class="region-name">{r.name}</span>
        <span class="region-foot">
          <span class="pct">{r.pct} %</span>
          {#if r.medal}<span class="medal {r.medal}" title="Médaille {TIER_NAMES[r.medal]}"><Medal size={16} /></span>{/if}
        </span>
        <span class="region-bar"><span style="width: {r.pct}%"></span></span>
      </button>
    {/each}
  </div>
</section>

<section class="block">
  <div class="kicker"><Award size={14} /> Hauts faits · {unlocked}/{achievements.list.length}</div>
  <div class="achievements">
    {#each achievements.list.filter((a) => showAll || !a.id.startsWith("region-")) as a (a.id)}
      <div class="achievement {a.tier}" class:locked={!a.unlocked}>
        <span class="badge">{#if a.unlocked}<Award size={20} />{:else}<Lock size={16} />{/if}</span>
        <span class="text">
          <strong>{a.title}</strong>
          <span class="muted small">{a.text}</span>
          {#if !a.unlocked}<span class="mini"><span style="width: {(a.done / a.goal) * 100}%"></span></span>{/if}
        </span>
      </div>
    {/each}
  </div>
  <button class="link" onclick={() => (showAll = !showAll)}>{showAll ? "Masquer les médailles de régions" : "Afficher aussi les hauts faits de régions"}</button>
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: 14px;
    margin-bottom: 14px;
  }
  @media (max-width: 900px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 16px 18px;
  }
  .kicker {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--gold);
    margin-bottom: 8px;
  }
  .small {
    font-size: 12px;
  }
  .rank {
    display: flex;
    gap: 18px;
    align-items: center;
    background: radial-gradient(circle at 10% 50%, rgba(201, 169, 91, 0.18), transparent 60%), var(--surface);
    border-color: var(--gold-dim);
  }
  .crest {
    position: relative;
    flex-shrink: 0;
    width: 82px;
    height: 82px;
    border-radius: 50%;
    border: 2px solid var(--gold);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--gold);
    box-shadow: 0 0 22px rgba(201, 169, 91, 0.35), inset 0 0 18px rgba(201, 169, 91, 0.2);
  }
  .crest span {
    font-family: var(--font-title);
    font-weight: 600;
    font-size: 16px;
    margin-top: -4px;
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  h2 {
    margin: 0 0 8px;
    font-family: var(--font-title);
    color: var(--gold);
    font-size: 26px;
  }
  .xp {
    height: 10px;
    border-radius: 5px;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    overflow: hidden;
  }
  .xp div {
    height: 100%;
    background: linear-gradient(90deg, #8a6d2c, #f3d27a);
    box-shadow: 0 0 10px rgba(243, 210, 122, 0.6);
  }
  .info p {
    margin: 8px 0 0;
  }
  .goal + .goal {
    margin-top: 12px;
  }
  .goal-head {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .bar,
  .mini {
    display: block;
    height: 6px;
    border-radius: 3px;
    background: var(--surface-2);
    overflow: hidden;
    margin-top: 5px;
  }
  .bar div,
  .mini span {
    display: block;
    height: 100%;
    background: var(--gold);
  }
  .bar.bronze div {
    background: #c27c45;
  }
  .bar.silver div {
    background: #c9d1d9;
  }
  .block {
    margin-bottom: 18px;
  }
  .regions {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
  }
  .region {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 110px;
    padding: 10px 12px 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background:
      linear-gradient(to top, rgba(14, 13, 11, 0.9) 12%, rgba(14, 13, 11, 0.05) 75%),
      var(--scene) center / cover no-repeat,
      var(--surface);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    transition:
      transform 0.15s,
      border-color 0.15s;
  }
  .region:hover {
    transform: translateY(-2px);
    border-color: var(--gold-dim);
  }
  .region-name {
    font-family: var(--font-title);
    font-weight: 600;
    text-shadow: 0 1px 3px #000;
  }
  .region-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .pct {
    font-size: 20px;
    font-weight: 600;
  }
  .medal {
    display: flex;
    padding: 4px;
    border-radius: 50%;
  }
  .medal.bronze {
    color: #e0965c;
    background: rgba(194, 124, 69, 0.2);
  }
  .medal.silver {
    color: #e6edf3;
    background: rgba(201, 209, 217, 0.2);
  }
  .medal.gold {
    color: #f3d27a;
    background: rgba(243, 210, 122, 0.25);
    box-shadow: 0 0 10px rgba(243, 210, 122, 0.5);
  }
  .region-bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
  }
  .region-bar span {
    display: block;
    height: 100%;
    background: var(--gold);
  }
  .achievements {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 8px;
  }
  .achievement {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
  }
  .achievement .badge {
    flex-shrink: 0;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .achievement.bronze .badge {
    color: #e0965c;
    background: rgba(194, 124, 69, 0.18);
  }
  .achievement.silver .badge {
    color: #e6edf3;
    background: rgba(201, 209, 217, 0.15);
  }
  .achievement.gold .badge {
    color: #f3d27a;
    background: rgba(243, 210, 122, 0.18);
  }
  .achievement.locked {
    opacity: 0.55;
  }
  .achievement.locked .badge {
    color: var(--muted);
    background: var(--surface-2);
  }
  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }
  .link {
    margin-top: 8px;
    background: none;
    border: none;
    padding: 0;
    color: var(--gold);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
</style>
