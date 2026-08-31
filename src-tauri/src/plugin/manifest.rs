use crate::plugin::permissions::{validate_permissions, API_VERSION};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

const ID_MAX_LENGTH: usize = 128;
const MIN_BACKGROUND_TICK_MS: u64 = 1_000;
const MAX_BACKGROUND_TICK_MS: u64 = 60_000;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginBackground {
    #[serde(default)]
    pub tick_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub backend: String,
    pub ui: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub background: PluginBackground,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported plugin manifest schema: {}",
                self.schema_version
            ));
        }
        if self.api_version != API_VERSION {
            return Err(format!(
                "unsupported plugin API version: {}",
                self.api_version
            ));
        }
        validate_id(&self.id)?;
        if self.name.trim().is_empty() || self.name.len() > 256 {
            return Err("plugin name must be non-empty and at most 256 bytes".into());
        }
        if self.description.len() > 4096 || self.author.len() > 256 {
            return Err("plugin metadata is too large".into());
        }
        Version::parse(&self.version)
            .map_err(|error| format!("invalid plugin version: {error}"))?;
        validate_relative_file(&self.backend, "backend")?;
        validate_relative_file(&self.ui, "ui")?;
        if let Some(icon) = &self.icon {
            validate_relative_file(icon, "icon")?;
        }
        if let Some(interval) = self.background.tick_interval_ms {
            if !(MIN_BACKGROUND_TICK_MS..=MAX_BACKGROUND_TICK_MS).contains(&interval) {
                return Err(format!(
                    "plugin background.tickIntervalMs must be between {MIN_BACKGROUND_TICK_MS} and {MAX_BACKGROUND_TICK_MS}"
                ));
            }
        }
        validate_permissions(&self.permissions)
    }

    pub fn backend_path(&self, root: &Path) -> PathBuf {
        root.join(Path::new(&self.backend))
    }

    pub fn ui_path(&self, root: &Path) -> PathBuf {
        root.join(Path::new(&self.ui))
    }
}

pub fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > ID_MAX_LENGTH {
        return Err("plugin id must be between 1 and 128 bytes".into());
    }
    if !id.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
    }) {
        return Err("plugin id contains unsupported characters".into());
    }
    Ok(())
}

pub fn validate_relative_file(value: &str, field: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() || value.contains('\\') {
        return Err(format!("plugin {field} must be a relative POSIX path"));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!("plugin {field} contains an unsafe path"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> PluginManifest {
        PluginManifest {
            schema_version: 1,
            id: "com.example.demo".into(),
            name: "Demo".into(),
            version: "1.0.0".into(),
            api_version: 1,
            author: String::new(),
            description: String::new(),
            categories: vec!["utility".into()],
            backend: "backend.wasm".into(),
            ui: "ui/index.html".into(),
            icon: None,
            permissions: vec!["ui.panel".into()],
            background: PluginBackground::default(),
        }
    }

    #[test]
    fn validates_manifest() {
        assert!(manifest().validate().is_ok());
    }

    #[test]
    fn rejects_unsafe_paths_and_ids() {
        let mut value = manifest();
        value.id = "../demo".into();
        assert!(value.validate().is_err());
        value.id = "com.example.demo".into();
        value.ui = "../index.html".into();
        assert!(value.validate().is_err());
    }
}
