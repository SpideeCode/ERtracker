<script lang="ts">
  // Sheet for a catalogue item ("equip:<cat>:<id>"), e.g. from the build advisor.
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { catalog } from "$lib/catalog.svelte";
  import EquipmentDetails from "./EquipmentDetails.svelte";

  let { id }: { id: string } = $props();

  const KINDS: Record<number, string> = { 1: "Objet", 2: "Arme", 3: "Armure", 4: "Talisman", 5: "Cendre de guerre" };

  const parts = $derived(id.split(":"));
  const cat = $derived(Number(parts[1]));
  const itemId = $derived(Number(parts[2]));
  const name = $derived(
    (cat === 2
      ? catalog.weapons.get(itemId)?.name
      : cat === 3
        ? catalog.armor.get(itemId)?.name
        : cat === 4
          ? catalog.talismans.get(itemId)?.name
          : cat === 1
            ? catalog.goods.get(itemId)?.name
            : catalog.ashesOfWar.get(itemId)?.name) ?? { en: "?", fr: null },
  );
  const wiki = $derived(`https://eldenring.wiki.fextralife.com/${encodeURIComponent(name.en).replace(/%20/g, "+")}`);
</script>

<header>
  <div class="badges"><span class="badge">{KINDS[cat] ?? "Objet"}</span></div>
  <h2>{name.fr ?? name.en}</h2>
  {#if name.fr && name.fr !== name.en}<div class="muted small">{name.en}</div>{/if}
</header>

<section class="card">
  <EquipmentDetails {cat} id={itemId} />
</section>

<section class="links">
  <button onclick={() => openUrl(wiki)}>Wiki ↗</button>
</section>
