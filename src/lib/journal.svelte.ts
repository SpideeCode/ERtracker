// The play journal of the current character, refreshed after every save.
import { invoke } from "@tauri-apps/api/core";
import type { JournalRecord } from "./history";

class Journal {
  records = $state<JournalRecord[]>([]);

  async reload() {
    try {
      this.records = await invoke<JournalRecord[]>("get_journal");
    } catch {
      this.records = [];
    }
  }
}

export const journal = new Journal();
