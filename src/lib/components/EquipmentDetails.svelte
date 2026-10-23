<script lang="ts">
  // Stats, ownership, upgrade costs and locations for one piece of equipment.
  import { catalog } from "$lib/catalog.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { tracker } from "$lib/tracker.svelte";
  import { DAMAGE_TYPES, STATS, estimate, grade, rarityOf, upgradeCost } from "$lib/equipment";
  import { pickupTitle } from "$lib/items";
  import { ui } from "$lib/ui.svelte";

  /** cat: 1 goods, 2 weapon, 3 armour, 4 talisman, 5 ash of war */
  let { cat, id }: { cat: number; id: number } = $props();

  const ARMOR_SLOTS = ["Tête", "Torse", "Bras", "Jambes"];
  const ABSORPTION = ["Physique", "Contondant", "Tranchant", "Perforant", "Magie", "Feu", "Foudre", "Sacré"];
  const ARMOR_RESIST = ["Immunité", "Robustesse", "Concentration", "Vitalité"];

  const weaponId = $derived(cat === 2 ? Math.floor(id / 10000) * 10000 : id);
  const weapon = $derived(cat === 2 ? catalog.weapons.get(weaponId) : undefined);
  const armor = $derived(cat === 3 ? catalog.armor.get(id) : undefined);
  const talisman = $derived(cat === 4 ? catalog.talismans.get(id) : undefined);
  const goods = $derived(cat === 1 ? catalog.goods.get(id) : undefined);
  const aow = $derived(cat === 5 ? catalog.ashesOfWar.get(id) : undefined);
  const info = $derived(weapon?.info ?? talisman?.info ?? goods?.info ?? aow?.info ?? null);
  const rarity = $derived(rarityOf(catalog.rarity(cat, id)));
  const owned = $derived(catalog.isOwned(cat, weaponId));
  const ownedLevel = $derived(cat === 2 ? catalog.owned.weaponLevel.get(weaponId) : undefined);
  const where = $derived(catalog.whereIs(cat, id));
  const stats = $derived(tracker.character?.stats);
  const damage = $derived(weapon ? estimate(weapon, stats) : null);

  let from = $state(0);
  let to = $state(0);
  $effect(() => {
    from = ownedLevel ?? 0;
    to = weapon?.max_level ?? 0;
  });
  const cost = $derived(weapon && gamedata.catalog && to > from ? upgradeCost(gamedata.catalog, weapon, from, to) : null);
</script>

