<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { readFile } from '@tauri-apps/plugin-fs';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';

	import { navigation } from '$lib/stores/navigation.svelte';
	import { settings } from '$lib/stores/settings.svelte';
	import { dragStore } from '$lib/stores/drag.svelte';
	import { fileClipboard } from '$lib/stores/clipboard.svelte';
	import { selection } from '$lib/stores/selection.svelte';
	import { getAffectedIds } from '$lib/composables/selection-utils';
	import { useCompressMutation, useExtractMutation } from '$lib/queries/archive';
	import {
		useDeleteMutation,
		usePasteMutation,
		useRenameMutation,
		useCreateDirMutation,
		useCreateFileMutation,
		useRestoreTrashMutation,
		usePurgeTrashMutation,
		useEmptyTrashMutation,
		usePermanentDeleteMutation,
		getTotalSize,
		formatFileSize
	} from '$lib/queries/mutations';
	import { createHotkey } from '@tanstack/svelte-hotkeys';
	import type { ContextMenuItem } from './ContextMenu.svelte';
	import ContextMenu from './ContextMenu.svelte';
	import OpenWithPopup from './OpenWithPopup.svelte';
	import InputPopup from './InputPopup.svelte';
	import FileGridToolbar from './FileGridToolbar.svelte';
	import TrashView from './TrashView.svelte';
	import FileGridView from './FileGridView.svelte';
	import FileListView from './FileListView.svelte';
	import {
		FolderOpen,
		Pencil,
		Trash2,
		Copy,
		Scissors,
		Pin,
		PinOff,
		RotateCw,
		FolderPlus,
		FilePlus,
		ClipboardPaste,
		CheckCheck,
		Archive,
		FileDown,
		Image,
		FileVideo,
		FileAudio
	} from 'lucide-svelte';
	import type { DirEntryInfo } from '$lib/types';
	import type { ComponentType } from 'svelte';

	let viewMode = $state<'grid' | 'list'>('grid');

	let ctxOpen = $state(false);
	let ctxX = $state(0);
	let ctxY = $state(0);
	let ctxItems = $state<ContextMenuItem[]>([]);

	const compressMutation = useCompressMutation();
	const extractMutation = useExtractMutation();
	const deleteMutation = useDeleteMutation();
	const pasteMutation = usePasteMutation();
	const renameMutation = useRenameMutation();
	const createDirMutation = useCreateDirMutation();
	const createFileMutation = useCreateFileMutation();
	const restoreTrashMutation = useRestoreTrashMutation();
	const purgeTrashMutation = usePurgeTrashMutation();
	const emptyTrashMutation = useEmptyTrashMutation();
	const permanentDeleteMutation = usePermanentDeleteMutation();

	let openWithPath = $state('');
	let openWithOpen = $state(false);

	let renamePopup = $state({ open: false, path: '', name: '' });
	let createFolderPopup = $state({ open: false });
	let createFilePopup = $state({ open: false });

	let confirmDeleteOpen = $state(false);
	let confirmDeletePaths = $state<string[]>([]);
	let confirmDeleteSize = $state(0);
	let confirmDeleteLoading = $state(false);

	let thumbnailUrls = $state<Map<string, string>>(new Map());

	const mediaExts = new Set([
		'png',
		'jpg',
		'jpeg',
		'gif',
		'bmp',
		'webp',
		'ico',
		'tiff',
		'tif',
		'avif',
		'mp4',
		'avi',
		'mov',
		'mkv',
		'webm',
		'm4v',
		'3gp',
		'mp3',
		'flac',
		'ogg',
		'aac',
		'wav',
		'm4a'
	]);

	function isMedia(entry: DirEntryInfo): boolean {
		return !entry.is_dir && !!entry.extension && mediaExts.has(entry.extension.toLowerCase());
	}

	function isArchiveFile(entry: DirEntryInfo): boolean {
		if (entry.is_dir) return false;
		const name = entry.name.toLowerCase();
		return (
			name.endsWith('.zip') ||
			name.endsWith('.tar') ||
			name.endsWith('.tar.gz') ||
			name.endsWith('.tgz') ||
			name.endsWith('.tar.bz2') ||
			name.endsWith('.tbz2') ||
			name.endsWith('.tar.xz') ||
			name.endsWith('.txz') ||
			name.endsWith('.tar.zst') ||
			name.endsWith('.tzst')
		);
	}

	function getMediaIcon(ext: string): ComponentType {
		if (['mp4', 'avi', 'mov', 'mkv', 'webm', 'm4v', '3gp'].includes(ext)) return FileVideo;
		if (['mp3', 'flac', 'ogg', 'aac', 'wav', 'm4a'].includes(ext)) return FileAudio;
		return Image;
	}

	let pending = new Set<string>();

	async function loadThumbnail(path: string) {
		if (pending.has(path) || thumbnailUrls.has(path)) return;
		pending.add(path);
		try {
			const cachePath = await invoke<string>('app_get_thumbnail', { path });
			const bytes = await readFile(cachePath);
			const blob = new Blob([bytes], { type: 'image/jpeg' });
			const url = URL.createObjectURL(blob);
			const next = new Map(thumbnailUrls);
			next.set(path, url);
			thumbnailUrls = next;
		} catch {
			const next = new Map(thumbnailUrls);
			next.set(path, '');
			thumbnailUrls = next;
		} finally {
			pending.delete(path);
		}
	}

	function getThumbnail(entry: DirEntryInfo): string | undefined {
		const val = thumbnailUrls.get(entry.path);
		return val === '' ? undefined : val;
	}

	$effect(() => {
		navigation.entries;
		return () => {
			for (const url of thumbnailUrls.values()) {
				if (url && url.startsWith('blob:')) URL.revokeObjectURL(url);
			}
			thumbnailUrls = new Map();
			pending = new Set();
		};
	});

	$effect(() => {
		if (!settings.thumbnailEnabled) return;
		const entries = navigation.visibleEntries;
		const cancelled = { value: false };
		const mediaPaths: string[] = [];
		for (const e of entries) {
			if (isMedia(e) && !thumbnailUrls.has(e.path) && !pending.has(e.path)) {
				mediaPaths.push(e.path);
			}
		}

		async function loadAll() {
			const MAX = 8;
			async function worker(start: number) {
				for (let i = start; i < mediaPaths.length; i += MAX) {
					if (cancelled.value) return;
					await loadThumbnail(mediaPaths[i]);
				}
			}
			const workers: Promise<void>[] = [];
			for (let i = 0; i < Math.min(MAX, mediaPaths.length); i++) {
				workers.push(worker(i));
			}
			await Promise.all(workers);
		}

		if (mediaPaths.length > 0) loadAll();

		return () => {
			cancelled.value = true;
		};
	});

	function showContextMenu(e: MouseEvent, items: ContextMenuItem[]) {
		e.preventDefault();
		if (ctxOpen) {
			ctxOpen = false;
			return;
		}
		ctxX = e.clientX;
		ctxY = e.clientY;
		ctxItems = items;
		ctxOpen = true;
	}

	function closeContextMenu() {
		ctxOpen = false;
	}

	function buildFileMenuItems(entry: DirEntryInfo): ContextMenuItem[] {
		const items: ContextMenuItem[] = [];
		const affectedPaths = getAffectedIds(entry.path, selection.paths);
		const isMulti = affectedPaths.length > 1;

		if (!isMulti && entry.is_dir) {
			items.push({
				label: 'Open',
				icon: FolderOpen,
				action: () => navigation.navigateTo(entry.path)
			});
		} else if (!isMulti && !entry.is_dir) {
			items.push({
				label: 'Open With',
				icon: FolderOpen,
				action: () => {
					openWithPath = entry.path;
					openWithOpen = true;
				}
			});
		}

		if (!isMulti && entry.is_dir) {
			const pinned = navigation.isPinned(entry.path);
			items.push({
				label: pinned ? 'Unpin' : 'Pin to Sidebar',
				icon: pinned ? PinOff : Pin,
				action: async () => {
					if (pinned) {
						await navigation.removePinned(entry.path);
					} else {
						await navigation.addPinned(entry.name, entry.path);
					}
				}
			});
		}

		items.push({
			label: isMulti ? `Copy (${affectedPaths.length} items)` : 'Copy',
			icon: Copy,
			action: () => {
				fileClipboard.copy(affectedPaths);
				selection.clear();
			}
		});

		items.push({
			label: isMulti ? `Cut (${affectedPaths.length} items)` : 'Cut',
			icon: Scissors,
			action: () => {
				fileClipboard.cut(affectedPaths);
				selection.clear();
			}
		});

		if (!isMulti) {
			items.push({
				label: 'Rename',
				icon: Pencil,
				action: () => {
					renamePopup = { open: true, path: entry.path, name: entry.name };
				}
			});
		}

		items.push({
			label: isMulti ? `Move to Trash (${affectedPaths.length} items)` : 'Move to Trash',
			icon: Trash2,
			action: () => deleteMutation.mutate(affectedPaths)
		});

		items.push({ separator: true, label: '', action: () => {} });

		if (!isMulti && isArchiveFile(entry)) {
			items.push({
				label: 'Extract Here',
				icon: FileDown,
				action: () => extractMutation.mutate({ archivePath: entry.path })
			});
		}

		items.push({
			label: isMulti ? `Compress (${affectedPaths.length} items)` : 'Compress',
			icon: Archive,
			action: () => compressMutation.mutate({ sources: affectedPaths, format: 'zip' })
		});

		items.push({
			label: isMulti ? `Copy Paths (${affectedPaths.length})` : 'Copy Path',
			icon: Copy,
			action: () => {
				writeText(affectedPaths.join('\n'));
				selection.clear();
			}
		});

		return items;
	}

	function buildEmptyMenuItems(): ContextMenuItem[] {
		const items: ContextMenuItem[] = [];

		items.push({
			label: 'Create Folder',
			icon: FolderPlus,
			action: () => {
				createFolderPopup = { open: true };
			}
		});

		items.push({
			label: 'Create File',
			icon: FilePlus,
			action: () => {
				createFilePopup = { open: true };
			}
		});

		if (fileClipboard.hasItems) {
			items.push({ separator: true, label: '', action: () => {} });

			items.push({
				label: fileClipboard.operation === 'copy' ? 'Paste' : 'Paste (Move)',
				icon: ClipboardPaste,
				action: () =>
					pasteMutation.mutate({
						operation: fileClipboard.operation,
						sources: fileClipboard.items,
						dest: navigation.currentPath
					})
			});
		}

		items.push({ separator: true, label: '', action: () => {} });

		items.push({
			label: 'Select All',
			icon: CheckCheck,
			action: () => {
				selection.selectAll(navigation.visibleEntries.map((en: any) => en.path));
			}
		});

		return items;
	}

	function buildTrashMenuItems(entryId: string): ContextMenuItem[] {
		const items: ContextMenuItem[] = [];
		const affectedIds = getAffectedIds(entryId, selection.paths);
		const isMulti = affectedIds.length > 1;

		items.push({
			label: isMulti ? `Restore (${affectedIds.length} items)` : 'Restore',
			icon: RotateCw,
			action: () => restoreTrashMutation.mutate(affectedIds)
		});

		items.push({
			label: isMulti ? `Delete Permanently (${affectedIds.length} items)` : 'Delete Permanently',
			icon: Trash2,
			action: () => purgeTrashMutation.mutate(affectedIds)
		});

		return items;
	}

	function buildTrashEmptyMenuItems(): ContextMenuItem[] {
		const items: ContextMenuItem[] = [];

		if (navigation.trashEntries.length > 0) {
			items.push({
				label: `Restore All (${navigation.trashEntries.length} items)`,
				icon: RotateCw,
				action: () => restoreTrashMutation.mutate(navigation.trashEntries.map((e: any) => e.id))
			});

			items.push({
				label: 'Empty Trash',
				icon: Trash2,
				action: () => emptyTrashMutation.mutate()
			});
		}

		if (selection.paths.size > 0) {
			items.push({ separator: true, label: '', action: () => {} });

			items.push({
				label: `Restore Selected (${selection.paths.size})`,
				icon: RotateCw,
				action: () => restoreTrashMutation.mutate(Array.from(selection.paths))
			});

			items.push({
				label: `Delete Selected Permanently (${selection.paths.size})`,
				icon: Trash2,
				action: () => purgeTrashMutation.mutate(Array.from(selection.paths))
			});
		}

		return items;
	}

	function openTrashContextMenu(e: MouseEvent, entryId: string | null) {
		showContextMenu(e, entryId ? buildTrashMenuItems(entryId) : buildTrashEmptyMenuItems());
	}

	function openFileContextMenu(e: MouseEvent, entry: DirEntryInfo | null) {
		showContextMenu(e, entry ? buildFileMenuItems(entry) : buildEmptyMenuItems());
	}

	function handleDragStart(e: DragEvent, entry: DirEntryInfo) {
		if (!e.dataTransfer) return;
		const fileUri = `file://${entry.path}`;
		e.dataTransfer.setData('text/uri-list', fileUri);
		e.dataTransfer.setData('text/plain', entry.path);
		if (entry.is_dir) {
			dragStore.startDrag(entry.name, entry.path);
			e.dataTransfer.effectAllowed = 'copyLink';
		} else {
			e.dataTransfer.effectAllowed = 'copy';
		}
	}

	function handleDragEnd() {
		dragStore.endDrag();
	}

	function handleDblClick(entry: DirEntryInfo) {
		if (entry.is_dir) {
			navigation.openEntry(entry);
		} else {
			invoke('app_open_default', { path: entry.path });
		}
	}

	createHotkey(
		'Escape',
		() => {
			if (ctxOpen) return;
			if (selection.paths.size > 0) {
				selection.clear();
			} else if (fileClipboard.hasItems) {
				fileClipboard.clear();
			}
		},
		() => ({ enabled: !ctxOpen })
	);

	createHotkey(
		'Mod+A',
		() => {
			if (ctxOpen) return;
			selection.selectAll(navigation.visibleEntries.map((en: any) => en.path));
		},
		() => ({ enabled: !ctxOpen })
	);

	createHotkey(
		'Delete',
		async () => {
			if (ctxOpen) return;
			if (selection.paths.size === 0) return;
			if (navigation.currentView === 'trash') return;

			const paths = Array.from(selection.paths);

			try {
				confirmDeleteLoading = true;
				const totalSize = await getTotalSize(paths);
				const LARGE_THRESHOLD = 1024 * 1024 * 1024; // 1 GB

				if (totalSize >= LARGE_THRESHOLD) {
					confirmDeletePaths = paths;
					confirmDeleteSize = totalSize;
					confirmDeleteOpen = true;
				} else {
					deleteMutation.mutate(paths);
				}
			} catch {
				deleteMutation.mutate(paths);
			} finally {
				confirmDeleteLoading = false;
			}
		},
		() => ({ enabled: !ctxOpen })
	);
