<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { navigation } from '$lib/stores/navigation.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { createHotkey } from '@tanstack/svelte-hotkeys';
  import type { DirEntryInfo } from '$lib/types';
  import {
    Minus, Square, Copy, X,
    ChevronLeft, ChevronRight, ChevronUp, RotateCw,
    CheckSquare,
  } from 'lucide-svelte';

  const appWindow = getCurrentWindow();
  let isMaximized = $state(false);

  appWindow.isMaximized().then((v) => (isMaximized = v));
  appWindow.onResized(async () => {
    isMaximized = await appWindow.isMaximized();
  });

  let editing = $state(false);
  let pathInput = $state('');
  let suggestions = $state<DirEntryInfo[]>([]);
  let selIndex = $state(0);
  let inputEl: HTMLInputElement | undefined = $state(undefined);
  let containerEl: HTMLElement | undefined = $state(undefined);
  let debounceTimer: number | undefined;

  function btnCls() {
    return `flex items-center justify-center w-7 h-7 rounded-md text-muted-foreground transition-all duration-100 border-0 bg-transparent cursor-pointer enabled:hover:bg-accent enabled:hover:text-foreground active:scale-90 disabled:opacity-30 disabled:cursor-default`;
  }

  function handleBreadcrumbClick(index: number) {
    const parts = navigation.breadcrumbs;
    const path = '/' + parts.slice(0, index + 1).join('/');
    navigation.navigateTo(path);
  }

  function startEditing() {
    editing = true;
    pathInput = navigation.currentPath;
    selIndex = 0;
    requestAnimationFrame(() => {
      inputEl?.focus();
      inputEl?.setSelectionRange(pathInput.length, pathInput.length);
    });
    fetchSuggestions(navigation.currentPath);
  }

  function submitPath(path: string) {
    editing = false;
    suggestions = [];
    if (path.trim()) {
      navigation.navigateTo(path.trim());
    }
  }

  function cancelEditing() {
    editing = false;
    suggestions = [];
  }

  function onInputChange() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fetchSuggestions(pathInput), 120);
  }

  async function fetchSuggestions(input: string) {
    if (!input || input === '/') {
      try {
        suggestions = await invoke<DirEntryInfo[]>('app_list_dir', { path: '/' });
      } catch { suggestions = []; }
      return;
    }
    const parts = input.split('/');
    const prefix = parts.pop() || '';
    const parent = parts.join('/') || '/';
    try {
      const entries = await invoke<DirEntryInfo[]>('app_list_dir', { path: parent });
      suggestions = entries.filter(
        (e) => e.is_dir && e.name.toLowerCase().startsWith(prefix.toLowerCase())
      );
    } catch { suggestions = []; }
  }

  function selectSuggestion(entry: DirEntryInfo) {
    pathInput = entry.path.endsWith('/') ? entry.path : entry.path + '/';
    suggestions = [];
    selIndex = 0;
    requestAnimationFrame(() => {
      inputEl?.focus();
      inputEl?.setSelectionRange(pathInput.length, pathInput.length);
    });
    fetchSuggestions(pathInput);
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      submitPath(pathInput);
    } else if (e.key === 'Escape') {
      cancelEditing();
    } else if (e.key === 'Tab' && suggestions.length > 0) {
      e.preventDefault();
      selectSuggestion(suggestions[selIndex]);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selIndex = Math.min(selIndex + 1, suggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selIndex = Math.max(selIndex - 1, 0);
    }
  }

  function onInputBlur() {
    setTimeout(() => {
      if (!containerEl?.contains(document.activeElement)) {
        cancelEditing();
      }
    }, 150);
  }

  createHotkey('Mod+L', () => {
    startEditing();
  });
</script>

