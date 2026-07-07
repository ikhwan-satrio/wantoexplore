import { invoke } from '@tauri-apps/api/core';
import { settings } from './settings.svelte';
import type { DirEntryInfo, TrashEntry } from '$lib/types';

export interface XdgDirs {
  home: string;
  desktop: string | null;
  documents: string | null;
  downloads: string | null;
  music: string | null;
  pictures: string | null;
  videos: string | null;
  templates: string | null;
  public_share: string | null;
}

export interface PinnedDir {
  name: string;
  path: string;
}

class NavigationStore {
  currentPath = $state<string>('/');
  currentView = $state<'dir' | 'trash'>('dir');
  entries = $state<DirEntryInfo[]>([]);
  trashEntries = $state<TrashEntry[]>([]);
  history = $state<string[]>(['/']);
  historyIndex = $state<number>(0);
  isLoading = $state(false);
  error = $state<string | null>(null);
  xdgDirs = $state<XdgDirs | null>(null);
  pinnedDirs = $state<PinnedDir[]>([]);

  canGoBack = $derived(this.historyIndex > 0);
  canGoForward = $derived(this.historyIndex < this.history.length - 1);
  breadcrumbs = $derived(
    this.currentView === 'trash' ? ['Trash'] : this.currentPath.split('/').filter(Boolean)
  );
  visibleEntries = $derived(
    this.currentView === 'trash'
      ? (this.trashEntries as any)
      : settings.showHiddenFiles
        ? this.entries
        : this.entries.filter((e) => !e.name.startsWith('.'))
  );

  async init() {
    try {
      this.xdgDirs = await invoke('app_get_xdg');
      this.pinnedDirs = await invoke('app_get_pinned');

      if (this.xdgDirs) {
        await this.navigateTo(this.xdgDirs.home);
      }
    } catch (e) {
      console.error('Failed to init navigation:', e);
    }
  }

  async navigateTo(path: string, addToHistory = true) {
    this.currentView = 'dir';
    this.isLoading = true;
    this.error = null;

    try {
      this.entries = await invoke('app_list_dir', { path });
      this.currentPath = path;
      await invoke('app_set_current_dir', { path });

      if (addToHistory) {
        this.history = this.history.slice(0, this.historyIndex + 1);
        this.history.push(path);
        this.historyIndex = this.history.length - 1;
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  // ─── Trash ──────────────────────────────────────────

  async navigateToTrash() {
    this.isLoading = true;
    this.error = null;
    this.currentView = 'trash';
    this.currentPath = 'trash://';
    try {
      this.trashEntries = await invoke('app_list_trash');
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  async refreshTrash() {
    await this.navigateToTrash();
  }

  async restoreTrashItem(id: string) {
    await invoke('app_restore_trash_item', { id });
    await this.refreshTrash();
  }

  async purgeTrashItem(id: string) {
    await invoke('app_purge_trash_item', { id });
    await this.refreshTrash();
  }

  async emptyTrash() {
    await invoke('app_empty_trash');
    await this.refreshTrash();
  }

  // ─── History ────────────────────────────────────────

  async goBack() {
    if (!this.canGoBack) return;
    this.historyIndex--;
    await this.navigateTo(this.history[this.historyIndex], false);
  }

  async goForward() {
    if (!this.canGoForward) return;
    this.historyIndex++;
    await this.navigateTo(this.history[this.historyIndex], false);
  }

  async goUp() {
    const parts = this.currentPath.split('/').filter(Boolean);
    if (parts.length === 0) return;
    parts.pop();
    await this.navigateTo('/' + parts.join('/') || '/');
  }

  async refresh() {
    if (this.currentView === 'trash') {
      await this.refreshTrash();
    } else {
      await this.navigateTo(this.currentPath, false);
    }
  }

  async openEntry(entry: DirEntryInfo) {
    if (entry.is_dir) {
      await this.navigateTo(entry.path);
    }
  }

  // ─── Pinned dirs ──────────────────────────────────

  async addPinned(name: string, path: string) {
    this.pinnedDirs = await invoke('app_add_pinned', { name, path });
  }

  async removePinned(path: string) {
    this.pinnedDirs = await invoke('app_remove_pinned', { path });
  }

  async reorderPinned(dirs: PinnedDir[]) {
    this.pinnedDirs = await invoke('app_reorder_pinned', { dirs });
  }

  isPinned(path: string): boolean {
    return this.pinnedDirs.some((d) => d.path === path);
  }
}

export const navigation = new NavigationStore();
