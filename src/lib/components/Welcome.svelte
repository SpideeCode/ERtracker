<script lang="ts">
  // First-launch tour: what was found on this PC, where things are, how to use it while playing.
  import { Compass, Map, Trophy, Telescope, BookOpen, Swords, Check, LoaderCircle, CircleAlert, Settings } from "@lucide/svelte";
  import { tracker } from "$lib/tracker.svelte";
  import { mapdata } from "$lib/mapdata.svelte";
  import { prefs } from "$lib/prefs.svelte";
  import { playtime } from "$lib/labels";

  let step = $state(0);
  const STEPS = 3;

  const character = $derived(tracker.character);
  const gameDir = $derived(mapdata.status?.game_dir ?? null);
  const dataReady = $derived(!!mapdata.status?.manifest && !mapdata.running);

  const spaces = [
    { icon: Compass, name: "Maintenant", text: "où tu es, le prochain boss, ta session, où aller ensuite" },
    { icon: Map, name: "Carte", text: "les 4 cartes du jeu, avec calques, légende et recherche" },
    { icon: Trophy, name: "Progression", text: "boss, objets, grâces, quêtes et collections, coché depuis ta sauvegarde" },
    { icon: Telescope, name: "Explorer", text: "les zones où il te reste le plus à faire, sans spoil" },
    { icon: BookOpen, name: "Mon aventure", text: "tes sessions, tes morts par boss, ton trajet" },
    { icon: Swords, name: "Builds", text: "les meilleures combinaisons pour un style, et le coût des améliorations" },
  ];

  function done() {
    prefs.welcomed = true;
    prefs.save();
  }
</script>

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="welcome-title">
    {#if step === 0}
      <h2 id="welcome-title">Bienvenue, Sans-éclat</h2>
      <p>
        ERtracker suit ta partie d'Elden Ring en direct : à chaque sauvegarde automatique du jeu, il relit ton fichier et
        coche ce que tu as vaincu, découvert et ramassé.
      </p>
      <ul class="checks">
        <li>
          {#if character}
            <Check size={16} class="ok" />
            <span>Sauvegarde trouvée : <strong>{character.name}</strong>, niveau {character.level}, {playtime(character.seconds_played)}</span>
          {:else}
            <CircleAlert size={16} class="warn" />
            <span>Pas encore de personnage : lance le jeu une fois pour créer ta sauvegarde.</span>
          {/if}
        </li>
        <li>
          {#if dataReady}
            <Check size={16} class="ok" />
            <span>Cartes et données lues depuis ton jeu{gameDir ? ` (${gameDir})` : ""}</span>
          {:else if mapdata.running}
            <LoaderCircle size={16} class="spin" />
            <span>Lecture des cartes et des données de ton jeu… {mapdata.percent} %</span>
          {:else}
            <CircleAlert size={16} class="warn" />
            <span>Cartes pas encore extraites : ouvre la Carte pour indiquer où est installé le jeu.</span>
          {/if}
        </li>
      </ul>
      <p class="muted small">Rien n'est envoyé sur Internet, et le jeu n'est jamais modifié pendant que tu joues.</p>
    {:else if step === 1}
      <h2 id="welcome-title">Six espaces, dans le menu à gauche</h2>
      <ul class="spaces">
        {#each spaces as s (s.name)}
          {@const Icon = s.icon}
          <li>
            <Icon size={18} strokeWidth={1.75} />
            <span><strong>{s.name}</strong> : {s.text}</span>
          </li>
        {/each}
      </ul>
      <p class="muted small">Clique sur n'importe quel nom de boss, de grâce ou d'objet pour ouvrir sa fiche : comment y aller, points faibles, conditions d'accès.</p>
    {:else}
      <h2 id="welcome-title">Pendant que tu joues</h2>
      <p>
        Laisse ERtracker ouvert : il se met à jour seul et tient ton journal (sessions, morts, trajet). Avec le jeu en
        <strong>fenêtre sans bordure</strong>, un petit panneau peut s'afficher par-dessus.
      </p>
      <ul class="keys">
        <li><kbd>Ctrl</kbd> <kbd>Maj</kbd> <kbd>O</kbd> afficher ou masquer l'overlay en jeu</li>
        <li><kbd>Ctrl</kbd> <kbd>Maj</kbd> <kbd>I</kbd> cliquer dans l'overlay, puis revenir au jeu</li>
        <li><kbd>Ctrl</kbd> <kbd>K</kbd> rechercher un boss, une grâce, un objet, un donjon</li>
        <li><kbd>M</kbd> ouvrir la carte · <kbd>Échap</kbd> fermer une fiche · <kbd>Alt</kbd> <kbd>←</kbd> fiche précédente</li>
      </ul>
      <p class="muted small with-icon"><Settings size={14} /> Le mode sans spoil et l'overlay se règlent dans Réglages.</p>
    {/if}

    <div class="foot">
      <div class="dots">
        {#each Array(STEPS) as _, i}<span class:on={i === step}></span>{/each}
      </div>
      {#if step < STEPS - 1}
        <button class="link" onclick={done}>Passer</button>
        <button class="primary" onclick={() => step++}>Suivant</button>
      {:else}
        <button class="ghost" onclick={() => step--}>Retour</button>
        <button class="primary" onclick={done}>C'est parti</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 4000;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }
  .dialog {
    width: min(560px, 100%);
    background: var(--surface);
    border: 1px solid var(--gold-dim);
    border-radius: 12px;
    padding: 26px 28px 20px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
    animation: rise 0.2s ease-out;
  }
  @keyframes rise {
    from {
      transform: translateY(12px);
      opacity: 0;
    }
  }
  h2 {
    margin: 0 0 12px;
    font-family: var(--font-title);
    font-weight: 600;
    color: var(--gold);
    font-size: 22px;
  }
  p {
    margin: 0 0 12px;
    color: var(--text-dim);
  }
  .small {
    font-size: 12px;
  }
  ul {
    list-style: none;
    margin: 0 0 14px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }
  li :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }
  .spaces :global(svg) {
    color: var(--gold);
  }
  .checks :global(.ok) {
    color: #5fae6a;
  }
  .checks :global(.warn) {
    color: #d39b45;
  }
  .checks :global(.spin) {
    color: var(--gold);
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .keys li {
    display: block;
    color: var(--text-dim);
  }
  kbd {
    display: inline-block;
    min-width: 18px;
    padding: 1px 6px;
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;
    border-radius: 4px;
    background: var(--surface-2);
    color: var(--text);
    font: 12px/1.4 inherit;
    text-align: center;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 18px;
  }
  .dots {
    display: flex;
    gap: 6px;
    margin-right: auto;
  }
  .dots span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--border-strong);
  }
  .dots span.on {
    background: var(--gold);
  }
  .link {
    background: none;
    border: none;
    color: var(--muted);
    font: inherit;
    cursor: pointer;
  }
  .link:hover {
    color: var(--text);
  }
</style>
