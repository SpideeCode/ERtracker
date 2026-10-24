<script lang="ts">
  // Builds, from the character's own save: best weapons in hand, what a few more
  // items would bring, where to put the next levels, and the strongest builds.
  import { Swords, PackagePlus, TrendingUp, Crown, Palette, MapPin, Hand, HandMetal, Shield } from "@lucide/svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { tracker } from "$lib/tracker.svelte";
  import { advisor, topBuilds, type Need, type Option, type TopBuild } from "$lib/buildAdvisor.svelte";
  import { DAMAGE_TYPES, upgradeCost, rarityOf } from "$lib/equipment";
  import { SCALING_STATS, STAT_NAMES, STAT_SHORT, WEAPON_KINDS } from "$lib/ar";
  import { catalog } from "$lib/catalog.svelte";
  import { ui } from "$lib/ui.svelte";
  import ThemeBuilds from "./ThemeBuilds.svelte";
  import { ROLL_LIMIT, bestSet, equipLoad, type Goal, type Roll } from "$lib/armor";
  import { iconUrl } from "$lib/art";

  type Tab = "mine" | "improve" | "best" | "themes";
  let tab = $state<Tab>(readTab());
  function readTab(): Tab {
    try {
      return (localStorage.getItem("ertracker-builds-tab") as Tab) || "mine";
    } catch {
      return "mine";
    }
  }
  $effect(() => {
    try {
      localStorage.setItem("ertracker-builds-tab", tab);
    } catch {
      /* convenience */
    }
  });
  const TABS: { id: Tab; label: string; icon: typeof Swords }[] = [
    { id: "mine", label: "Mon build", icon: Swords },
    { id: "improve", label: "M'améliorer", icon: PackagePlus },
    { id: "best", label: "Meilleurs builds", icon: Crown },
    { id: "themes", label: "Par élément", icon: Palette },
  ];

  const stats = $derived(advisor.stats);
  const best = $derived(advisor.current[0] ?? null);
  /** Weapon studied in "next levels": the best in hand unless another is picked. */
  let pickedId = $state<number | null>(null);
  const studied = $derived(advisor.current.find((o) => o.weapon.id === pickedId) ?? best);

  /** Main stats of the character, e.g. "Force". */
  const style = $derived.by(() => {
    const main = SCALING_STATS.filter((i) => stats[i] >= 30).sort((a, b) => stats[b] - stats[a]);
    return main.length ? main.map((i) => STAT_NAMES[i]).join(" / ") : "pas encore marqué";
  });

  function openNeed(n: Need, o: Option) {
    if (n.where.length) ui.sheet = n.where[0];
    else if (n.kind === "weapon") ui.sheet = `equip:2:${o.weapon.id}`;
    else if (n.kind === "ash" && o.ash) ui.sheet = `equip:5:${o.ash.id}`;
  }

  // Strongest builds at a level: computed on demand (a second or two).
  let topLevel = $state(150);
  let top = $state<TopBuild[] | null>(null);
  let computing = $state(false);
  let archetype = $state<string | null>(null);
  function computeTop() {
    if (!gamedata.catalog) return;
    computing = true;
    setTimeout(() => {
      top = topBuilds(gamedata.catalog!, topLevel);
      computing = false;
    }, 30);
  }
  $effect(() => {
    if (tab === "best" && !top && !computing && gamedata.catalog?.reinforce) {
      topLevel = Math.max(100, Math.round((tracker.character?.level ?? 150) / 25) * 25);
      computeTop();
    }
  });
  const archetypes = $derived([...new Set((top ?? []).slice(0, 300).map((b) => b.archetype))].slice(0, 10));
  const topShown = $derived((top ?? []).filter((b) => !archetype || b.archetype === archetype).slice(0, 15));

  const damageLine = (ar: { byType: number[] }) =>
    ar.byType.map((v, t) => (v ? `${DAMAGE_TYPES[t]} ${v}` : "")).filter(Boolean).join(" · ");
  const ownsWeapon = (id: number) => advisor.own.weapons.has(id);

  // Armour: best set under the weight that keeps the chosen roll, with the weapon in hand.
  let roll = $state<Roll>("medium");
  let goal = $state<Goal>("defense");
  let ownedArmor = $state(true);
  const SLOTS = ["Tête", "Torse", "Bras", "Jambes"];
  const load = $derived(equipLoad(stats[2]));
  const weaponWeight = $derived(studied?.weapon.weight ?? 0);
  const budget = $derived(Math.max(0, load * ROLL_LIMIT[roll] - weaponWeight - 0.05));
  const armorSet = $derived.by(() => {
    const all = gamedata.catalog?.armor ?? [];
    const pool = ownedArmor ? all.filter((p) => catalog.isOwned(3, p.id)) : all.filter((p) => p.weight > 0);
    return bestSet(pool, budget, goal);
  });
  /** "Cendres de guerre : Lame sacrée" -> "Lame sacrée" */
  const ashName = (a: { name: { fr: string | null; en: string } }) => (a.name.fr ?? a.name.en).replace(/^(Cendres de guerre|Ash of War)\s*:\s*/, "");
  /** The weapon as it is now, when the advice changes its affinity. */
  const asNow = (o: Option) => {
    const mine = advisor.own.weapons.get(o.weapon.id);
    if (!mine || mine.affinity === o.variant.affinity) return null;
    return advisor.options.find((x) => x.weapon.id === o.weapon.id && x.variant.affinity === mine.affinity) ?? null;
  };
  const kindLabel = $derived(advisor.kind === null ? "arme" : (WEAPON_KINDS[advisor.kind] ?? "arme").toLowerCase());
