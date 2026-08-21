use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn custom_i18n_dir() -> Option<PathBuf> {
    let base = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let dir = base.join("i18n");
    if dir.is_dir() {
        return Some(dir);
    }
    if cfg!(debug_assertions) {
        let dev_dir = base
            .ancestors()
            .find(|ancestor| ancestor.join("i18n").is_dir())?
            .join("i18n");
        return Some(dev_dir);
    }
    None
}

fn sanitize_locale(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 32 {
        return None;
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some(trimmed.to_string())
}

fn locale_file(dir: &std::path::Path, locale: &str) -> Option<PathBuf> {
    let safe = sanitize_locale(locale)?;
    Some(dir.join(format!("{safe}.json")))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLocale {
    pub code: String,
    pub name: String,
}

#[tauri::command]
pub fn i18n_list_custom() -> Vec<CustomLocale> {
    let Some(dir) = custom_i18n_dir() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut locales = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let Some(code) = sanitize_locale(stem) else {
                continue;
            };
            let name = fs::read_to_string(&path)
                .ok()
                .and_then(|contents| serde_json::from_str::<Value>(&contents).ok())
                .and_then(|value| {
                    value
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .map(str::to_string)
                })
                .unwrap_or(code.clone());
            locales.push(CustomLocale { code, name });
        }
    }
    locales.sort_by(|a, b| a.code.cmp(&b.code));
    locales
}

#[tauri::command]
pub fn i18n_load_custom(locale: String) -> Option<Value> {
    let Some(dir) = custom_i18n_dir() else {
        return None;
    };
    let Some(path) = locale_file(&dir, &locale) else {
        return None;
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    let Ok(value) = serde_json::from_str::<Value>(&contents) else {
        return None;
    };
    if value.is_object() {
        Some(value)
    } else {
        None
    }
}
