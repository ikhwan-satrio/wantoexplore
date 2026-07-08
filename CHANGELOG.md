# Changelog

## [1.2.0] - 2026-07-08

### Added
- **Archive support** — compress and extract ZIP, Tar, Tar.Gz, Tar.Bz2, Tar.XZ, Tar.Zst archives
- **Context menu actions** — Extract Here (for archives), Compress (for files/folders)
- **Delete hotkey** — press `Delete` to move selected files to trash
- **Permanent delete for large files** — files totaling >= 1 GB trigger a confirmation dialog and bypass trash
- **Keyboard shortcuts** — `Escape` to clear selection, `Mod+A` to select all
- **Progress streaming** — archive compress/extract operations now emit real-time progress events to the UI toast
- **Non-blocking archive operations** — compress/extract run on Tokio's thread pool via `spawn_blocking`, keeping the UI responsive

### Changed
- **Component split** — `FileGrid` refactored into smaller components: `FileGridToolbar`, `FileGridView`, `FileListView`, `TrashView` for easier debugging
- **Mutation pattern** — all async actions (delete, paste, rename, create, restore, purge, empty trash, compress, extract) use `@tanstack/svelte-query` mutations with `svelte-sonner` toast feedback
- **Hotkey library** — migrated to `@tanstack/svelte-hotkeys` for keyboard shortcut management
- **State management** — shared selection state extracted to `stores/selection.svelte.ts`
- **Context menu** — fixed Svelte 5 reactivity by using plain `$state` variables instead of `$state({...methods})` objects

### Fixed
- **Context menu crash** — `$state()` with methods inside helper functions caused infinite re-render loops; fixed by inlining state in component
- **Type errors** — `ComponentType` from `svelte` replaced with `ComponentType` from `svelte/component` for lucide-svelte icon compatibility
- **Accessibility warnings** — trash list rows now have `role="button"`, `tabindex`, and `onkeydown` handlers
