use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use zstd::stream::Decoder as ZstdDecoder;
use zstd::stream::Encoder as ZstdEncoder;

#[derive(Clone, serde::Serialize)]
pub struct ArchiveProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
}

#[derive(Clone, serde::Serialize)]
pub struct ArchiveFormat {
    pub ext: String,
    pub label: String,
}

pub fn get_supported_compress_formats() -> Vec<ArchiveFormat> {
    vec![
        ArchiveFormat {
            ext: "zip".into(),
            label: "ZIP Archive".into(),
        },
        ArchiveFormat {
            ext: "tar.gz".into(),
            label: "Tar Gzip".into(),
        },
        ArchiveFormat {
            ext: "tar.bz2".into(),
            label: "Tar Bzip2".into(),
        },
        ArchiveFormat {
            ext: "tar.xz".into(),
            label: "Tar XZ".into(),
        },
        ArchiveFormat {
            ext: "tar.zst".into(),
            label: "Tar Zstandard".into(),
        },
    ]
}

fn count_files(path: &Path) -> Result<usize, String> {
    if path.is_file() {
        return Ok(1);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        count += count_files(&entry.path())?;
    }
    Ok(count)
}

// ─── ZIP ─────────────────────────────────────────────────────────────

fn add_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    path: &Path,
    base: &Path,
    options: SimpleFileOptions,
) -> Result<usize, String> {
    let mut count = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            count += add_to_zip(zip, &entry.path(), base, options)?;
        }
    } else {
        let name = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        let mut f = File::open(path).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        zip.write_all(&buf).map_err(|e| e.to_string())?;
        count = 1;
    }
    Ok(count)
}

fn compress_to_zip(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));
    let mut zip = ZipWriter::new(file);
    let mut total = 0;
    let mut processed = 0;
    let mut progress = Vec::new();

    for src in sources {
        let path = PathBuf::from(src);
        total += count_files(&path)?;
    }

    for src in sources {
        let path = PathBuf::from(src);
        let count = add_to_zip(&mut zip, &path, path.parent().unwrap_or(&path), options)?;
        processed += count;
        let p = ArchiveProgress {
            current: processed,
            total,
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };
        on_progress(p.clone());
        progress.push(p);
    }

    zip.finish()
        .map_err(|e| format!("Failed to finish archive: {}", e))?;
    Ok(progress)
}

fn extract_zip(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut zip =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;
    let total = zip.len();
    let mut progress = Vec::new();

    for i in 0..total {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let out_path = dest.join(entry.mangled_name());

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
        }

        let p = ArchiveProgress {
            current: i + 1,
            total,
            file_name: entry.mangled_name().to_string_lossy().to_string(),
        };
        on_progress(p.clone());
        progress.push(p);
    }

    Ok(progress)
}

// ─── TAR helpers ─────────────────────────────────────────────────────

fn compress_tar_with_writer(
    sources: &[String],
    writer: Box<dyn Write>,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let mut tar = tar::Builder::new(writer);
    let mut total = 0;
    let mut processed = 0;
    let mut progress = Vec::new();

    for src in sources {
        let path = PathBuf::from(src);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", src));
        }
        total += count_files(&path)?;
    }

    for src in sources {
        let path = PathBuf::from(src);
        if path.is_dir() {
            tar.append_dir_all(path.file_name().unwrap_or_default(), &path)
                .map_err(|e| e.to_string())?;
        } else {
            let mut f = File::open(&path).map_err(|e| e.to_string())?;
            tar.append_file(path.file_name().unwrap_or_default(), &mut f)
                .map_err(|e| e.to_string())?;
        }
        processed += 1;
        let p = ArchiveProgress {
            current: processed,
            total,
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };
        on_progress(p.clone());
        progress.push(p);
    }

    tar.finish()
        .map_err(|e| format!("Failed to finish archive: {}", e))?;
    Ok(progress)
}

// ─── TAR.GZ ──────────────────────────────────────────────────────────

fn compress_to_tar_gz(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    let enc = GzEncoder::new(file, Compression::default());
    compress_tar_with_writer(sources, Box::new(enc), on_progress)
}

fn extract_tar_gz(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    let dec = GzDecoder::new(file);
    extract_tar_entries(dec, dest, on_progress)
}

// ─── TAR.BZ2 ─────────────────────────────────────────────────────────

