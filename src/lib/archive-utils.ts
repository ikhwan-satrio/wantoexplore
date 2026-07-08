import { toast } from 'svelte-sonner';
import { invoke } from '@tauri-apps/api/core';
import { navigation } from '$lib/stores/navigation.svelte';

interface ArchiveProgress {
  current: number;
  total: number;
  file_name: string;
}

function isArchive(name: string): boolean {
  const lower = name.toLowerCase();
  return lower.endsWith('.zip') ||
    lower.endsWith('.tar') ||
    lower.endsWith('.tar.gz') ||
    lower.endsWith('.tgz') ||
    lower.endsWith('.tar.bz2') ||
    lower.endsWith('.tbz2') ||
    lower.endsWith('.tar.xz') ||
    lower.endsWith('.txz') ||
    lower.endsWith('.tar.zst') ||
    lower.endsWith('.tzst');
}

function getArchiveExt(format: string): string {
  return format === 'tar.gz' ? '.tar.gz' : `.${format}`;
}

export async function compressFiles(sources: string[], format: string = 'zip') {
  if (sources.length === 0) return;

  const firstSrc = sources[0];
  const parts = firstSrc.split('/');
  parts.pop();
  const parentDir = parts.join('/') || '/';

  const baseName = sources.length === 1
    ? (parts.pop() || 'archive')
    : (parts.pop() || 'archive');

  const ext = getArchiveExt(format);
  const dest = `${parentDir}/${baseName}${ext}`;

  const toastId = toast.loading(`Compressing ${sources.length} item(s)...`, {
    duration: Infinity,
  });

  try {
    const result = await invoke<ArchiveProgress[]>('app_compress_files', {
      sources,
      dest,
      format,
    });

    const finalProgress = result[result.length - 1];
    toast.success(`Compressed to ${baseName}${ext}`, {
      id: toastId,
      description: `${finalProgress.total} file(s) processed`,
      duration: 4000,
    });

    await navigation.refresh();
  } catch (e) {
    toast.error(`Compress failed: ${String(e)}`, {
      id: toastId,
      duration: 6000,
    });
  }
}

export async function extractArchive(archivePath: string) {
  const parts = archivePath.split('/');
  parts.pop();
  const parentDir = parts.join('/') || '/';

  const archiveName = archivePath.split('/').pop() || 'archive';
  const folderName = archiveName.replace(/\.(zip|tar|tgz|tbz2|txz|tzst)$/i, '')
    .replace(/\.tar\.(gz|bz2|xz|zst)$/i, '');
  const dest = `${parentDir}/${folderName}`;

  const toastId = toast.loading(`Extracting ${archiveName}...`, {
    duration: Infinity,
  });

  try {
    await invoke('app_create_dir', { parent: parentDir, name: folderName }).catch(() => {});

    const result = await invoke<ArchiveProgress[]>('app_extract_archive', {
      archive: archivePath,
      dest,
    });

    const finalProgress = result[result.length - 1];
    toast.success(`Extracted to ${folderName}/`, {
      id: toastId,
      description: `${finalProgress.total} file(s) extracted`,
      duration: 4000,
    });

    await navigation.refresh();
  } catch (e) {
    toast.error(`Extract failed: ${String(e)}`, {
      id: toastId,
      duration: 6000,
    });
  }
}
