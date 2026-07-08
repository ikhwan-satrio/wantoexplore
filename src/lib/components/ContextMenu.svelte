<script lang="ts">
	import type { ComponentType } from 'svelte';

	export interface ContextMenuItem {
		label: string;
		icon?: ComponentType;
		action: () => void;
		disabled?: boolean;
		separator?: boolean;
	}

	let {
		x,
		y,
		open,
		items,
		onclose
	}: {
		x: number;
		y: number;
		open: boolean;
		items: ContextMenuItem[];
		onclose: () => void;
	} = $props();

	let menuRef: HTMLDivElement | undefined = $state(undefined);
	let menuX = $state(0);
	let menuY = $state(0);

	$effect(() => {
		if (!open || !menuRef) return;

		const rect = menuRef.getBoundingClientRect();
		const vw = window.innerWidth;
		const vh = window.innerHeight;

		menuX = x + rect.width > vw ? vw - rect.width - 8 : x;
		menuY = y + rect.height > vh ? vh - rect.height - 8 : y;
	});

	$effect(() => {
		if (!open) return;

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') onclose();
		}

		function handleClickOutside(e: MouseEvent) {
			if (menuRef && !menuRef.contains(e.target as Node)) {
				onclose();
			}
		}

		const raf = requestAnimationFrame(() => {
			document.addEventListener('keydown', handleKeydown);
			document.addEventListener('mousedown', handleClickOutside);
		});

		return () => {
			cancelAnimationFrame(raf);
			document.removeEventListener('keydown', handleKeydown);
			document.removeEventListener('mousedown', handleClickOutside);
		};
	});
</script>

{#if open}
	<div
		class="fixed inset-0 z-40"
		role="presentation"
		onclick={onclose}
		oncontextmenu={(e) => e.preventDefault()}
	></div>

	<div
		bind:this={menuRef}
		style="left: {menuX}px; top: {menuY}px;"
		class="fixed z-50 min-w-45 rounded-lg border border-border bg-popover p-1 shadow-lg"
		role="menu"
		tabindex="-1"
		oncontextmenu={(e) => e.preventDefault()}
	>
		{#each items as item}
			{#if item.separator}
				<div class="my-1 border-t border-border"></div>
			{:else}
				<button
					class="flex w-full items-center gap-2.5 rounded-md px-3 py-1.5 text-xs text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground disabled:opacity-40 disabled:pointer-events-none cursor-pointer border-0 bg-transparent text-left"
					disabled={item.disabled}
					onclick={(e) => {
						e.stopPropagation();
						onclose();
						item.action();
					}}
					role="menuitem"
				>
					{#if item.icon}
						{@const Icon = item.icon}
						<Icon class="h-4 w-4 shrink-0" />
					{/if}
					<span>{item.label}</span>
				</button>
			{/if}
		{/each}
	</div>
{/if}