<div class="details">
  <div class="tags">
    <span class="tag" style="color: {rarity.color}; border-color: {rarity.color}">{rarity.label}</span>
    {#if owned}
      <span class="tag ok">Possédé{ownedLevel !== undefined && weapon ? ` (+${ownedLevel})` : ""}</span>
    {:else}
      <span class="tag">Pas encore possédé</span>
    {/if}
  </div>

  {#if info}<p class="info">{info.fr ?? info.en}</p>{/if}

  {#if weapon && damage}
    <h4>Dégâts au niveau max (+{weapon.max_level})</h4>
    <div class="grid">
      {#each DAMAGE_TYPES as t, i}
        {#if weapon.attack_max[i]}
          <div class="cell"><span>{t}</span><strong>{weapon.attack_max[i]}</strong></div>
        {/if}
      {/each}
    </div>
    <p class="muted small">
      Estimation avec tes stats : <strong>{damage.total}</strong> de dégâts{damage.meetsRequirements ? "" : " — prérequis non atteints"}.
    </p>
    <h4>Scaling (max) et prérequis</h4>
    <div class="grid">
      {#each STATS as s, i}
        {#if weapon.scaling_max[i] || weapon.requirements[i]}
          <div class="cell" class:missing={stats && stats[i + 3] < weapon.requirements[i]}>
            <span>{s}</span>
            <strong>{grade(weapon.scaling_max[i])}</strong>
            <span class="muted small">{weapon.requirements[i] ? `requis ${weapon.requirements[i]}` : "—"}</span>
          </div>
        {/if}
      {/each}
    </div>
    <p class="muted small">
      Poids {weapon.weight} · {weapon.max_level === 10 ? "pierres sombres (+10 max)" : "pierres de forge (+25 max)"}{weapon.infusable
        ? " · peut changer de cendre et d'affinité"
        : ""}
    </p>

    {#if weapon.max_level > 0}
      <h4>Coût d'amélioration</h4>
      <div class="upgrade">
        <label>De +<select bind:value={from}>{#each Array(weapon.max_level + 1) as _, l}<option value={l}>{l}</option>{/each}</select></label>
        <label>à +<select bind:value={to}>{#each Array(weapon.max_level + 1) as _, l}<option value={l}>{l}</option>{/each}</select></label>
      </div>
      {#if cost}
        <ul class="cost">
          <li><strong>{cost.runes.toLocaleString("fr-FR")}</strong> runes</li>
          {#each cost.materials as m}
            <li><strong>{m.count} ×</strong> {m.name}</li>
          {/each}
        </ul>
      {:else}
        <p class="muted small">Choisis un niveau d'arrivée plus haut.</p>
      {/if}
    {/if}
  {/if}

  {#if armor}
    <h4>{ARMOR_SLOTS[armor.slot] ?? "Armure"} · poids {armor.weight}</h4>
    <div class="grid">
      {#each ABSORPTION as t, i}
        <div class="cell"><span>{t}</span><strong>{armor.absorption[i]}</strong></div>
      {/each}
    </div>
    <div class="grid">
      {#each ARMOR_RESIST as t, i}
        <div class="cell"><span>{t}</span><strong>{armor.resistance[i]}</strong></div>
      {/each}
    </div>
  {/if}

  {#if talisman}
    <p class="muted small">Poids {talisman.weight}</p>
  {/if}

  {#if goods?.spell}
    <h4>Sort</h4>
    <div class="grid">
      <div class="cell"><span>PC</span><strong>{goods.spell.fp}</strong></div>
      <div class="cell"><span>Emplacements</span><strong>{goods.spell.slots}</strong></div>
      {#each ["Intelligence", "Foi", "Arcane"] as s, i}
        {#if goods.spell.requirements[i]}
          <div class="cell" class:missing={stats && stats[[5, 6, 7][i]] < goods.spell.requirements[i]}>
            <span>{s} requise</span><strong>{goods.spell.requirements[i]}</strong>
          </div>
        {/if}
      {/each}
    </div>
  {/if}

  <h4>Où le trouver</h4>
  {#if where.length}
    <ul class="where">
      {#each where.slice(0, 5) as pid}
        {@const p = gamedata.items.get(pid)}
        {#if p}
          <li>
            <button class="place" onclick={() => (ui.sheet = pid)}>{pickupTitle(p)}</button>
            <span class="muted small">{tracker.completed.has(pid) ? "· déjà ramassé" : ""}</span>
          </li>
        {/if}
      {/each}
    </ul>
  {:else}
    <p class="muted small">Pas posé sur la carte : butin de boss, achat chez un marchand, quête ou échange de souvenir.</p>
  {/if}
</div>

<style>
  .details {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .tags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .tag {
    font-size: 11px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 1px 8px;
    color: var(--text-dim);
  }
  .tag.ok {
    border-color: #5fae6a;
    color: #5fae6a;
  }
  .info {
    margin: 4px 0;
    color: var(--text-dim);
  }
  h4 {
    margin: 10px 0 2px;
    font-size: 12px;
    color: var(--muted);
    font-weight: 600;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .cell {
    background: var(--surface-2);
    border-radius: 6px;
    padding: 5px 8px;
    display: flex;
    flex-direction: column;
    font-size: 12px;
    border: 1px solid transparent;
  }
  .cell strong {
    font-size: 14px;
  }
  .cell.missing {
    border-color: #a0503f;
  }
  .small {
    font-size: 12px;
  }
  .upgrade {
    display: flex;
    gap: 12px;
  }
  .upgrade select {
    margin-left: 4px;
    padding: 2px 6px;
  }
  .cost {
    list-style: none;
    padding: 0;
    margin: 4px 0;
  }
  .cost li {
    padding: 2px 0;
  }
  .where {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .place {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: var(--gold);
    cursor: pointer;
    text-decoration: underline dotted var(--gold-dim);
    text-underline-offset: 3px;
  }
</style>
