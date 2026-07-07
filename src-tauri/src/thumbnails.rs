use image::GenericImageView;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAX_THUMB_SIZE: u32 = 128;
const JPEG_QUALITY: u8 = 70;

#[derive(Clone)]
pub struct ThumbnailCache {
    cache_dir: PathBuf,
}

fn path_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hex::encode(hasher.finalize())
}

impl ThumbnailCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        let thumb_dir = cache_dir.join("thumbnails");
        let _ = fs::create_dir_all(&thumb_dir);
        Self {
            cache_dir: thumb_dir,
        }
    }

    fn thumb_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.jpg", hash))
    }

    fn meta_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", hash))
    }

    fn current_mtime(path: &str) -> u64 {
        Path::new(path)
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn is_cache_valid(&self, hash: &str, expected_mtime: u64) -> bool {
        let meta_path = self.meta_path(hash);
        let thumb_path = self.thumb_path(hash);
        if !meta_path.exists() || !thumb_path.exists() {
            return false;
        }
        let stored_mtime: u64 = fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        stored_mtime == expected_mtime
    }

    fn write_cache(&self, hash: &str, mtime: u64, jpeg_bytes: &[u8]) {
        let meta_path = self.meta_path(hash);
        let thumb_path = self.thumb_path(hash);
        let _ = fs::write(&thumb_path, jpeg_bytes);
        let _ = fs::write(&meta_path, mtime.to_string());
    }

    pub fn get_or_generate(&self, path: &str) -> Result<PathBuf, String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mtime = Self::current_mtime(path);
        let hash = path_hash(path);
        let thumb_path = self.thumb_path(&hash);

        // Try disk cache first
        if self.is_cache_valid(&hash, mtime) {
            return Ok(thumb_path);
        }

        // Generate
        let jpeg_bytes = match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tiff" | "tif" | "avif"
            | "heic" | "heif" => generate_image_thumbnail(path)?,
            "mp4" | "avi" | "mov" | "mkv" | "webm" | "m4v" | "3gp" => {
                generate_video_thumbnail(path)?
            }
            _ => return Err("unsupported".to_string()),
        };

        // Write to disk cache
        self.write_cache(&hash, mtime, &jpeg_bytes);
        Ok(thumb_path)
    }
}

fn generate_image_thumbnail(path: &str) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

    let (w, h) = img.dimensions();
    let thumb = if w > MAX_THUMB_SIZE || h > MAX_THUMB_SIZE {
        img.resize(MAX_THUMB_SIZE, MAX_THUMB_SIZE, image::imageops::FilterType::CatmullRom)
    } else {
        img
    };

    let mut buf = Cursor::new(Vec::new());
    let mut jpeg_encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY);
    jpeg_encoder
        .encode(
            thumb.as_bytes(),
            thumb.width(),
            thumb.height(),
            thumb.color().into(),
        )
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    Ok(buf.into_inner())
}

fn generate_video_thumbnail(path: &str) -> Result<Vec<u8>, String> {
    let result = try_ffmpegthumbnailer(path);
    if result.is_ok() {
        return result;
    }
    try_ffmpeg(path)
}

fn try_ffmpegthumbnailer(path: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("ffmpegthumbnailer")
        .args(["-i", path, "-s", &MAX_THUMB_SIZE.to_string(), "-o", "-", "-c", "jpeg", "-q", &JPEG_QUALITY.to_string()])
        .output()
        .map_err(|_| "ffmpegthumbnailer not found".to_string())?;

    if output.status.success() && !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err("ffmpegthumbnailer failed".to_string())
    }
}

fn try_ffmpeg(path: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-ss",
            "00:00:01",
            "-i",
            path,
            "-vframes",
            "1",
            "-vf",
            &format!(
                "scale='min({},iw)':min'({},ih)':force_original_aspect_ratio=decrease",
                MAX_THUMB_SIZE, MAX_THUMB_SIZE
            ),
            "-f",
            "image2pipe",
            "-q:v",
            &JPEG_QUALITY.to_string(),
            "-",
        ])
        .output()
        .map_err(|_| "ffmpeg not found".to_string())?;

    if output.status.success() && !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err("ffmpeg failed".to_string())
    }
}
