use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_VERSION: u32 = 1;
const DEFAULT_FRAME_RATE: u32 = 60;
const MIN_FRAME_RATE: u32 = 15;
const MAX_FRAME_RATE: u32 = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub animation_frame_rate: Option<u32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            animation_frame_rate: Some(DEFAULT_FRAME_RATE),
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

    Ok(AppSettings {
        version: SETTINGS_VERSION,
        animation_frame_rate: frame_rate,
    })
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
