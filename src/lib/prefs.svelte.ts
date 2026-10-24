// User preferences kept in this browser profile (convenience only).
const KEY = "ertracker-prefs";

interface Prefs {
  sidebarCollapsed: boolean;
  /** Hide names of what has not been found yet. */
  spoilerFree: boolean;
  /** The first-launch tour was seen. */
  welcomed: boolean;
}

const defaults: Prefs = { sidebarCollapsed: false, spoilerFree: false, welcomed: false };

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

  load() {
    const p = read();
    this.sidebarCollapsed = p.sidebarCollapsed;
    this.spoilerFree = p.spoilerFree;
    this.welcomed = p.welcomed;
  }

  save() {
    try {
      localStorage.setItem(KEY, JSON.stringify({ sidebarCollapsed: this.sidebarCollapsed, spoilerFree: this.spoilerFree, welcomed: this.welcomed }));
    } catch {
      /* preferences are a convenience */
    }
  }
}

export const prefs = new PrefsStore();
