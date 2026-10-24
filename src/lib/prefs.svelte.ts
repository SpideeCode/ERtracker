// User preferences kept in this browser profile (convenience only).
const KEY = "ertracker-prefs";

/** How the player plays, which shapes the home page and the menu. */
export type Profile = "discover" | "complete" | "build";

export const PROFILES: { id: Profile; title: string; text: string }[] = [
  {
    id: "discover",
    title: "Je découvre le jeu",
    text: "Pas de spoil : les noms restent cachés tant que tu ne les as pas trouvés. L'appli t'aide surtout à savoir où aller.",
  },
  {
    id: "complete",
    title: "Je vise le 100 %",
    text: "Tout est visible : ce qui reste par zone, les donjons ratés, les quêtes en cours et les objets manquants.",
  },
  {
    id: "build",
    title: "NG+ ou nouveau build",
    text: "Tu connais le jeu : l'appli met en avant tes armes, ce qui améliorerait ton build et où le récupérer.",
  },
];

interface Prefs {
  sidebarCollapsed: boolean;
  /** Hide names of what has not been found yet. */
  spoilerFree: boolean;
  /** The first-launch tour was seen. */
  welcomed: boolean;
  profile: Profile | null;
}

const defaults: Prefs = { sidebarCollapsed: false, spoilerFree: false, welcomed: false, profile: null };

function read(): Prefs {
  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return defaults;
  }
}

class PrefsStore {
  sidebarCollapsed = $state(defaults.sidebarCollapsed);
  spoilerFree = $state(defaults.spoilerFree);
  welcomed = $state(defaults.welcomed);
  profile = $state<Profile | null>(defaults.profile);

  load() {
    const p = read();
    this.sidebarCollapsed = p.sidebarCollapsed;
    this.spoilerFree = p.spoilerFree;
    this.welcomed = p.welcomed;
    this.profile = p.profile;
  }

  /** Picking a profile also sets its natural spoiler setting. */
  setProfile(profile: Profile) {
    this.profile = profile;
    this.spoilerFree = profile === "discover";
    this.save();
  }

  save() {
    try {
      localStorage.setItem(
        KEY,
        JSON.stringify({
          sidebarCollapsed: this.sidebarCollapsed,
          spoilerFree: this.spoilerFree,
          welcomed: this.welcomed,
          profile: this.profile,
        }),
      );
    } catch {
      /* preferences are a convenience */
    }
  }
}

export const prefs = new PrefsStore();
