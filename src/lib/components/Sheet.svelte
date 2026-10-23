<script lang="ts">
  // Side sheet frame: picks the right sheet for the entry kind.
  import { ui } from "$lib/ui.svelte";
  import BossSheet from "./BossSheet.svelte";
  import GraceSheet from "./GraceSheet.svelte";
  import ItemSheet from "./ItemSheet.svelte";

  let { id }: { id: string } = $props();
  const kind = $derived(id.split(":")[0]);
</script>

<aside class="sheet">
  <button class="close" onclick={() => (ui.sheet = null)} title="Fermer (Échap)">×</button>
  {#if kind === "boss"}
    <BossSheet {id} />
  {:else if kind === "grace"}
    <GraceSheet {id} />
  {:else if kind === "item"}
    <ItemSheet {id} />
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
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
    z-index: 1;
  }
  .close:hover {
    color: var(--text);
  }
  /* Shared by the sheets rendered inside. */
  .sheet :global(h2) {
    margin: 10px 40px 2px 0;
    font-family: Georgia, serif;
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
