const STORAGE_KEY = 'teddypicker-settings';

interface SettingsData {
  thumbnailEnabled: boolean;
  showHiddenFiles: boolean;
}

class SettingsStore {
  thumbnailEnabled = $state(true);
  showHiddenFiles = $state(false);

  constructor() {
    this.load();
  }

  private load() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const data: SettingsData = JSON.parse(raw);
        this.thumbnailEnabled = data.thumbnailEnabled ?? true;
        this.showHiddenFiles = data.showHiddenFiles ?? false;
      }
    } catch {
      // ignore
    }
  }

  private save() {
    try {
      const data: SettingsData = {
        thumbnailEnabled: this.thumbnailEnabled,
        showHiddenFiles: this.showHiddenFiles,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
    } catch {
      // ignore
    }
  }

  setThumbnailEnabled(enabled: boolean) {
    this.thumbnailEnabled = enabled;
    this.save();
  }

  setShowHiddenFiles(show: boolean) {
    this.showHiddenFiles = show;
    this.save();
  }
}

export const settings = new SettingsStore();
