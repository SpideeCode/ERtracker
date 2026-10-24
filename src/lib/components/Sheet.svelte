<script lang="ts">
  // Side sheet frame: picks the right sheet for the entry kind.
  import { ui } from "$lib/ui.svelte";
  import { describe } from "$lib/tracker.svelte";
  import { gamedata } from "$lib/gamedata.svelte";
  import { spoiler } from "$lib/spoiler.svelte";
  import { ArrowLeft, EyeOff, X } from "@lucide/svelte";
  import BossSheet from "./BossSheet.svelte";
  import GraceSheet from "./GraceSheet.svelte";
  import ItemSheet from "./ItemSheet.svelte";
  import EquipSheet from "./EquipSheet.svelte";
  import PlaceSheet from "./PlaceSheet.svelte";

  let { id }: { id: string } = $props();
  const kind = $derived(id.split(":")[0]);
  const previous = $derived.by(() => {
    const prev = ui.sheetHistory.at(-1);
    if (!prev) return null;
    if (prev.startsWith("place:")) {
      const p = gamedata.places[Number(prev.slice(6))];
      return p ? (p.name.fr ?? p.name.en) : "Retour";
    }
    if (prev.startsWith("equip:")) return "Retour";
    return spoiler.name(prev, describe(prev).label);
  });
</script>

<aside class="sheet">
  {#if previous}
    <button class="back" onclick={() => ui.back()} title="Revenir à la fiche précédente (Alt+←)">
      <ArrowLeft size={14} /> <span>{previous}</span>
    </button>
  {/if}
  <button class="close" onclick={() => (ui.sheet = null)} title="Fermer (Échap)"><X size={16} /></button>
  {#if spoiler.hidden(id)}
    <div class="veil">
      <EyeOff size={28} />
      <h2>{spoiler.name(id, "")}</h2>
      <p class="muted">
        Mode sans spoil : tu ne l'as pas encore {kind === "boss" ? "vaincu" : kind === "grace" ? "découverte" : "ramassé"}, alors la fiche reste
        cachée.
      </p>
      <button onclick={() => spoiler.revealed.add(id)}>Afficher la fiche quand même</button>
    </div>
  {:else if kind === "boss"}
    <BossSheet {id} />
  {:else if kind === "grace"}
    <GraceSheet {id} />
  {:else if kind === "item"}
    <ItemSheet {id} />
  {:else if kind === "equip"}
    <EquipSheet {id} />
  {:else if kind === "place"}
    <PlaceSheet {id} />
  {/if}
</aside>

<style>
  .sheet {
    position: relative;
    height: 100%;
    overflow-y: auto;
    padding: 20px 20px 28px;
    background: var(--surface);
    border-left: 1px solid var(--border-strong);
    box-sizing: border-box;
  }
  .close {
    position: absolute;
    top: 10px;
    right: 12px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 1;
  }
  .close:hover,
  .back:hover {
    color: var(--text);
  }
  .back {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: calc(100% - 44px);
    background: none;
    border: none;
    padding: 0;
    margin-bottom: 4px;
    color: var(--text-dim);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .veil {
    margin-top: 40px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 6px;
    color: var(--muted);
  }
  .veil h2 {
    margin: 6px 0 0 !important;
  }
  .veil p {
    max-width: 300px;
  }
  .veil button {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 7px 14px;
    font: inherit;
    cursor: pointer;
  }
  .veil button:hover {
    border-color: var(--gold-dim);
  }
  .back span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Shared by the sheets rendered inside. */
  .sheet :global(h2) {
    margin: 10px 40px 2px 0;
    font-family: var(--font-title);
    font-weight: 500;
    font-size: 24px;
    line-height: 1.2;
    color: var(--gold);
  }
  .sheet :global(h3) {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin: 0 0 10px;
  }
  .sheet :global(.small) {
    font-size: 12px;
  }
  .sheet :global(.badges) {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .sheet :global(.badge) {
    font-size: 11px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 1px 8px;
    color: var(--text-dim);
  }
  .sheet :global(.badge.gold) {
    border-color: var(--gold);
    color: var(--gold);
  }
  .sheet :global(.badge.ok) {
    border-color: #5fae6a;
    color: #5fae6a;
  }
  .sheet :global(.meta) {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
    color: var(--text-dim);
  }
  .sheet :global(.meta span + span::before) {
    content: "·";
    margin-right: 6px;
    color: var(--muted);
  }
  .sheet :global(.card) {
    margin-top: 18px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
  }
  .sheet :global(.notice) {
    margin-top: 16px;
    border: 1px solid var(--gold-dim);
    border-radius: 6px;
    padding: 8px 10px;
  }
  .sheet :global(.links) {
    display: flex;
    gap: 8px;
    margin-top: 18px;
  }
  .sheet :global(.links button) {
    flex: 1;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text);
    border-radius: 6px;
    padding: 7px 10px;
    font: inherit;
    cursor: pointer;
  }
  .sheet :global(.links button:hover) {
    border-color: var(--gold-dim);
  }
</style>
