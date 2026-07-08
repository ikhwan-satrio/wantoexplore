import { createMutation, useQueryClient } from '@tanstack/svelte-query';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { navigation } from '$lib/stores/navigation.svelte';
import { fileClipboard } from '$lib/stores/clipboard.svelte';
import { selection } from '$lib/stores/selection.svelte';

function useRefresh() {
  const queryClient = useQueryClient();
  return () => {
    navigation.refresh();
    queryClient.invalidateQueries({ queryKey: ['directory'] });
  };
}

export function useDeleteMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async (paths: string[]) => {
      for (const p of paths) {
        await invoke('app_delete_file', { path: p });
      }
    },
    onMutate: () => {
      return { toastId: toast.loading('Moving to trash...', { duration: Infinity }) };
    },
    onSuccess: (_data, paths, context) => {
      toast.success(`Moved ${paths.length} item(s) to trash`, {
        id: context?.toastId,
        duration: 3000,
      });
      selection.clear();
      refresh();
    },
    onError: (error, _paths, context) => {
      toast.error(`Delete failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function usePasteMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async ({ operation, sources, dest }: { operation: 'copy' | 'cut'; sources: string[]; dest: string }) => {
      if (operation === 'copy') {
        await invoke('app_copy_files', { sources, dest });
      } else {
        await invoke('app_move_files', { sources, dest });
      }
    },
    onMutate: () => {
      return { toastId: toast.loading('Pasting...', { duration: Infinity }) };
    },
    onSuccess: (_data, variables, context) => {
      const label = variables.operation === 'copy' ? 'Copied' : 'Moved';
      toast.success(`${label} ${variables.sources.length} item(s)`, {
        id: context?.toastId,
        duration: 3000,
      });
      fileClipboard.clear();
      selection.clear();
      refresh();
    },
    onError: (error, _variables, context) => {
      toast.error(`Paste failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function useRenameMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async ({ oldPath, newName }: { oldPath: string; newName: string }) => {
      return invoke('app_rename_file', { oldPath, newName });
    },
    onMutate: () => {
      return { toastId: toast.loading('Renaming...', { duration: Infinity }) };
    },
    onSuccess: (_data, _variables, context) => {
      toast.success('Renamed successfully', {
        id: context?.toastId,
        duration: 3000,
      });
      refresh();
    },
    onError: (error, _variables, context) => {
      toast.error(`Rename failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function useCreateDirMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async ({ parent, name }: { parent: string; name: string }) => {
      return invoke('app_create_dir', { parent, name });
    },
    onMutate: () => {
      return { toastId: toast.loading('Creating folder...', { duration: Infinity }) };
    },
    onSuccess: (_data, _variables, context) => {
      toast.success('Folder created', {
        id: context?.toastId,
        duration: 3000,
      });
      refresh();
    },
    onError: (error, _variables, context) => {
      toast.error(`Create folder failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function useCreateFileMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async ({ parent, name }: { parent: string; name: string }) => {
      return invoke('app_create_file', { parent, name });
    },
    onMutate: () => {
      return { toastId: toast.loading('Creating file...', { duration: Infinity }) };
    },
    onSuccess: (_data, _variables, context) => {
      toast.success('File created', {
        id: context?.toastId,
        duration: 3000,
      });
      refresh();
    },
    onError: (error, _variables, context) => {
      toast.error(`Create file failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function useRestoreTrashMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async (ids: string[]) => {
      for (const id of ids) {
        await navigation.restoreTrashItem(id);
      }
    },
    onMutate: () => {
      return { toastId: toast.loading('Restoring...', { duration: Infinity }) };
    },
    onSuccess: (_data, ids, context) => {
      toast.success(`Restored ${ids.length} item(s)`, {
        id: context?.toastId,
        duration: 3000,
      });
      selection.clear();
      refresh();
    },
    onError: (error, _ids, context) => {
      toast.error(`Restore failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function usePurgeTrashMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async (ids: string[]) => {
      for (const id of ids) {
        await navigation.purgeTrashItem(id);
      }
    },
    onMutate: () => {
      return { toastId: toast.loading('Deleting permanently...', { duration: Infinity }) };
    },
    onSuccess: (_data, ids, context) => {
      toast.success(`Deleted ${ids.length} item(s) permanently`, {
        id: context?.toastId,
        duration: 3000,
      });
      selection.clear();
      refresh();
    },
    onError: (error, _ids, context) => {
      toast.error(`Delete failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function useEmptyTrashMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async () => {
      await navigation.emptyTrash();
    },
    onMutate: () => {
      return { toastId: toast.loading('Emptying trash...', { duration: Infinity }) };
    },
    onSuccess: (_data, _variables, context) => {
      toast.success('Trash emptied', {
        id: context?.toastId,
        duration: 3000,
      });
      refresh();
    },
    onError: (error, _variables, context) => {
      toast.error(`Empty trash failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export function usePermanentDeleteMutation() {
  const refresh = useRefresh();
  return createMutation(() => ({
    mutationFn: async (paths: string[]) => {
      await invoke('app_permanently_delete', { paths });
    },
    onMutate: () => {
      return { toastId: toast.loading('Deleting permanently...', { duration: Infinity }) };
    },
    onSuccess: (_data, paths, context) => {
      toast.success(`Deleted ${paths.length} item(s) permanently`, {
        id: context?.toastId,
        duration: 3000,
      });
      selection.clear();
      refresh();
    },
    onError: (error, _paths, context) => {
      toast.error(`Delete failed: ${error.message}`, {
        id: context?.toastId,
        duration: 5000,
      });
    },
  }));
}

export async function getTotalSize(paths: string[]): Promise<number> {
  return invoke<number>('app_get_total_size', { paths });
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
