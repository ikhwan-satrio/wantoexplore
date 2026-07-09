<script lang="ts">
	import { Loader2 } from 'lucide-svelte';
	import { navigation } from '$lib/stores/navigation.svelte';
	import { selection } from '$lib/stores/selection.svelte';

	let { viewMode = $bindable<'grid' | 'list'>('grid') }: { viewMode: 'grid' | 'list' } = $props();
</script>

<div class="flex shrink-0 items-center justify-between border-b border-border px-3 py-1">
	<span class="text-xs text-muted-foreground">
		{#if navigation.currentView === 'trash'}
			{navigation.trashEntries.length} items
		{:else if selection.paths.size > 0}
			<span class="text-primary font-medium">{selection.paths.size} selected</span>
			<span class="ml-1 opacity-60">/ {navigation.visibleEntries.length} items</span>
		{:else}
			{navigation.visibleEntries.length} / {navigation.entries.length} items
		{/if}
	</span>
	<div class="flex items-center gap-1">
		<button
			class="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground {viewMode ===
			'grid'
				? 'bg-accent text-foreground'
				: ''}"
			onclick={() => (viewMode = 'grid')}
			title="Grid view"
		>
			<svg class="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
				<rect x="1" y="1" width="6" height="6" rx="1" />
				<rect x="9" y="1" width="6" height="6" rx="1" />
				<rect x="1" y="9" width="6" height="6" rx="1" />
				<rect x="9" y="9" width="6" height="6" rx="1" />
			</svg>
		</button>
		<button
			class="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground {viewMode ===
			'list'
				? 'bg-accent text-foreground'
				: ''}"
			onclick={() => (viewMode = 'list')}
			title="List view"
		>
			<svg class="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
				<rect x="1" y="2" width="14" height="2.5" rx="0.5" />
				<rect x="1" y="6.75" width="14" height="2.5" rx="0.5" />
				<rect x="1" y="11.5" width="14" height="2.5" rx="0.5" />
			</svg>
		</button>
	</div>
</div>