</script>

<OpenWithPopup
	path={openWithPath}
	open={openWithOpen}
	onclose={() => {
		openWithOpen = false;
	}}
/>

<InputPopup
	title="Rename"
	label="Enter a new name:"
	value={renamePopup.name}
	open={renamePopup.open}
	onclose={() => {
		renamePopup = { ...renamePopup, open: false };
	}}
	onsubmit={(newName) => {
		if (newName && newName !== renamePopup.name) {
			renameMutation.mutate({ oldPath: renamePopup.path, newName });
		}
		renamePopup = { ...renamePopup, open: false };
	}}
/>

<InputPopup
	title="Create Folder"
	label="Enter folder name:"
	value="New Folder"
	open={createFolderPopup.open}
	onclose={() => {
		createFolderPopup = { open: false };
	}}
	onsubmit={(name) => {
		createDirMutation.mutate({ parent: navigation.currentPath, name });
		createFolderPopup = { open: false };
	}}
/>

<InputPopup
	title="Create File"
	label="Enter file name:"
	value="new-file.txt"
	open={createFilePopup.open}
	onclose={() => {
		createFilePopup = { open: false };
	}}
	onsubmit={(name) => {
		createFileMutation.mutate({ parent: navigation.currentPath, name });
		createFilePopup = { open: false };
	}}