<div class="flex p-4 shrink-0 select-none items-center border-b border-border bg-background" bind:this={containerEl}>
  <!-- Nav buttons -->
  <div class="flex items-center gap-0.5 pl-2" data-tauri-drag-region>
    <button class={btnCls()} disabled={!navigation.canGoBack} onclick={() => navigation.goBack()} title="Back">
      <ChevronLeft class="h-4 w-4" />
    </button>
    <button class={btnCls()} disabled={!navigation.canGoForward} onclick={() => navigation.goForward()} title="Forward">
      <ChevronRight class="h-4 w-4" />
    </button>
    <button class={btnCls()} disabled={navigation.currentPath === '/'} onclick={() => navigation.goUp()} title="Up">
      <ChevronUp class="h-4 w-4" />
    </button>
    <button class={btnCls()} onclick={() => navigation.refresh()} title="Refresh">
      <RotateCw class="h-3.5 w-3.5" />
    </button>
  </div>

  <!-- Path breadcrumb / input -->
  <div class="relative mx-4 flex flex-1 items-center overflow-hidden rounded-md bg-muted/50 px-2 py-1" data-tauri-drag-region>
    {#if editing}
      <input
        bind:this={inputEl}
        type="text"
        bind:value={pathInput}
        oninput={onInputChange}
        onkeydown={onInputKeydown}
        onblur={onInputBlur}
        class="z-10 w-full bg-transparent py-1 text-xs text-foreground outline-none"
        placeholder="/"
      />
      {#if suggestions.length > 0}
        <div class="absolute left-0 right-0 top-full z-50 mt-1 max-h-48 overflow-y-auto rounded-lg border border-border bg-popover p-1 shadow-lg">
          {#each suggestions as entry, i}
            <button
              class="flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-xs text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground {i === selIndex ? 'bg-accent text-accent-foreground' : ''} cursor-pointer border-0 bg-transparent text-left"
              onmousedown={(e) => { e.preventDefault(); selectSuggestion(entry); }}
            >
              <span class="shrink-0 text-muted-foreground">/</span>
              <span>{entry.name}</span>
            </button>
          {/each}
        </div>
      {/if}
    {:else}
      <button
        class="flex min-w-0 flex-1 items-center gap-0 text-left"
        onclick={startEditing}
        title="Click to edit path (Ctrl+L)"
      >
        {#each navigation.breadcrumbs as part, i}
          {#if i > 0}
            <span class="mx-0.5 shrink-0 text-muted-foreground/50">/</span>
          {/if}
          <span
            class="shrink-0 truncate rounded px-1.5 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            role="button"
            tabindex="-1"
            onclick={(e) => { e.stopPropagation(); handleBreadcrumbClick(i); }}
            onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); handleBreadcrumbClick(i); } }}
          >
            {part}
          </span>
        {/each}
      </button>
    {/if}
  </div>

  <!-- Selection indicator -->
  {#if selection.hasSelection}
    <button
      class="flex items-center gap-1.5 mr-2 px-2 py-1 rounded-md bg-primary/10 text-primary text-xs font-medium transition-all hover:bg-primary/20 cursor-pointer border-0"
      onclick={() => selection.clear()}
      title="Click to deselect all"
    >
      <CheckSquare class="h-3.5 w-3.5" />
      <span>{selection.count}</span>
    </button>
  {/if}

  <!-- Window controls -->
  <div class="ml-auto flex h-full">
    <button class="flex items-center justify-center w-9 h-full text-muted-foreground transition-all duration-100 border-0 bg-transparent cursor-pointer hover:bg-accent hover:text-foreground" onclick={() => appWindow.minimize()} aria-label="Minimize">
      <Minus class="h-3.5 w-3.5" />
    </button>
    <button class="flex items-center justify-center w-9 h-full text-muted-foreground transition-all duration-100 border-0 bg-transparent cursor-pointer hover:bg-accent hover:text-foreground" onclick={() => appWindow.toggleMaximize()} aria-label={isMaximized ? 'Restore' : 'Maximize'}>
      {#if isMaximized}
        <Copy class="h-3 w-3" />
      {:else}
        <Square class="h-3 w-3" />
      {/if}
    </button>
    <button class="flex items-center justify-center w-9 h-full text-muted-foreground transition-all duration-100 border-0 bg-transparent cursor-pointer hover:bg-destructive hover:text-white" onclick={() => appWindow.close()} aria-label="Close">
      <X class="h-3.5 w-3.5" />
    </button>
  </div>
</div>
