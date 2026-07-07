use crate::extension_manager::{ExtensionManager, ExtensionType};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PermissionCheck {
    pub allowed: bool,
    pub reason: String,
}

pub struct CapabilityChecker<'a> {
    extension_manager: &'a ExtensionManager,
    current_dir: Option<PathBuf>,
}

impl<'a> CapabilityChecker<'a> {
    pub fn new(extension_manager: &'a ExtensionManager) -> Self {
        Self {
            extension_manager,
            current_dir: None,
        }
    }

    pub fn with_current_dir(mut self, dir: PathBuf) -> Self {
        self.current_dir = Some(dir);
        self
    }

    pub fn check_permission(
        &self,
        plugin_id: &str,
        permission: &str,
        target_path: Option<&Path>,
    ) -> PermissionCheck {
        let ext = match self.extension_manager.get_extension_by_id(plugin_id) {
            Some(ext) => ext,
            None => {
                return PermissionCheck {
                    allowed: false,
                    reason: format!("Plugin '{}' not found", plugin_id),
                }
            }
        };

        if ext.manifest.extension_type != ExtensionType::Plugin {
            return PermissionCheck {
                allowed: false,
                reason: "Extension is not a plugin".to_string(),
            };
        }

        if !ext.enabled {
            return PermissionCheck {
                allowed: false,
                reason: "Plugin is disabled".to_string(),
            };
        }

        // Check if plugin has declared this permission
        if !ext.manifest.permissions.contains(&permission.to_string()) {
            return PermissionCheck {
                allowed: false,
                reason: format!(
                    "Plugin '{}' does not have '{}' permission declared",
                    plugin_id, permission
                ),
            };
        }

        // Check scope-based permissions
        match permission {
            "fs:read" | "fs:write" | "fs:*" => {
                self.check_fs_permission(target_path)
            }
            _ => PermissionCheck {
                allowed: true,
                reason: "Permission granted".to_string(),
            },
        }
    }

    fn check_fs_permission(&self, target_path: Option<&Path>) -> PermissionCheck {
        let target = match target_path {
            Some(p) => p,
            None => {
                return PermissionCheck {
                    allowed: true,
                    reason: "No specific path to check".to_string(),
                }
            }
        };

        // If we have a current directory context, ensure target is within it
        if let Some(current_dir) = &self.current_dir {
            match target.strip_prefix(current_dir) {
                Ok(_) => PermissionCheck {
                    allowed: true,
                    reason: "Path is within current directory".to_string(),
                },
                Err(_) => PermissionCheck {
                    allowed: false,
                    reason: format!(
                        "Access denied: target path '{}' is outside current directory",
                        target.display()
                    ),
                },
            }
        } else {
            // No current directory context, allow but log
            PermissionCheck {
                allowed: true,
                reason: "No directory context, allowing access".to_string(),
            }
        }
    }

    pub fn can_read_file(&self, plugin_id: &str, path: &Path) -> PermissionCheck {
        self.check_permission(plugin_id, "fs:read", Some(path))
    }

    pub fn can_write_file(&self, plugin_id: &str, path: &Path) -> PermissionCheck {
        self.check_permission(plugin_id, "fs:write", Some(path))
    }

    pub fn can_list_directory(&self, plugin_id: &str, path: &Path) -> PermissionCheck {
        self.check_permission(plugin_id, "fs:read", Some(path))
    }
}
