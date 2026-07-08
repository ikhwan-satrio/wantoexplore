import { createMutation, useQueryClient } from '@tanstack/svelte-query';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import { navigation } from '$lib/stores/navigation.svelte';

interface ArchiveProgress {
  current: number;
  total: number;
  file_name: string;
}

function getArchiveExt(format: string): string {
  return format === 'tar.gz' ? '.tar.gz' : `.${format}`;
}

function getParentDir(path: string): string {
  const parts = path.split('/');
  parts.pop();
  return parts.join('/') || '/';
}

function getBaseName(path: string): string {
  const parts = path.split('/');
  return parts.pop() || 'archive';
}

function formatPercent(current: number, total: number): string {
  if (total === 0) return '0%';
  return `${Math.round((current / total) * 100)}%`;
}

export function useCompressMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: async ({ sources, format }: { sources: string[]; format: string }) => {
      if (sources.length === 0) throw new Error('No files selected');

      const parentDir = getParentDir(sources[0]);
      const baseName = sources.length === 1 ? getBaseName(sources[0]) : 'archive';
      const ext = getArchiveExt(format);
      const dest = `${parentDir}/${baseName}${ext}`;

      let unlisten: UnlistenFn | null = null;
      unlisten = await listen<ArchiveProgress>('archive-progress', (event) => {
        const { current, total, file_name } = event.payload;
        toast.loading(`Compressing ${file_name} (${formatPercent(current, total)})`, {
          id: 'archive-progress',
        });
      });

      const result = await invoke<ArchiveProgress[]>('app_compress_files', {
        sources,
        dest,
        format,
      });

      unlisten?.();
      return { result, dest, baseName: `${baseName}${ext}` };
    },
    onMutate: () => {
      const toastId = toast.loading('Compressing...', { id: 'archive-progress', duration: Infinity });
      return { toastId };
    },
    onSuccess: (data, _variables, context) => {
      const finalProgress = data.result[data.result.length - 1];
      toast.success(`Compressed to ${data.baseName}`, {
        id: context?.toastId,
        description: `${finalProgress.total} file(s) processed`,
        duration: 4000,
      });
      navigation.refresh();
      queryClient.invalidateQueries({ queryKey: ['directory'] });
    },
    onError: (error, _variables, context) => {
      toast.error(`Compress failed: ${error.message}`, {
        id: context?.toastId,
        duration: 6000,
      });
    },
  }));
}

export function useExtractMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: async ({ archivePath }: { archivePath: string }) => {
      const parentDir = getParentDir(archivePath);
      const archiveName = getBaseName(archivePath);
      const folderName = archiveName
        .replace(/\.(zip|tar|tgz|tbz2|txz|tzst)$/i, '')
        .replace(/\.tar\.(gz|bz2|xz|zst)$/i, '');
      const dest = `${parentDir}/${folderName}`;

      await invoke('app_create_dir', { parent: parentDir, name: folderName }).catch(() => {});

      let unlisten: UnlistenFn | null = null;
      unlisten = await listen<ArchiveProgress>('archive-progress', (event) => {
        const { current, total, file_name } = event.payload;
        toast.loading(`Extracting ${file_name} (${formatPercent(current, total)})`, {
          id: 'archive-progress',
        });
      });

      const result = await invoke<ArchiveProgress[]>('app_extract_archive', {
        archive: archivePath,
        dest,
      });

      unlisten?.();
      return { result, folderName };
    },
    onMutate: () => {
      const toastId = toast.loading('Extracting...', { id: 'archive-progress', duration: Infinity });
      return { toastId };
    },
    onSuccess: (data, _variables, context) => {
      const finalProgress = data.result[data.result.length - 1];
      toast.success(`Extracted to ${data.folderName}/`, {
        id: context?.toastId,
        description: `${finalProgress.total} file(s) extracted`,
        duration: 4000,
      });
      navigation.refresh();
      queryClient.invalidateQueries({ queryKey: ['directory'] });
    },
    onError: (error, _variables, context) => {
      toast.error(`Extract failed: ${error.message}`, {
        id: context?.toastId,
        duration: 6000,
      });
    },
  }));
}
