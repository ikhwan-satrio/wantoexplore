mod capabilities;
mod extension_manager;
mod navigation;
mod open_with;
mod thumbnails;

use capabilities::CapabilityChecker;
use extension_manager::{ExtensionManager, ExtensionType};
use navigation::{get_pinned_dirs, get_xdg_dirs, PinnedDir, XdgDirs};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tokio::sync::Semaphore;

#[derive(Serialize)]
pub struct DirEntryInfo {
    name: String,
    path: String,
    is_dir: bool,
    extension: Option<String>,
}

#[derive(Serialize)]
pub struct ExtensionInfo {
    id: String,
    name: String,
    version: String,
    #[serde(rename = "type")]
    extension_type: String,
    enabled: bool,
}

pub struct AppState {
    extension_manager: Mutex<ExtensionManager>,
    current_dir: Mutex<Option<PathBuf>>,
    thumbnails: Arc<thumbnails::ThumbnailCache>,
    thumbnail_semaphore: Arc<Semaphore>,
}

// ─── Filesystem commands ─────────────────────────────────────────────

#[tauri::command]
fn app_list_dir(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Vec<DirEntryInfo>, String> {
    let target: PathBuf = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let dir = state.current_dir.lock().map_err(|e| e.to_string())?;
            match dir.clone() {
                Some(d) => d,
                None => std::env::current_dir().map_err(|e| e.to_string())?,
            }
        }
    };

    let mut entries: Vec<DirEntryInfo> = std::fs::read_dir(&target)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();

            let path = entry.path();
            let is_dir = metadata.is_dir();
            let extension = if is_dir {
                None
            } else {
                path.extension()
                    .map(|e| e.to_string_lossy().to_string())
            };

            Some(DirEntryInfo {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir,
                extension,
            })
        })
        .collect();

    // directories first, then alphabetical
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

#[tauri::command]
fn app_set_current_dir(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let mut current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;
    *current_dir = Some(PathBuf::from(path));
    Ok(())
}

