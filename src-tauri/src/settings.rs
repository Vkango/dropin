use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const SETTINGS_VERSION: u32 = 4;
const DEFAULT_FRAME_RATE: u32 = 60;
const MIN_FRAME_RATE: u32 = 15;
const MAX_FRAME_RATE: u32 = 120;
const DEFAULT_THEME_MODE: &str = "system";
const DEFAULT_AUTO_ALBUM_THEME: bool = true;
const DEFAULT_MANUAL_THEME_COLOR: &str = "#88d0ec";
const DEFAULT_LANGUAGE: &str = "system";
const DEFAULT_SIDEBAR_WIDTH: u32 = 280;
const MIN_SIDEBAR_WIDTH: u32 = 200;
const MAX_SIDEBAR_WIDTH: u32 = 480;

fn default_theme_mode() -> String {
    DEFAULT_THEME_MODE.to_string()
}

fn default_auto_album_theme() -> bool {
    DEFAULT_AUTO_ALBUM_THEME
}

fn default_manual_theme_color() -> String {
    DEFAULT_MANUAL_THEME_COLOR.to_string()
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_string()
}

fn default_sidebar_width() -> u32 {
    DEFAULT_SIDEBAR_WIDTH
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapSettings {
    pub data_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub animation_frame_rate: Option<u32>,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_auto_album_theme")]
    pub auto_album_theme: bool,
    #[serde(default = "default_manual_theme_color")]
    pub manual_theme_color: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            animation_frame_rate: Some(DEFAULT_FRAME_RATE),
            theme_mode: default_theme_mode(),
            auto_album_theme: default_auto_album_theme(),
            manual_theme_color: default_manual_theme_color(),
            language: default_language(),
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
        }
    }
}

fn bootstrap_path(app: &AppHandle) -> Option<PathBuf> {
    let directory = app.path().app_data_dir().ok()?;
    Some(directory.join("settings.json"))
}

pub fn bootstrap_data_dir(app: &AppHandle) -> Option<String> {
    let path = bootstrap_path(app)?;
    let contents = fs::read_to_string(path).ok()?;
    let settings: BootstrapSettings = serde_json::from_str(&contents).ok()?;
    settings
        .data_dir
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn settings_path(paths: &crate::paths::AppPaths) -> Result<PathBuf, String> {
    fs::create_dir_all(&paths.root).map_err(|error| error.to_string())?;
    Ok(paths.settings_file.clone())
}

fn normalize(settings: AppSettings) -> Result<AppSettings, String> {
    let frame_rate = settings
        .animation_frame_rate
        .map(|value| value.clamp(MIN_FRAME_RATE, MAX_FRAME_RATE));

    let theme_mode = match settings.theme_mode.as_str() {
        "system" | "light" | "dark" => settings.theme_mode,
        _ => default_theme_mode(),
    };
    let manual_theme_color = if is_valid_theme_color(&settings.manual_theme_color) {
        settings.manual_theme_color.to_lowercase()
    } else {
        default_manual_theme_color()
    };
    let language = normalize_language(&settings.language);
    let sidebar_width = settings
        .sidebar_width
        .clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH);

    Ok(AppSettings {
        version: SETTINGS_VERSION,
        animation_frame_rate: frame_rate,
        theme_mode,
        auto_album_theme: settings.auto_album_theme,
        manual_theme_color,
        language,
        sidebar_width,
    })
}

fn normalize_language(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 32 {
        return default_language();
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return default_language();
    }
    trimmed.to_string()
}

fn is_valid_theme_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..]
            .iter()
            .all(|character| character.is_ascii_hexdigit())
}

#[tauri::command]
pub fn app_settings_read(paths: tauri::State<'_, crate::paths::AppPaths>) -> Result<AppSettings, String> {
    let path = settings_path(&paths)?;
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AppSettings::default())
        }
        Err(_) => return Ok(AppSettings::default()),
    };

    let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) else {
        return Ok(AppSettings::default());
    };
    normalize(settings)
}

#[tauri::command]
pub fn app_settings_write(
    paths: tauri::State<'_, crate::paths::AppPaths>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let normalized = normalize(settings)?;
    let path = settings_path(&paths)?;
    let temporary_path = path.with_extension("json.tmp");
    let contents = serde_json::to_vec_pretty(&normalized).map_err(|error| error.to_string())?;

    fs::write(&temporary_path, contents).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| error.to_string())?;

    Ok(normalized)
}

fn write_bootstrap(app: &AppHandle, data_dir: Option<String>) -> Result<(), String> {
    let path = bootstrap_path(app).ok_or_else(|| "cannot resolve bootstrap settings path".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let payload = BootstrapSettings {
        data_dir: data_dir.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
    };
    let contents = serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, contents).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn data_dir_read(app: AppHandle) -> Result<Option<String>, String> {
    Ok(bootstrap_data_dir(&app))
}

#[tauri::command]
pub fn data_dir_set(app: AppHandle, data_dir: Option<String>) -> Result<Option<String>, String> {
    let value = data_dir
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(ref dir) = value {
        if !Path::new(dir).is_absolute() {
            return Err("data dir must be an absolute path".to_string());
        }
        fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }
    write_bootstrap(&app, value.clone())?;
    Ok(value)
}
