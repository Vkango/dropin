use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const SETTINGS_VERSION: u32 = 7;
const DEFAULT_VOLUME: f32 = 75.0;
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
const MIN_LYRICS_FONT_SIZE: u32 = 20;
const MAX_LYRICS_FONT_SIZE: u32 = 56;
const DEFAULT_SHOW_SECONDARY_LYRICS: bool = true;
const DEFAULT_ALBUM_SHAPE: &str = "circle";
const DEFAULT_ALBUM_ROTATION: bool = true;
const DEFAULT_GPU_MODE: &str = "auto";

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

fn default_lyrics_font_size() -> u32 {
    32
}

fn default_show_secondary_lyrics() -> bool {
    DEFAULT_SHOW_SECONDARY_LYRICS
}

fn default_album_shape() -> String {
    DEFAULT_ALBUM_SHAPE.to_string()
}

fn default_album_rotation() -> bool {
    DEFAULT_ALBUM_ROTATION
}

fn default_gpu_mode() -> String {
    DEFAULT_GPU_MODE.to_string()
}

fn default_volume() -> f32 {
    DEFAULT_VOLUME
}

fn default_effects() -> Value {
    Value::Object(serde_json::Map::new())
}

fn default_playback() -> Value {
    serde_json::json!({
        "speed": 0.0,
        "frequencyRatio": 1.0,
        "pan": 0.0,
        "reverse": false,
    })
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
    #[serde(default = "default_lyrics_font_size")]
    pub lyrics_font_size: u32,
    #[serde(default = "default_show_secondary_lyrics")]
    pub show_secondary_lyrics: bool,
    #[serde(default = "default_album_shape")]
    pub album_shape: String,
    #[serde(default = "default_album_rotation")]
    pub album_rotation: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_effects")]
    pub effects: Value,
    #[serde(default = "default_playback")]
    pub playback: Value,
    #[serde(default = "default_gpu_mode")]
    pub gpu_mode: String,
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
            lyrics_font_size: default_lyrics_font_size(),
            show_secondary_lyrics: default_show_secondary_lyrics(),
            album_shape: default_album_shape(),
            album_rotation: default_album_rotation(),
            volume: DEFAULT_VOLUME,
            effects: default_effects(),
            playback: default_playback(),
            gpu_mode: default_gpu_mode(),
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

fn settings_path(paths: &crate::core::paths::AppPaths) -> Result<PathBuf, String> {
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
    let lyrics_font_size = settings
        .lyrics_font_size
        .clamp(MIN_LYRICS_FONT_SIZE, MAX_LYRICS_FONT_SIZE);
    let album_shape = match settings.album_shape.as_str() {
        "circle" | "rounded-rect" => settings.album_shape,
        _ => default_album_shape(),
    };
    let volume = if settings.volume.is_finite() {
        settings.volume.clamp(0.0, 100.0)
    } else {
        DEFAULT_VOLUME
    };
    let effects = match settings.effects {
        Value::Object(value) => Value::Object(value),
        _ => default_effects(),
    };
    let playback = match settings.playback {
        Value::Object(value) => Value::Object(value),
        _ => default_playback(),
    };
    let gpu_mode = match settings.gpu_mode.as_str() {
        "auto" | "high-performance" | "compatibility" => settings.gpu_mode,
        _ => default_gpu_mode(),
    };

    Ok(AppSettings {
        version: SETTINGS_VERSION,
        animation_frame_rate: frame_rate,
        theme_mode,
        auto_album_theme: settings.auto_album_theme,
        manual_theme_color,
        language,
        sidebar_width,
        lyrics_font_size,
        show_secondary_lyrics: settings.show_secondary_lyrics,
        album_shape,
        album_rotation: settings.album_rotation,
        volume,
        effects,
        playback,
        gpu_mode,
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
pub fn app_settings_read(paths: tauri::State<'_, crate::core::paths::AppPaths>) -> Result<AppSettings, String> {
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
    paths: tauri::State<'_, crate::core::paths::AppPaths>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn settings_defaults_include_audio_state() {
        let settings = AppSettings::default();
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.volume, DEFAULT_VOLUME);
        assert!(settings.effects.as_object().is_some());
        assert_eq!(settings.playback["speed"], serde_json::json!(0.0));
        assert_eq!(settings.playback["frequencyRatio"], serde_json::json!(1.0));
        assert_eq!(settings.playback["pan"], serde_json::json!(0.0));
    }

    #[test]
    fn settings_normalize_audio_state() {
        let mut settings = AppSettings::default();
        settings.volume = 180.0;
        settings.effects = json!([]);
        let normalized = normalize(settings).expect("normalized settings");
        assert_eq!(normalized.volume, 100.0);
        assert!(normalized.effects.as_object().is_some());
    }
}
