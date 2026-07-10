<script lang="ts">
	import { navigation } from '$lib/stores/navigation.svelte';
	import { selection } from '$lib/stores/selection.svelte';
	import { handleItemClick, clearSelectionIfActive } from '$lib/composables/selection-utils';
	import { FolderOpen, RotateCw } from 'lucide-svelte';

	let {
		viewMode,
		onTrashContextMenu
	}: {
		viewMode: 'grid' | 'list';
		onTrashContextMenu: (e: MouseEvent, entryId: string | null) => void;
	} = $props();

	function formatTime(ts: number): string {
		if (!ts) return '';
		const d = new Date(ts * 1000);
		return (
			d.toLocaleDateString() +
			' ' +
			d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
		);
	}
</script>

{#if navigation.trashEntries.length === 0}
	<div class="flex flex-1 items-center justify-center" role="none">
		<p class="text-sm text-muted-foreground">Trash is empty</p>
	</div>
{:else if viewMode === 'grid'}
	<div
		class="flex min-h-0 flex-1 flex-wrap content-start gap-3 overflow-y-auto p-4"
		role="none"
		onclick={clearSelectionIfActive}
		oncontextmenu={(e) => onTrashContextMenu(e, null)}
	>
		{#each navigation.trashEntries as entry (entry.id)}
			<button
				class="flex w-[160px] cursor-pointer flex-col items-center gap-2 overflow-hidden rounded-lg border border-border bg-card p-3 text-center shadow-xs transition-all hover:bg-accent hover:text-accent-foreground active:scale-95 {selection.paths.has(
					entry.id
				)
					? 'bg-accent text-accent-foreground border-ring'
					: ''}"
				onclick={(e) => {
					e.stopPropagation();
					handleItemClick(entry.id, e);
				}}
				oncontextmenu={(e) => {
					e.stopPropagation();
					onTrashContextMenu(e, entry.id);
				}}
			>
				<FolderOpen class="h-10 w-10 shrink-0 text-muted-foreground" />
				<span class="line-clamp-2 w-full break-all text-xs leading-tight text-muted-foreground">
					{entry.name}
				</span>
				<span
					class="w-full truncate text-[0.6rem] text-muted-foreground/60"
					title={entry.original_path}
				>
					{entry.original_path}
				</span>
				{#if entry.time_deleted}
					<span class="text-[0.55rem] text-muted-foreground/50"
						>{formatTime(entry.time_deleted)}</span
					>
				{/if}
			</button>
		{/each}
	</div>
{:else}
	<div
		class="flex min-h-0 flex-1 flex-col overflow-y-auto"
		role="none"
		onclick={clearSelectionIfActive}
		oncontextmenu={(e) => onTrashContextMenu(e, null)}
	>
		<div
			class="flex items-center border-b border-border px-4 py-1.5 text-[0.7rem] font-medium uppercase tracking-wider text-muted-foreground select-none"
		>
			<span class="flex-1">Name</span>
			<span class="w-40 shrink-0 text-right">Deleted</span>
			<span class="w-6 shrink-0"></span>
		</div>
		{#each navigation.trashEntries as entry (entry.id)}
			<div
				role="button"
				tabindex="-1"
				class="flex w-full cursor-pointer items-center gap-3 overflow-hidden border-b border-border bg-transparent px-4 py-2 text-left text-sm transition-all hover:bg-accent {selection.paths.has(
					entry.id
				)
					? 'bg-accent'
					: ''}"
				onclick={(e) => {
					e.stopPropagation();
					handleItemClick(entry.id, e);
				}}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						handleItemClick(entry.id, e);
					}
				}}
				oncontextmenu={(e) => {
					e.stopPropagation();
					onTrashContextMenu(e, entry.id);
				}}
			>
				<FolderOpen class="h-5 w-5 shrink-0 text-muted-foreground" />
				<div class="min-w-0 flex-1">
					<div class="truncate">{entry.name}</div>
					<div class="truncate text-[0.65rem] text-muted-foreground/60">
						{entry.original_path}
					</div>
				</div>
				<span class="w-40 shrink-0 text-right text-xs text-muted-foreground">
					{entry.time_deleted ? formatTime(entry.time_deleted) : ''}
				</span>
				<div class="flex w-6 shrink-0 items-center gap-1">
					<button
						class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer border-0 bg-transparent"
						onclick={(e) => {
							e.stopPropagation();
							navigation.restoreTrashItem(entry.id);
						}}
						title="Restore"
					>
						<RotateCw class="h-3 w-3" />
					</button>
				</div>
			</div>
		{/each}
	</div>
{/if}