#[tauri::command]
fn app_get_current_dir(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;
    Ok(current_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
fn app_rename_file(old_path: String, new_name: String) -> Result<String, String> {
    let src = PathBuf::from(&old_path);
    let parent = src.parent().ok_or("Cannot determine parent directory")?;
    let dst = parent.join(&new_name);

    if dst.exists() {
        return Err(format!("A file named '{}' already exists", new_name));
    }

    std::fs::rename(&src, &dst).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().to_string())
}

#[tauri::command]
fn app_delete_file(path: String) -> Result<(), String> {
    let target = PathBuf::from(&path);
    trash::delete(&target).map_err(|e| format!("Failed to move to trash: {}", e))
}

#[derive(Serialize)]
struct TrashEntry {
    id: String,
    name: String,
    original_path: String,
    is_dir: bool,
    time_deleted: i64,
}

#[tauri::command]
fn app_list_trash() -> Result<Vec<TrashEntry>, String> {
    let items = trash::os_limited::list().map_err(|e| format!("Failed to list trash: {}", e))?;
    let mut entries = Vec::new();
    for item in &items {
        let original_path = item.original_path();
        // let meta = trash::os_limited::metadata(item).ok();
        let is_dir = original_path.extension().is_none();
        entries.push(TrashEntry {
            id: item.id.to_string_lossy().to_string(),
            name: item.name.to_string_lossy().to_string(),
            original_path: original_path.to_string_lossy().to_string(),
            time_deleted: item.time_deleted,
            is_dir,
        });
    }
    Ok(entries)
}

#[tauri::command]
fn app_restore_trash_item(id: String) -> Result<(), String> {
    let items = trash::os_limited::list().map_err(|e| format!("Failed to list trash: {}", e))?;
    let item = items.into_iter().find(|i| i.id.to_string_lossy() == id)
        .ok_or("Item not found in trash")?;
    trash::os_limited::restore_all(vec![item]).map_err(|e| format!("Failed to restore: {}", e))
}

#[tauri::command]
fn app_purge_trash_item(id: String) -> Result<(), String> {
    let items = trash::os_limited::list().map_err(|e| format!("Failed to list trash: {}", e))?;
    let item = items.into_iter().find(|i| i.id.to_string_lossy() == id)
        .ok_or("Item not found in trash")?;
    trash::os_limited::purge_all([item]).map_err(|e| format!("Failed to purge: {}", e))
}

#[tauri::command]
fn app_empty_trash() -> Result<(), String> {
    let items = trash::os_limited::list().map_err(|e| format!("Failed to list trash: {}", e))?;
    trash::os_limited::purge_all(items).map_err(|e| format!("Failed to empty trash: {}", e))
}

#[tauri::command]
fn app_get_open_with_apps(path: String) -> Result<Vec<open_with::OpenWithApp>, String> {
    Ok(open_with::find_apps_for_file(&path))
}

#[tauri::command]
fn app_open_with_file(path: String, exec_template: String) -> Result<(), String> {
    open_with::open_file_with(&path, &exec_template)
}

#[tauri::command]
fn app_create_dir(parent: String, name: String) -> Result<String, String> {
    let target = PathBuf::from(&parent).join(&name);
    std::fs::create_dir(&target).map_err(|e| format!("Failed to create directory: {}", e))?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn app_create_file(parent: String, name: String) -> Result<String, String> {
    let target = PathBuf::from(&parent).join(&name);
    std::fs::write(&target, "").map_err(|e| format!("Failed to create file: {}", e))?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn app_copy_files(sources: Vec<String>, dest: String) -> Result<(), String> {
    let dest = PathBuf::from(&dest);
    for src in &sources {
        let src = PathBuf::from(src);
        let file_name = src.file_name().ok_or("Invalid source path")?;
        let dst = dest.join(file_name);
        if dst.exists() {
            return Err(format!("'{}' already exists", file_name.to_string_lossy()));
        }
        if src.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst).map_err(|e| format!("Failed to copy: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn app_move_files(sources: Vec<String>, dest: String) -> Result<(), String> {
    let dest = PathBuf::from(&dest);
    for src in &sources {
        let src = PathBuf::from(src);
        let file_name = src.file_name().ok_or("Invalid source path")?;
        let dst = dest.join(file_name);
        if dst.exists() {
            return Err(format!("'{}' already exists", file_name.to_string_lossy()));
        }
        std::fs::rename(&src, &dst).map_err(|e| format!("Failed to move: {}", e))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir: {}", e))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(&entry.path(), &dst_path).map_err(|e| format!("Failed to copy: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn app_open_default(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open").arg(&path).spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd").args(["/c", "start", "", &path]).spawn();

    result.map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn app_get_thumbnail(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let cache = state.thumbnails.clone();
    let sem = state.thumbnail_semaphore.clone();
    let permit = sem.acquire_owned().await.map_err(|e| e.to_string())?;

    let cache_path = tokio::task::spawn_blocking(move || {
        let result = cache.get_or_generate(&path);
        drop(permit);
        result
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(cache_path.to_string_lossy().to_string())
}

// ─── XDG / Navigation commands ───────────────────────────────────────

#[tauri::command]
fn app_get_xdg() -> Result<XdgDirs, String> {
    Ok(get_xdg_dirs())
}

#[tauri::command]
fn app_get_pinned(tauri_app: tauri::AppHandle) -> Result<Vec<PinnedDir>, String> {
    Ok(get_pinned_dirs(&tauri_app))
}

#[tauri::command]
fn app_add_pinned(
    tauri_app: tauri::AppHandle,
    name: String,
    path: String,
) -> Result<Vec<PinnedDir>, String> {
    navigation::add_pinned_dir(&tauri_app, name, path)
}

#[tauri::command]
fn app_remove_pinned(
    tauri_app: tauri::AppHandle,
    path: String,
) -> Result<Vec<PinnedDir>, String> {
    navigation::remove_pinned_dir(&tauri_app, &path)
}

#[tauri::command]
fn app_reorder_pinned(
    tauri_app: tauri::AppHandle,
    dirs: Vec<PinnedDir>,
) -> Result<Vec<PinnedDir>, String> {
    navigation::reorder_pinned_dirs(&tauri_app, dirs)
}

// ─── Extension commands ──────────────────────────────────────────────

#[tauri::command]
fn app_get_extensions(state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager
        .get_extensions()
        .iter()
        .map(|ext| ExtensionInfo {
            id: ext.manifest.id.clone(),
            name: ext.manifest.name.clone(),
            version: ext.manifest.version.clone(),
            extension_type: match ext.manifest.extension_type {
                ExtensionType::Theme => "theme".to_string(),
                ExtensionType::Plugin => "plugin".to_string(),
            },
            enabled: ext.enabled,
        })
        .collect())
}

#[tauri::command]
fn app_get_themes(state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager
        .get_themes()
        .iter()
        .map(|ext| ExtensionInfo {
            id: ext.manifest.id.clone(),
            name: ext.manifest.name.clone(),
            version: ext.manifest.version.clone(),
            extension_type: "theme".to_string(),
            enabled: ext.enabled,
        })
        .collect())
}

#[tauri::command]
fn app_get_plugins(state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager
        .get_plugins()
        .iter()
        .map(|ext| ExtensionInfo {
            id: ext.manifest.id.clone(),
            name: ext.manifest.name.clone(),
            version: ext.manifest.version.clone(),
            extension_type: "plugin".to_string(),
            enabled: ext.enabled,
        })
        .collect())
}

#[tauri::command]
fn app_get_theme_css(state: State<'_, AppState>, theme_id: String) -> Result<String, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    manager.get_theme_css(&theme_id)
}

#[tauri::command]
fn app_get_plugin_js(state: State<'_, AppState>, plugin_id: String) -> Result<String, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    manager.get_plugin_js(&plugin_id)
}

#[tauri::command]
fn app_enable_extension(
    state: State<'_, AppState>,
    extension_id: String,
) -> Result<bool, String> {
    let mut manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.enable_extension(&extension_id))
}

#[tauri::command]
fn app_disable_extension(
    state: State<'_, AppState>,
    extension_id: String,
) -> Result<bool, String> {
    let mut manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.disable_extension(&extension_id))
}

#[tauri::command]
fn app_reload_extensions(state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    manager.scan_extensions();
    Ok(())
}

// ─── Plugin permission commands ──────────────────────────────────────

#[tauri::command]
fn app_check_plugin_permission(
    state: State<'_, AppState>,
    plugin_id: String,
    permission: String,
    path: Option<String>,
) -> Result<bool, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    let current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;

    let checker = CapabilityChecker::new(&manager);
    let checker = if let Some(dir) = current_dir.as_ref() {
        checker.with_current_dir(dir.clone())
    } else {
        checker
    };

    let target_path = path.as_ref().map(|p| PathBuf::from(p));
    let result = checker.check_permission(&plugin_id, &permission, target_path.as_deref());

    Ok(result.allowed)
}

#[tauri::command]
fn app_plugin_read_file(
    state: State<'_, AppState>,
    plugin_id: String,
    path: String,
) -> Result<String, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    let current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;

    let checker = CapabilityChecker::new(&manager);
    let checker = if let Some(dir) = current_dir.as_ref() {
        checker.with_current_dir(dir.clone())
    } else {
        checker
    };

    let target_path = PathBuf::from(&path);
    let permission_check = checker.can_read_file(&plugin_id, &target_path);

    if !permission_check.allowed {
        return Err(permission_check.reason);
    }

    std::fs::read_to_string(&target_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn app_plugin_write_file(
    state: State<'_, AppState>,
    plugin_id: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    let current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;

    let checker = CapabilityChecker::new(&manager);
    let checker = if let Some(dir) = current_dir.as_ref() {
        checker.with_current_dir(dir.clone())
    } else {
        checker
    };

    let target_path = PathBuf::from(&path);
    let permission_check = checker.can_write_file(&plugin_id, &target_path);

    if !permission_check.allowed {
        return Err(permission_check.reason);
    }

    std::fs::write(&target_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn app_plugin_list_dir(
    state: State<'_, AppState>,
    plugin_id: String,
    path: String,
) -> Result<Vec<DirEntryInfo>, String> {
    let manager = state.extension_manager.lock().map_err(|e| e.to_string())?;
    let current_dir = state.current_dir.lock().map_err(|e| e.to_string())?;

    let checker = CapabilityChecker::new(&manager);
    let checker = if let Some(dir) = current_dir.as_ref() {
        checker.with_current_dir(dir.clone())
    } else {
        checker
    };

    let target_path = PathBuf::from(&path);
    let permission_check = checker.can_list_directory(&plugin_id, &target_path);

    if !permission_check.allowed {
        return Err(permission_check.reason);
    }

    drop(manager);
    drop(current_dir);

    // inline the listing logic
    let target = PathBuf::from(&path);
    let mut entries: Vec<DirEntryInfo> = std::fs::read_dir(&target)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            let path = entry.path();
            let is_dir = metadata.is_dir();
            let extension = if is_dir {
                None
            } else {
                path.extension().map(|e| e.to_string_lossy().to_string())
            };
            Some(DirEntryInfo {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir,
                extension,
            })
        })
        .collect();
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(entries)
}

// ─── App entry ───────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let extension_manager = ExtensionManager::new(app.handle());
            let cache_dir = app.path().cache_dir().unwrap_or_else(|_| {
                std::env::temp_dir().join("teddypicker-cache")
            });
            let app_state = AppState {
                extension_manager: Mutex::new(extension_manager),
                current_dir: Mutex::new(None),
                thumbnails: Arc::new(thumbnails::ThumbnailCache::new(cache_dir)),
                thumbnail_semaphore: Arc::new(Semaphore::new(4)),
            };
            app.manage(app_state);

            Ok(())
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // filesystem
            app_list_dir,
            app_set_current_dir,
            app_get_current_dir,
            app_rename_file,
            app_delete_file,
            app_list_trash,
            app_restore_trash_item,
            app_purge_trash_item,
            app_empty_trash,
            app_get_open_with_apps,
            app_open_with_file,
            app_open_default,
            app_create_dir,
            app_create_file,
            app_copy_files,
            app_move_files,
            app_get_thumbnail,
            // navigation
            app_get_xdg,
            app_get_pinned,
            app_add_pinned,
            app_remove_pinned,
            app_reorder_pinned,
            // extensions
            app_get_extensions,
            app_get_themes,
            app_get_plugins,
            app_get_theme_css,
            app_get_plugin_js,
            app_enable_extension,
            app_disable_extension,
            app_reload_extensions,
            // plugin permissions
            app_check_plugin_permission,
            app_plugin_read_file,
            app_plugin_write_file,
            app_plugin_list_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