/>

<ContextMenu x={ctxX} y={ctxY} open={ctxOpen} items={ctxItems} onclose={closeContextMenu} />

{#if confirmDeleteOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="w-full max-w-md rounded-lg border border-border bg-background p-6 shadow-lg">
			<h2 class="text-lg font-semibold text-foreground">Permanent Delete</h2>
			<p class="mt-2 text-sm text-muted-foreground">
				You are about to permanently delete {confirmDeletePaths.length} item(s)
				({formatFileSize(confirmDeleteSize)}). This action cannot be undone.
			</p>
			<div class="mt-4 flex justify-end gap-2">
				<button
					class="inline-flex h-8 items-center justify-center rounded-md border border-border bg-transparent px-3 text-xs font-medium text-foreground transition-colors hover:bg-accent"
					onclick={() => {
						confirmDeleteOpen = false;
						confirmDeletePaths = [];
					}}
				>
					Cancel
				</button>
				<button
					class="inline-flex h-8 items-center justify-center rounded-md bg-destructive px-3 text-xs font-medium text-destructive-foreground transition-colors hover:bg-destructive/90"
					onclick={() => {
						permanentDeleteMutation.mutate(confirmDeletePaths);
						confirmDeleteOpen = false;
						confirmDeletePaths = [];
					}}
				>
					Delete Permanently
				</button>
			</div>
		</div>
	</div>
{/if}

<div class="flex h-full w-full select-none flex-col">
	<FileGridToolbar bind:viewMode />

	{#if navigation.isLoading}
		<div class="flex flex-1 items-center justify-center">
			<div class="h-6 w-6 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
		</div>
	{:else if navigation.error}
		<div class="flex flex-1 flex-col items-center justify-center gap-2">
			<p class="text-sm text-destructive">{navigation.error}</p>
			<button
				class="inline-flex h-8 items-center justify-center rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground hover:bg-primary/90"
				onclick={() => navigation.refresh()}
			>
				Retry
			</button>
		</div>
	{:else if navigation.currentView === 'trash'}
		<TrashView {viewMode} onTrashContextMenu={openTrashContextMenu} />
	{:else if navigation.visibleEntries.length === 0}
		<div
			class="flex flex-1 items-center justify-center"
			role="none"
			oncontextmenu={(e) => openFileContextMenu(e, null)}
		>
			<p class="text-sm text-muted-foreground">Empty directory</p>
		</div>
	{:else if viewMode === 'grid'}
		<FileGridView
			entries={navigation.visibleEntries}
			onContextMenu={openFileContextMenu}
			onDblClick={handleDblClick}
			onDragStart={handleDragStart}
			onDragEnd={handleDragEnd}
			{getThumbnail}
			{isMedia}
			{getMediaIcon}
		/>
	{:else}
		<FileListView
			entries={navigation.visibleEntries}
			onContextMenu={openFileContextMenu}
			onDblClick={handleDblClick}
			onDragStart={handleDragStart}
			onDragEnd={handleDragEnd}
			{getThumbnail}
			{isMedia}
			{getMediaIcon}
		/>
	{/if}
</div>
