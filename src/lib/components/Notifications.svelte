<script lang="ts">
  // Header bell: everything the save gained or lost since the app started.
  import { tracker } from "$lib/tracker.svelte";
  import { achievement, ago } from "$lib/labels";
  import { ui } from "$lib/ui.svelte";
  import { Bell } from "@lucide/svelte";

  let { now }: { now: number } = $props();
  let open = $state(false);
  let seenAt = $state(Date.now());
  let root: HTMLDivElement;

  const unread = $derived(tracker.log.filter((l) => l.gained && l.at > seenAt).length);

  function toggle() {
    open = !open;
    seenAt = Date.now();
  }

  function onWindowClick(e: MouseEvent) {
    if (open && !root.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="bell" bind:this={root}>
  <button class="icon" class:active={open} onclick={toggle} title="Notifications">
    <Bell size={18} />
    {#if unread}<span class="badge">{unread > 99 ? "99+" : unread}</span>{/if}
  </button>
  {#if open}
    <div class="menu">
      <div class="head">Cette session</div>
      {#if tracker.log.length}
        <ul>
          {#each tracker.log as l (l.at + l.id + l.gained)}
            <li>
              <button
                onclick={() => {
                  ui.open(l.id);
                  open = false;
                }}
              >
                <span class="sign" class:lost={!l.gained}>{l.gained ? "+" : "−"}</span>
                <span class="text">
                  <span class="label">{l.label}</span>
                  <span class="sub">{l.gained ? achievement(l.category) : "Retiré de la sauvegarde"} · {ago(l.at, now)}</span>
                </span>
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">Rien pour l'instant. Ce que tu bats, découvres ou ramasses apparaîtra ici à chaque sauvegarde du jeu.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .bell {
    position: relative;
  }
  .icon {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    background: none;
    border: 1px solid transparent;
    border-radius: 8px;
    color: var(--text-dim);
    cursor: pointer;
  }
  .icon:hover,
  .icon.active {
    color: var(--text);
    border-color: var(--border-strong);
  }
  .badge {
    position: absolute;
    top: 1px;
    right: 0;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    box-sizing: border-box;
    border-radius: 8px;
    background: var(--gold);
    color: #1a1508;
    font-size: 10px;
    font-weight: 700;
    line-height: 16px;
    text-align: center;
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 3000;
    width: 340px;
    max-height: 70vh;
    overflow-y: auto;
    background: var(--surface);
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    padding: 6px;
  }
  .head {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--gold);
    padding: 4px 6px 6px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li button {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    background: none;
    border: none;
    border-radius: 6px;
    padding: 6px;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  li button:hover {
    background: var(--surface-2);
  }
  .sign {
    color: #5fae6a;
    font-weight: 700;
    width: 10px;
  }
  .sign.lost {
    color: var(--danger);
  }
  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }
  .empty {
    margin: 0;
    padding: 6px 6px 8px;
    font-size: 13px;
    color: var(--text-dim);
  }
</style>
