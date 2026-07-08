let selectedPaths = $state<Set<string>>(new Set());

export const selection = {
  get paths() { return selectedPaths; },
  get count() { return selectedPaths.size; },
  get hasSelection() { return selectedPaths.size > 0; },

  toggle(path: string) {
    const next = new Set(selectedPaths);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    selectedPaths = next;
  },

  select(path: string) {
    selectedPaths = new Set([path]);
  },

  selectAll(paths: string[]) {
    selectedPaths = new Set(paths);
  },

  clear() {
    selectedPaths = new Set();
  },

  isSelected(path: string): boolean {
    return selectedPaths.has(path);
  },
};
