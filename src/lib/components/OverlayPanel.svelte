<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let visible = $state<boolean | null>(null);
  let error = $state<string | null>(null);

  async function toggle() {
    error = null;
    try {
      visible = await invoke<boolean>("toggle_overlay");
    } catch (e) {
      error = String(e);
    }
  }

  async function interact() {
    await invoke("set_overlay_interactive", { on: true });
  }
</script>

<p class="muted">
  Un petit panneau ERtracker affiché par-dessus le jeu : boss le plus proche (distance, direction, essais), ce qu'il
  reste dans ta zone, tes morts de la session, et les notifications quand tu bats un boss ou trouves un objet.
</p>

<section class="card">
  <h3>Comment ça marche</h3>
  <ol>
    <li>Dans Elden Ring : <strong>Système → Graphismes → Mode d'écran : Fenêtré sans bordure</strong> (le plein écran exclusif masque toutes les autres fenêtres).</li>
    <li>Active l'overlay ci-dessous ou avec <kbd>Ctrl</kbd>+<kbd>Maj</kbd>+<kbd>O</kbd>, même en jeu.</li>
    <li>Les clics passent à travers vers le jeu. Pour régler l'overlay (coin, opacité, taille) : <kbd>Ctrl</kbd>+<kbd>Maj</kbd>+<kbd>I</kbd>, puis encore une fois pour revenir au jeu.</li>
  </ol>
  <p class="muted small">
    C'est une fenêtre Windows séparée, transparente et toujours au premier plan : rien n'est injecté dans le jeu, l'anti-triche
    n'est pas concerné.
  </p>
  <div class="actions">
    <button class="primary" onclick={toggle}>{visible ? "Masquer l'overlay" : "Afficher l'overlay"}</button>
    <button class="ghost" onclick={interact}>Régler l'overlay</button>
  </div>
  {#if error}<p class="error">Échec : {error}</p>{/if}
</section>

<style>
  .card {
    margin-top: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    max-width: 760px;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 14px;
  }
  ol {
    margin: 0 0 8px;
    padding-left: 20px;
  }
  li {
    margin: 6px 0;
  }
  kbd {
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 0 5px;
    font-size: 12px;
  }
  .small {
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .error {
    color: var(--danger);
  }
</style>
