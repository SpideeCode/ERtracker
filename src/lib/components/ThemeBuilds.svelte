<script lang="ts">
  import { Shield, Feather } from "@lucide/svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { catalog } from "$lib/catalog.svelte";
  import { tracker } from "$lib/tracker.svelte";
  import { THEMES, bestArmor, bestAshesOfWar, bestGoods, bestTalismans, bestWeapons, targetStats } from "$lib/builds";
  import { STATS, grade, rarityOf } from "$lib/equipment";
  import { ui } from "$lib/ui.svelte";

  let themeId = $state("holy");
  let ownedOnly = $state(false);

  const theme = $derived(THEMES.find((t) => t.id === themeId)!);
  const cat = $derived(gamedata.catalog);
  const stats = $derived(targetStats(theme, tracker.character?.stats));

  const keep = (c: number, id: number) => !ownedOnly || catalog.isOwned(c, id);
  const weapons = $derived(cat ? bestWeapons(cat, theme, stats, 40).filter((p) => keep(2, p.weapon.id)).slice(0, 12) : []);
  const talismans = $derived(cat ? bestTalismans(cat, theme, 20).filter((t) => keep(4, t.id)).slice(0, 8) : []);
  const tears = $derived(cat ? bestGoods(cat, theme, [10], 10).filter((g) => keep(1, g.id)) : []);
  const spells = $derived(
    cat && theme.spells.length
      ? bestGoods(cat, theme, theme.spells, 30, true)
          .filter((g) => keep(1, g.id))
          .slice(0, 10)
      : [],
  );
  const ashes = $derived(cat ? bestAshesOfWar(cat, theme, 12).filter((a) => keep(5, a.id)).slice(0, 6) : []);
  const armor = $derived(cat ? bestArmor(cat) : []);

  const SLOTS = ["Tête", "Torse", "Bras", "Jambes"];
  const open = (c: number, id: number) => (ui.sheet = `equip:${c}:${id}`);
  const status = (c: number, id: number) => {
    if (catalog.isOwned(c, id)) return { label: "possédé", cls: "ok" };
    return catalog.whereIs(c, id).length ? { label: "sur la carte", cls: "map" } : { label: "ailleurs", cls: "" };
  };
</script>

