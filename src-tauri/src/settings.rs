use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_VERSION: u32 = 3;
const DEFAULT_FRAME_RATE: u32 = 60;
const MIN_FRAME_RATE: u32 = 15;
const MAX_FRAME_RATE: u32 = 120;
const DEFAULT_THEME_MODE: &str = "system";
const DEFAULT_AUTO_ALBUM_THEME: bool = true;
const DEFAULT_MANUAL_THEME_COLOR: &str = "#88d0ec";
const DEFAULT_LANGUAGE: &str = "system";

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
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("settings.json"))
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

    Ok(AppSettings {
        version: SETTINGS_VERSION,
        animation_frame_rate: frame_rate,
        theme_mode,
        auto_album_theme: settings.auto_album_theme,
        manual_theme_color,
        language,
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
pub fn app_settings_read(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
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
pub fn app_settings_write(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let normalized = normalize(settings)?;
    let path = settings_path(&app)?;
    let temporary_path = path.with_extension("json.tmp");
    let contents = serde_json::to_vec_pretty(&normalized).map_err(|error| error.to_string())?;

    fs::write(&temporary_path, contents).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| error.to_string())?;

    Ok(normalized)
}
