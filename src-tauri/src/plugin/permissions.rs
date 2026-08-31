use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;

pub const UI_PANEL: &str = "ui.panel";
pub const PLAYER_READ: &str = "player.read";
pub const PLAYER_CONTROL: &str = "player.control";
pub const LIBRARY_READ: &str = "library.read";
pub const NOTIFICATION_SHOW: &str = "notification.show";
pub const STORAGE_PLUGIN: &str = "storage.plugin";

pub const ALLOWED_PERMISSIONS: &[&str] = &[
    UI_PANEL,
    PLAYER_READ,
    PLAYER_CONTROL,
    LIBRARY_READ,
    NOTIFICATION_SHOW,
    STORAGE_PLUGIN,
];

pub fn is_allowed(permission: &str) -> bool {
    ALLOWED_PERMISSIONS.contains(&permission)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PermissionState {
    pub declared: Vec<String>,
    pub granted: Vec<String>,
}

impl PermissionState {
    pub fn new(declared: Vec<String>, granted: Vec<String>) -> Self {
        Self { declared, granted }
    }
}

pub fn validate_permissions(permissions: &[String]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for permission in permissions {
        if !is_allowed(permission) {
            return Err(format!("unsupported plugin permission: {permission}"));
        }
        if !seen.insert(permission) {
            return Err(format!("duplicate plugin permission: {permission}"));
        }
    }
    Ok(())
}
