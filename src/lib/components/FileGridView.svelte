<script lang="ts">
	import { selection } from '$lib/stores/selection.svelte';
	import { handleItemClick, clearSelectionIfActive } from '$lib/composables/selection-utils';
	import { getFileIcon, getFileColor } from '$lib/file-icons';
	import type { DirEntryInfo } from '$lib/types';
	import type { ComponentType } from 'svelte';
	import { Image, FileVideo, FileAudio } from 'lucide-svelte';

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
	class="flex min-h-0 flex-1 flex-wrap content-start gap-3 overflow-y-auto p-4"
	role="none"
	onclick={clearSelectionIfActive}
	oncontextmenu={(e) => onContextMenu(e, null)}
>
	{#each entries as entry (entry.path)}
		{@const Icon = getFileIcon(entry)}
		{@const thumb = getThumbnail(entry)}
		<button
			class="flex w-[120px] cursor-pointer flex-col items-center gap-2 rounded-lg border border-border bg-card p-3 text-center shadow-xs transition-all hover:bg-accent hover:text-accent-foreground active:scale-95 {selection.paths.has(
				entry.path
			)
				? 'bg-accent text-accent-foreground border-ring'
				: ''}"
			style="content-visibility: auto; contain-intrinsic-size: 120px 120px;"
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
					class="h-[76px] w-[76px] shrink-0 rounded object-cover"
					loading="lazy"
				/>
			{:else if isMedia(entry)}
				{@const MediaIcon = getMediaIcon(entry.extension?.toLowerCase() || '')}
				<MediaIcon class="h-10 w-10 shrink-0 text-muted-foreground" />
			{:else}
				<Icon class="h-10 w-10 shrink-0 {getFileColor(entry)}" />
			{/if}
			<span class="line-clamp-2 w-full break-all text-xs leading-tight text-muted-foreground">
				{entry.name}
			</span>
		</button>
	{/each}
</div>
