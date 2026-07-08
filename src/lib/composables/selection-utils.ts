import { selection } from '$lib/stores/selection.svelte';
import type { ContextMenuItem } from '$lib/components/ContextMenu.svelte';

export function getAffectedIds<T>(clickedId: T, selectionPaths: Set<T>): T[] {
  if (selectionPaths.size > 1 && selectionPaths.has(clickedId)) {
    return Array.from(selectionPaths);
  }
  return [clickedId];
}

export function handleItemClick(id: string, e?: MouseEvent | KeyboardEvent) {
  if (e?.ctrlKey || e?.metaKey) {
    selection.toggle(id);
  } else {
    selection.select(id);
  }
}

export function clearSelectionIfActive() {
  if (selection.paths.size > 0) selection.clear();
}
