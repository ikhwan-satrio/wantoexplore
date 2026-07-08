<script lang="ts">
	import { selection } from '$lib/stores/selection.svelte';
	import { handleItemClick, clearSelectionIfActive } from '$lib/composables/selection-utils';
	import { getFileIcon, getFileColor } from '$lib/file-icons';
	import type { DirEntryInfo } from '$lib/types';
	import type { ComponentType } from 'svelte';

	let {
		entries,
		onContextMenu,
		onDblClick,
		onDragStart,
		onDragEnd,
		getThumbnail,
		isMedia,
		getMediaIcon
	}: {
		entries: DirEntryInfo[];
		onContextMenu: (e: MouseEvent, entry: DirEntryInfo | null) => void;
		onDblClick: (entry: DirEntryInfo) => void;
		onDragStart: (e: DragEvent, entry: DirEntryInfo) => void;
		onDragEnd: () => void;
		getThumbnail: (entry: DirEntryInfo) => string | undefined;
		isMedia: (entry: DirEntryInfo) => boolean;
		getMediaIcon: (ext: string) => ComponentType;
	} = $props();
</script>

<div
	class="flex min-h-0 flex-1 flex-col overflow-y-auto"
	role="none"
	onclick={clearSelectionIfActive}
	oncontextmenu={(e) => onContextMenu(e, null)}
>
	<div
		class="flex items-center border-b border-border px-4 py-1.5 text-[0.7rem] font-medium uppercase tracking-wider text-muted-foreground select-none"
	>
		<span class="flex-1">Name</span>
		<span class="w-20 shrink-0 text-right">Type</span>
	</div>
	{#each entries as entry (entry.path)}
		{@const Icon = getFileIcon(entry)}
		{@const thumb = getThumbnail(entry)}
		<button
			class="flex w-full cursor-pointer items-center gap-3 border-b border-border bg-transparent px-4 py-2.5 text-left text-sm transition-all hover:bg-accent {selection.paths.has(
				entry.path
			)
				? 'bg-accent'
				: ''}"
			onclick={(e) => {
				e.stopPropagation();
				handleItemClick(entry.path, e);
			}}
			ondblclick={() => onDblClick(entry)}
			oncontextmenu={(e) => {
				e.stopPropagation();
				onContextMenu(e, entry);
			}}
			draggable={true}
			ondragstart={(e) => onDragStart(e, entry)}
			ondragend={onDragEnd}
		>
			{#if thumb}
				<img
					src={thumb}
					alt={entry.name}
					class="h-8 w-8 shrink-0 rounded object-cover"
					loading="lazy"
				/>
			{:else if isMedia(entry)}
				{@const MediaIcon = getMediaIcon(entry.extension?.toLowerCase() || '')}
				<MediaIcon class="h-5 w-5 shrink-0 text-muted-foreground" />
			{:else}
				<Icon class="h-5 w-5 shrink-0 {getFileColor(entry)}" />
			{/if}
			<span class="flex-1 truncate">{entry.name}</span>
			<span class="w-20 shrink-0 text-right text-xs text-muted-foreground">
				{entry.is_dir ? 'Folder' : entry.extension || 'File'}
			</span>
		</button>
	{/each}
</div>