{#if !cat}
  <p class="notice">Les données d'équipement arrivent avec la mise à jour des données du jeu (onglet Carte).</p>
{:else}
  <div class="themes">
    {#each THEMES as t (t.id)}
      <button class="chip" class:active={t.id === themeId} onclick={() => (themeId = t.id)}>{t.name}</button>
    {/each}
  </div>

  <section class="card intro">
    <p><strong>Stats :</strong> {theme.advice}</p>
    <p><strong>Affinité conseillée</strong> pour une arme qui peut changer de cendre : {theme.affinity}.</p>
    <p class="muted small">
      Dégâts estimés au niveau d'amélioration maximum, avec tes stats et {theme.stats.map((s) => STATS[s]).join(" et ")} à 60 au moins.
    </p>
    <label class="check"><input type="checkbox" bind:checked={ownedOnly} /> Seulement ce que je possède</label>
  </section>

  <section>
    <h3>Armes</h3>
    <table>
      <tbody>
        {#each weapons as p (p.weapon.id)}
          {@const s = status(2, p.weapon.id)}
          <tr onclick={() => open(2, p.weapon.id)}>
            <td>
              <span style="color: {rarityOf(p.weapon.rarity).color}">{p.weapon.name.fr ?? p.weapon.name.en}</span>
              <div class="muted small">
                {STATS.map((n, i) => (p.weapon.scaling_max[i] >= 25 ? `${n.slice(0, 3)} ${grade(p.weapon.scaling_max[i])}` : "")).filter(Boolean).join(" · ")}
                {p.meets ? "" : " · prérequis non atteints"}
              </div>
            </td>
            <td class="num"><strong>{p.damage}</strong><div class="muted small">dégâts</div></td>
            <td class="state"><span class="pill {s.cls}">{s.label}</span></td>
          </tr>
        {:else}
          <tr><td class="muted">Aucune arme pour ce thème.</td></tr>
        {/each}
      </tbody>
    </table>
  </section>

  {#if ashes.length}
    <section>
      <h3>Cendres de guerre</h3>
      <ul class="list">
        {#each ashes as a (a.id)}
          {@const s = status(5, a.id)}
          <li>
            <button onclick={() => open(5, a.id)}>{a.name.fr ?? a.name.en}</button>
            <span class="muted small">{a.info?.fr ?? a.info?.en ?? ""}</span>
            <span class="pill {s.cls}">{s.label}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section>
    <h3>Talismans</h3>
    <ul class="list">
      {#each talismans as t (t.id)}
        {@const s = status(4, t.id)}
        <li>
          <button style="color: {rarityOf(t.rarity).color}" onclick={() => open(4, t.id)}>{t.name.fr ?? t.name.en}</button>
          <span class="muted small">{t.info?.fr ?? t.info?.en ?? ""}</span>
          <span class="pill {s.cls}">{s.label}</span>
        </li>
      {:else}
        <li class="muted">Aucun talisman spécifique.</li>
      {/each}
    </ul>
  </section>

  {#if spells.length}
    <section>
      <h3>{theme.spells.includes(5) ? "Sorts" : "Incantations"}</h3>
      <ul class="list">
        {#each spells as g (g.id)}
          {@const s = status(1, g.id)}
          <li>
            <button style="color: {rarityOf(g.rarity).color}" onclick={() => open(1, g.id)}>{g.name.fr ?? g.name.en}</button>
            <span class="muted small">
              {g.spell ? `${g.spell.fp} PC · ${["Int", "Foi", "Arc"].map((n, i) => (g.spell!.requirements[i] ? `${n} ${g.spell!.requirements[i]}` : "")).filter(Boolean).join(" ")}` : ""}
            </span>
            <span class="pill {s.cls}">{s.label}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if tears.length}
    <section>
      <h3>Larmes de cristal (Fiole de larmes)</h3>
      <ul class="list">
        {#each tears as g (g.id)}
          {@const s = status(1, g.id)}
          <li>
            <button onclick={() => open(1, g.id)}>{g.name.fr ?? g.name.en}</button>
            <span class="muted small">{g.info?.fr ?? g.info?.en ?? ""}</span>
            <span class="pill {s.cls}">{s.label}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section>
    <h3>Armures</h3>
    <p class="muted small">Pour chaque emplacement : la plus protectrice, et le meilleur rapport protection/poids.</p>
    <div class="armor">
      {#each armor as slot (slot.slot)}
        <div class="card">
          <h4>{SLOTS[slot.slot]}</h4>
          {#each [...slot.heavy.slice(0, 2), ...slot.light.slice(0, 2)] as a, i (a.id + "-" + i)}
            <button class="armor-row" onclick={() => open(3, a.id)}>
              <span class="with-icon">{#if i < 2}<Shield size={13} />{:else}<Feather size={13} />{/if} {a.name.fr ?? a.name.en}</span>
              <span class="muted small">{a.weight} · {catalog.isOwned(3, a.id) ? "possédé" : ""}</span>
            </button>
          {/each}
        </div>
      {/each}
    </div>
  </section>
{/if}

<style>
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
    margin: 22px 0 8px;
  }
  h4 {
    margin: 0 0 6px;
    font-size: 13px;
  }
  .notice {
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .themes {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    background: var(--surface);
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    border-radius: 14px;
    padding: 4px 12px;
    font: inherit;
    cursor: pointer;
  }
  .chip.active {
    background: var(--gold);
    border-color: var(--gold);
    color: #1a1508;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 14px;
  }
  .intro {
    margin-top: 14px;
  }
  .intro p {
    margin: 0 0 6px;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
    cursor: pointer;
    margin-top: 6px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  tr {
    cursor: pointer;
    border-bottom: 1px solid var(--border);
  }
  tr:hover {
    background: var(--surface);
  }
  td {
    padding: 6px 8px;
  }
  .num {
    text-align: right;
    white-space: nowrap;
  }
  .state {
    width: 100px;
    text-align: right;
  }
  .small {
    font-size: 12px;
  }
  .pill {
    font-size: 11px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 0 6px;
    color: var(--muted);
    white-space: nowrap;
  }
  .pill.ok {
    color: #5fae6a;
    border-color: #3f6b42;
  }
  .pill.map {
    color: var(--gold);
    border-color: var(--gold-dim);
  }
  .list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .list li {
    display: grid;
    grid-template-columns: minmax(160px, 260px) 1fr auto;
    gap: 10px;
    align-items: baseline;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
  }
  .list button,
  .armor-row {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .list button:hover,
  .armor-row:hover {
    color: var(--gold);
  }
  .armor {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
    gap: 10px;
  }
  .armor-row {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 3px 0;
    gap: 8px;
  }
</style>
