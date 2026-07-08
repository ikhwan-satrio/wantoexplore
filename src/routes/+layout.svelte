<script lang="ts">
  import "./layout.css"
  import { onMount } from 'svelte';
  import AppLayout from '$lib/components/layout/app-layout.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { navigation } from '$lib/stores/navigation.svelte';
  import { Toaster } from '$lib/components/ui/sonner';
  import { ModeWatcher } from "mode-watcher";
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 0,
        refetchOnWindowFocus: false,
      },
    },
  });

  let { children } = $props();

  onMount(() => {
    navigation.init();
  });
</script>

<QueryClientProvider client={queryClient}>
  <AppLayout>
    {#snippet sidebar()}
      <Sidebar />
    {/snippet}

    {@render children()}
  </AppLayout>
  <ModeWatcher/>
  <Toaster position="bottom-right" richColors closeButton />
</QueryClientProvider>