fn compress_to_tar_bz2(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    let enc = BzEncoder::new(file, BzCompression::default());
    compress_tar_with_writer(sources, Box::new(enc), on_progress)
}

fn extract_tar_bz2(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    let dec = BzDecoder::new(file);
    extract_tar_entries(dec, dest, on_progress)
}

// ─── TAR.XZ ──────────────────────────────────────────────────────────

fn compress_to_tar_xz(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    let enc = XzEncoder::new(file, 6);
    compress_tar_with_writer(sources, Box::new(enc), on_progress)
}

fn extract_tar_xz(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    let dec = XzDecoder::new(file);
    extract_tar_entries(dec, dest, on_progress)
}

// ─── TAR.ZST ─────────────────────────────────────────────────────────

fn compress_to_tar_zst(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    let enc = ZstdEncoder::new(file, 3).map_err(|e| e.to_string())?;
    compress_tar_with_writer(sources, Box::new(enc), on_progress)
}

fn extract_tar_zst(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    let dec = ZstdDecoder::new(file).map_err(|e| e.to_string())?;
    extract_tar_entries(dec, dest, on_progress)
}

// ─── TAR (uncompressed) ──────────────────────────────────────────────

fn compress_to_tar(
    sources: &[String],
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let file = File::create(dest).map_err(|e| format!("Failed to create archive: {}", e))?;
    compress_tar_with_writer(sources, Box::new(file), on_progress)
}

fn extract_tar(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let file = File::open(archive).map_err(|e| format!("Failed to open archive: {}", e))?;
    extract_tar_entries(file, dest, on_progress)
}

// ─── TAR extract helper ──────────────────────────────────────────────

fn extract_tar_entries<R: Read>(
    reader: R,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let mut tar = TarArchive::new(reader);
    let entries: Vec<_> = tar.entries().map_err(|e| e.to_string())?.collect();
    let total = entries.len();
    let mut progress = Vec::new();
    let mut idx = 0;

    for entry in entries {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let out_path = dest.join(entry.path().map_err(|e| e.to_string())?);

        entry.unpack(&out_path).map_err(|e| e.to_string())?;

        idx += 1;
        let p = ArchiveProgress {
            current: idx,
            total,
            file_name: entry
                .path()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string(),
        };
        on_progress(p.clone());
        progress.push(p);
    }

    Ok(progress)
}

// ─── Detection ───────────────────────────────────────────────────────

pub fn get_archive_type(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let lower = name.to_lowercase();

    if lower.ends_with(".zip") {
        Some("zip")
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Some("tar.gz")
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Some("tar.bz2")
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Some("tar.xz")
    } else if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") {
        Some("tar.zst")
    } else if lower.ends_with(".tar") {
        Some("tar")
    } else {
        None
    }
}

pub fn compress_files(
    sources: &[String],
    dest: &Path,
    format: &str,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    // Validate all source paths exist
    for src in sources {
        let path = PathBuf::from(src);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", src));
        }
    }

    match format {
        "zip" => compress_to_zip(sources, dest, on_progress),
        "tar" => compress_to_tar(sources, dest, on_progress),
        "tar.gz" | "tgz" => compress_to_tar_gz(sources, dest, on_progress),
        "tar.bz2" | "tbz2" => compress_to_tar_bz2(sources, dest, on_progress),
        "tar.xz" | "txz" => compress_to_tar_xz(sources, dest, on_progress),
        "tar.zst" | "tzst" => compress_to_tar_zst(sources, dest, on_progress),
        _ => Err(format!("Unsupported archive format: {}", format)),
    }
}

pub fn extract_archive(
    archive: &Path,
    dest: &Path,
    on_progress: &mut dyn FnMut(ArchiveProgress),
) -> Result<Vec<ArchiveProgress>, String> {
    let format = get_archive_type(archive)
        .ok_or_else(|| format!("Cannot determine archive type for: {}", archive.display()))?;

    match format {
        "zip" => extract_zip(archive, dest, on_progress),
        "tar" => extract_tar(archive, dest, on_progress),
        "tar.gz" | "tgz" => extract_tar_gz(archive, dest, on_progress),
        "tar.bz2" | "tbz2" => extract_tar_bz2(archive, dest, on_progress),
        "tar.xz" | "txz" => extract_tar_xz(archive, dest, on_progress),
        "tar.zst" | "tzst" => extract_tar_zst(archive, dest, on_progress),
        _ => Err(format!("Unsupported archive format: {}", format)),
    }
}