</script>

<div class="tabs">
  {#each TABS as t (t.id)}
    {@const Icon = t.icon}
    <button class:active={tab === t.id} onclick={() => (tab = t.id)}><Icon size={15} /> {t.label}</button>
  {/each}
</div>

{#if tab === "themes"}
  <ThemeBuilds />
{:else if !advisor.ready}
  <p class="notice">
    Le calcul des dégâts arrive avec la mise à jour des données du jeu (elle se lance seule, voir la barre en haut).
  </p>
{:else if tab === "mine"}
  {@render kindFilter()}
  <section class="card">
    <div class="kicker">Ton personnage</div>
    <div class="me">
      <div>
        <h3>{tracker.character?.name} · niveau {tracker.character?.level}</h3>
        <p class="muted">
          Style : <strong>{style}</strong>{tracker.snapshot?.cycle ? ` · NG+${tracker.snapshot.cycle}` : ""}
        </p>
      </div>
      <div class="stats">
        {#each stats as v, i}
          <span class="stat" class:main={SCALING_STATS.includes(i) && v >= 30}><small>{STAT_SHORT[i]}</small>{v}</span>
        {/each}
      </div>
    </div>
  </section>

  <section class="card">
    <div class="kicker">Tes meilleures armes{advisor.kind === null ? "" : ` (${kindLabel})`}, avec ce que tu as déjà</div>
    <p class="muted small">
      Puissance d'attaque calculée comme dans le jeu avec tes stats et le niveau de chaque arme. Une affinité
      n'apparaît que si tu as la pierre à aiguiser et une cendre de guerre qui la permet.
    </p>
    <ol class="options">
      {#each advisor.current.slice(0, 6) as o, i (o.weapon.id)}
        {@render option(o, i === 0, true)}
      {:else}
        <li class="muted">Tu n'as pas encore d'arme de ce type : regarde l'onglet « M'améliorer ».</li>
      {/each}
    </ol>
  </section>

  <section class="card">
    <div class="kicker"><Shield size={14} /> Armure conseillée</div>
    <p class="muted small">
      Charge max {load.toFixed(1)} (Endurance {stats[2]}). Pour une roulade {roll === "medium" ? "moyenne" : "légère"}, tout ton
      équipement doit rester sous {(load * ROLL_LIMIT[roll]).toFixed(1)} : il reste <strong>{budget.toFixed(1)}</strong> pour l'armure
      avec {studied?.name ?? "ton arme"} ({weaponWeight.toFixed(1)}).
    </p>
    <div class="controls">
      <div class="segmented">
        <button class:active={roll === "medium"} onclick={() => (roll = "medium")}>Roulade moyenne</button>
        <button class:active={roll === "light"} onclick={() => (roll = "light")}>Légère</button>
      </div>
      <div class="segmented">
        <button class:active={goal === "defense"} onclick={() => (goal = "defense")}>Priorité défense</button>
        <button class:active={goal === "poise"} onclick={() => (goal = "poise")}>Priorité posture</button>
      </div>
      <div class="segmented">
        <button class:active={ownedArmor} onclick={() => (ownedArmor = true)}>Mes armures</button>
        <button class:active={!ownedArmor} onclick={() => (ownedArmor = false)}>Tout le jeu</button>
      </div>
    </div>
    {#if armorSet && armorSet.pieces.length}
      <div class="armor">
        {#each [0, 1, 2, 3] as slot (slot)}
          {@const p = armorSet.pieces.find((x) => x.slot === slot)}
          <div class="piece">
            {#if p && iconUrl(p.icon)}<img src={iconUrl(p.icon)} alt="" loading="lazy" />{:else}<span class="empty"></span>{/if}
            <span class="muted small">{SLOTS[slot]}</span>
            {#if p}
              <button class="name small" onclick={() => (ui.sheet = `equip:3:${p.id}`)}>{p.name.fr ?? p.name.en}</button>
              <span class="muted small">{p.weight.toFixed(1)} · posture {Math.round(p.poise ?? 0)}</span>
              {#if !catalog.isOwned(3, p.id)}
                {#if catalog.whereIs(3, p.id).length}
                  <button class="pill map" onclick={() => (ui.sheet = catalog.whereIs(3, p.id)[0])}><MapPin size={11} /> sur la carte</button>
                {:else}<span class="pill">boss, marchand ou quête</span>{/if}
              {/if}
            {:else}
              <span class="muted small">Rien (le poids manque)</span>
            {/if}
          </div>
        {/each}
      </div>
      <p class="totals">
        <span>Physique <strong>{armorSet.physical.toFixed(1)} %</strong></span>
        <span>Éléments <strong>{armorSet.elemental.toFixed(1)} %</strong></span>
        <span>Posture <strong>{Math.round(armorSet.poise)}</strong></span>
        <span>Poids <strong>{armorSet.weight.toFixed(1)}</strong> / {budget.toFixed(1)}</span>
      </p>
    {:else}
      <p class="muted">Pas de combinaison possible avec ce poids : monte l'Endurance ou prends une arme plus légère.</p>
    {/if}
  </section>

  {#if studied}
    {@const plans = [5, 10, 20].map((n) => ({ n, r: advisor.nextLevels(studied, n) }))}
    {@const re = advisor.respec(studied)}
    <section class="card">
      <div class="kicker"><TrendingUp size={14} /> Où mettre tes prochains niveaux</div>
      <label class="pick">
        Pour
        <select bind:value={pickedId}>
          <option value={null}>{best?.name} (ta meilleure arme)</option>
          {#each advisor.current.slice(1, 12) as o (o.weapon.id)}
            <option value={o.weapon.id}>{o.name}</option>
          {/each}
        </select>
      </label>
      <table class="plan">
        <thead><tr><th>Niveaux</th><th>Répartition</th><th class="num">Attaque</th></tr></thead>
        <tbody>
          {#each plans as { n, r } (n)}
            <tr>
              <td>+{n}</td>
              <td>{@render spread(r.stats)}</td>
              <td class="num"><strong>{r.ar.total}</strong> <span class="gain">+{r.ar.total - studied.ar.total}</span></td>
            </tr>
          {/each}
          <tr class="respec">
            <td>Renaissance</td>
            <td>
              {@render spread(re.stats)}
              <div class="muted small">Même niveau, stats offensives refaites (Larme larvaire chez Rennala). Base 10, ta classe peut imposer plus.</div>
            </td>
            <td class="num">
              <strong>{re.ar.total}</strong>
              <span class="gain" class:neg={re.ar.total < studied.ar.total}>{re.ar.total >= studied.ar.total ? "+" : ""}{re.ar.total - studied.ar.total}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </section>
  {/if}
{:else if tab === "improve"}
  {@render kindFilter()}
  <p class="lead">
    {#if best}
      Ta référence : <strong>{best.name} +{best.level}</strong>, <strong>{best.ar.total}</strong> d'attaque. Voici ce que tu peux viser,
      selon le nombre d'objets à récupérer.
    {:else}
      Tu n'as pas encore de {kindLabel} : voici les meilleures à récupérer.
    {/if}
    Une nouvelle arme est comptée au niveau de ta meilleure (+{advisor.reach.regular}, ou +{advisor.reach.somber} pour une arme unique).
  </p>
  {#each advisor.upgrades as list, i}
    {@const beats = list.filter((o) => !best || o.ar.total > best.ar.total).length}
    <section class="card">
      <div class="kicker"><PackagePlus size={14} /> Avec {i + 1} objet{i ? "s" : ""} à récupérer</div>
      {#if best && list.length && !beats}
        <p class="muted small">Rien ne dépasse ta meilleure arme ici : voici les options les plus proches, utiles pour changer de style.</p>
      {/if}
      {#if list.length}
        <ol class="options">
          {#each list as o (o.weapon.id + ":" + o.variant.affinity)}
            {@render option(o, false, false)}
          {/each}
        </ol>
      {:else}
        <p class="muted">Aucune option avec {i + 1} objet{i ? "s" : ""}.</p>
      {/if}
    </section>
  {/each}
  {#if advisor.smithing.length}
    <section class="card">
      <div class="kicker"><TrendingUp size={14} /> En améliorant une arme que tu as</div>
      <ol class="options">
        {#each advisor.smithing as { o, max } (o.weapon.id)}
          {@const cost = upgradeCost(gamedata.catalog!, o.weapon, o.level, o.weapon.max_level)}
          <li class="option">
            {#if iconUrl(o.weapon.icon)}<img class="wicon" src={iconUrl(o.weapon.icon)} alt="" loading="lazy" />{/if}
            <div class="main">
              <button class="name" style="color: {rarityOf(o.weapon.rarity).color}" onclick={() => (ui.sheet = `equip:2:${o.weapon.id}`)}>{o.name}</button>
              <span class="muted small">+{o.level} → +{o.weapon.max_level} · {o.ar.total} → {max.ar.total}</span>
              <div class="muted small">{cost.runes.toLocaleString("fr-FR")} runes · {cost.materials.map((m) => `${m.count} ${m.name}`).join(", ")}</div>
            </div>
            <div class="side">
              <strong class="ar">{max.ar.total}</strong>
              {#if best}<span class="gain" class:neg={max.ar.total < best.ar.total}>{max.ar.total >= best.ar.total ? "+" : ""}{max.ar.total - best.ar.total}</span>{/if}
            </div>
          </li>
        {/each}
      </ol>
    </section>
  {/if}
{:else if tab === "best"}
  <section class="card">
    <div class="kicker"><Crown size={14} /> Les builds les plus puissants, calculés</div>
    <p class="muted small">
      Pour chaque arme et chaque affinité, au niveau d'amélioration maximum : Vigueur 60, Esprit 20, Endurance 25, et le
      reste réparti au mieux pour l'arme. Classés par puissance d'attaque (hors buffs, saignement et effets).
    </p>
    <div class="controls">
      <label>
        Niveau du personnage
        <select bind:value={topLevel} onchange={computeTop}>
          {#each [80, 100, 125, 150, 175, 200, 250] as l}<option value={l}>{l}</option>{/each}
        </select>
      </label>
      <div class="chips">
        <button class="chip" class:active={!archetype} onclick={() => (archetype = null)}>Tous</button>
        {#each archetypes as a (a)}
          <button class="chip" class:active={archetype === a} onclick={() => (archetype = a)}>{a}</button>
        {/each}
      </div>
    </div>
    {#if computing || !top}
      <p class="muted">Calcul de toutes les combinaisons…</p>
    {:else}
      <ol class="options">
        {#each topShown as b (b.weapon.id + ":" + b.variant.affinity)}
          <li class="option">
            {#if iconUrl(b.weapon.icon)}<img class="wicon" src={iconUrl(b.weapon.icon)} alt="" loading="lazy" />{/if}
            <div class="main">
              <button class="name" style="color: {rarityOf(b.weapon.rarity).color}" onclick={() => (ui.sheet = `equip:2:${b.weapon.id}`)}>{b.name}</button>
              <span class="muted small">+{b.weapon.max_level} · {b.archetype}{b.twoHanded ? " · à deux mains" : ""}</span>
              <div class="muted small">{damageLine(b.ar)}</div>
              <div class="small">{@render spread(b.stats)}</div>
            </div>
            <div class="side">
              <strong class="ar">{b.ar.total}</strong>
              {#if ownsWeapon(b.weapon.id)}<span class="pill ok">à toi</span>
              {:else if catalog.whereIs(2, b.weapon.id).length}
                <button class="pill map" onclick={() => (ui.sheet = catalog.whereIs(2, b.weapon.id)[0])}><MapPin size={11} /> sur la carte</button>
              {:else}<span class="pill">boss, marchand ou quête</span>{/if}
            </div>
          </li>
        {/each}
      </ol>
    {/if}
  </section>
{/if}

{#snippet kindFilter()}
  <label class="kind">
    Type d'arme
    <select bind:value={advisor.kind}>
      <option value={null}>Tous les types</option>
      {#each advisor.kinds as k (k)}
        {#if WEAPON_KINDS[k]}<option value={k}>{WEAPON_KINDS[k]}</option>{/if}
      {/each}
    </select>
  </label>
{/snippet}

{#snippet spread(s: number[])}
  <span class="spread">
    {#each SCALING_STATS as i (i)}
      {#if s[i] !== stats[i] || s[i] >= 20}
        <span class:up={s[i] > stats[i]} class:down={s[i] < stats[i]}>{STAT_SHORT[i]} {s[i]}{s[i] > stats[i] ? ` (+${s[i] - stats[i]})` : ""}</span>
      {/if}
    {/each}
  </span>
{/snippet}

{#snippet option(o: Option, first: boolean, mine: boolean)}
  {@const max = o.weapon.max_level}
  {@const cost = mine && o.level < max ? upgradeCost(gamedata.catalog!, o.weapon, o.level, max) : null}
  <li class="option" class:first>
    {#if iconUrl(o.weapon.icon)}<img class="wicon" src={iconUrl(o.weapon.icon)} alt="" loading="lazy" />{/if}
    <div class="main">
      <button class="name" style="color: {rarityOf(o.weapon.rarity).color}" onclick={() => (ui.sheet = `equip:2:${o.weapon.id}`)}>{o.name}</button>
      <span class="muted small">
        +{o.level}
        {#if o.twoHanded}· <Hand size={11} /> à deux mains (à une main : {o.oneHand}){:else}· <HandMetal size={11} /> à une main{/if}
      </span>
      <div class="muted small">{damageLine(o.ar)}</div>
      {#if o.ar.missing.length}
        <div class="warn small">Il te manque des points en {o.ar.missing.map((i) => STAT_NAMES[i]).join(", ")} (dégâts réduits)</div>
      {/if}
      {#if o.ash && !o.needs.some((n) => n.kind === "ash")}
        {@const now = asNow(o)}
        <div class="small">
          Mets-lui la cendre <strong>{ashName(o.ash)}</strong> (déjà à toi) en affinité {o.name.replace(o.weapon.name.fr ?? o.weapon.name.en, "").trim() || "standard"}, à une grâce
          {#if now}<span class="muted"> · aujourd'hui : {now.name}, {now.ar.total}</span>{/if}
        </div>
      {/if}
      {#if o.needs.length}
        <ul class="needs">
          {#each o.needs as n (n.label)}
            <li>
              <span class="need-kind">{n.kind === "weapon" ? "Arme" : n.kind === "ash" ? "Cendre de guerre" : "Pierre à aiguiser"}</span>
              {n.label}
              <button class="where" onclick={() => openNeed(n, o)}>
                {#if n.where.length}<MapPin size={11} /> Où ?{:else}Comment l'obtenir{/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if cost}
        <div class="muted small">
          À +{max} : {cost.runes.toLocaleString("fr-FR")} runes, {cost.materials.map((m) => `${m.count} ${m.name}`).join(", ")}
        </div>
      {/if}
    </div>
    <div class="side">
      <strong class="ar">{o.ar.total}</strong>
      {#if !mine && best}
        {@const d = o.ar.total - best.ar.total}
        <span class="gain" class:neg={d < 0}>{d >= 0 ? "+" : ""}{d} ({d >= 0 ? "+" : ""}{Math.round((d / Math.max(1, best.ar.total)) * 100)} %)</span>
      {/if}
      {#if mine && first}<span class="pill ok">la meilleure</span>{/if}
    </div>
  </li>
{/snippet}

<style>
  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 18px;
  }
  .tabs button {
    display: flex;
    align-items: center;
    gap: 6px;
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
    margin-bottom: 14px;
    max-width: 900px;
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
  h3 {
    margin: 0;
    font-family: var(--font-title);
    font-weight: 600;
  }
  p {
    margin: 4px 0 8px;
  }
  .lead {
    max-width: 900px;
    color: var(--text-dim);
  }
  .small {
    font-size: 12px;
  }
  .notice {
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 10px 12px;
    max-width: 700px;
  }
  .me {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }
  .stats {
    display: flex;
    gap: 6px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 38px;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-weight: 600;
  }
  .stat small {
    font-weight: 400;
    font-size: 10px;
    color: var(--muted);
  }
  .stat.main {
    border-color: var(--gold-dim);
    color: var(--gold);
  }
  .options {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .option {
    display: flex;
    gap: 12px;
    padding: 10px 0;
    border-top: 1px solid var(--border);
  }
  .option:first-child {
    border-top: none;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name {
    align-self: flex-start;
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-weight: 600;
    font-size: 15px;
    cursor: pointer;
    text-align: left;
  }
  .name:hover {
    text-decoration: underline;
  }
  .main > .muted.small:first-of-type {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    min-width: 110px;
  }
  .ar {
    font-size: 22px;
    color: var(--gold);
    font-variant-numeric: tabular-nums;
  }
  .gain {
    color: #6fbf73;
    font-size: 12px;
  }
  .gain.neg {
    color: var(--danger);
  }
  .warn {
    color: #f0c27a;
  }
  .needs {
    list-style: none;
    margin: 6px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .needs li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  .need-kind {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 0 6px;
  }
  .where {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: none;
    border: none;
    color: var(--gold);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    padding: 0;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 1px 8px;
    color: var(--text-dim);
    background: none;
    font-family: inherit;
  }
  .pill.ok {
    border-color: #5fae6a;
    color: #5fae6a;
  }
  button.pill.map {
    border-color: var(--gold-dim);
    color: var(--gold);
    cursor: pointer;
  }
  .armor {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
    margin-top: 10px;
  }
  @media (max-width: 760px) {
    .armor {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  .wicon {
    width: 56px;
    height: 56px;
    object-fit: contain;
    flex-shrink: 0;
    filter: drop-shadow(0 3px 6px rgba(0, 0, 0, 0.6));
  }
  .piece {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 3px;
    padding: 10px 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
  }
  .piece img,
  .piece .empty {
    width: 72px;
    height: 72px;
    object-fit: contain;
  }
  .piece .name {
    align-self: center;
    text-align: center;
    color: var(--text);
    font-size: 13px;
  }
  .totals {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    margin-top: 10px;
  }
  .kind {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
    color: var(--text-dim);
  }
  .pick {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    color: var(--text-dim);
  }
  .plan {
    width: 100%;
    border-collapse: collapse;
  }
  .plan th {
    text-align: left;
    font-size: 11px;
    font-weight: 500;
    color: var(--muted);
    padding: 4px 6px;
  }
  .plan td {
    padding: 8px 6px;
    border-top: 1px solid var(--border);
    vertical-align: top;
  }
  .num {
    text-align: right;
    white-space: nowrap;
  }
  .spread {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 4px 10px;
  }
  .spread .up {
    color: #6fbf73;
  }
  .spread .down {
    color: #f0a07a;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    margin-bottom: 8px;
  }
  .controls label {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-dim);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    background: none;
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    border-radius: 14px;
    padding: 2px 10px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .chip.active {
    border-color: var(--gold);
    color: var(--gold);
  }
</style>
