use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub library_dir: PathBuf,
    pub config_dir: PathBuf,
    pub database: PathBuf,
    pub settings_file: PathBuf,
    pub cache_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub urls_dir: PathBuf,
    pub log_file: PathBuf,
    pub plugins_dir: PathBuf,
    pub plugin_data_dir: PathBuf,
    pub plugins_file: PathBuf,
}

fn fallback_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("application")
}

pub fn resolve(data_dir: Option<&str>) -> AppPaths {
    let root = match data_dir.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(fallback_root)
            .join("application"),
    };
    let library_dir = root.join("library");
    let config_dir = root.join("config");
    let cache_dir = root.join("cache");
    let plugins_dir = root.join("plugins");
    let plugin_data_dir = root.join("plugin-data");
    AppPaths {
        database: library_dir.join("dropin.sqlite3"),
        settings_file: config_dir.join("settings.json"),
        covers_dir: cache_dir.join("covers"),
        urls_dir: cache_dir.join("urls"),
        log_file: config_dir.join("scan.log"),
        plugins_file: config_dir.join("plugins.json"),
        plugins_dir,
        plugin_data_dir,
        library_dir,
        config_dir,
        cache_dir,
        root,
    }
}

impl AppPaths {
    pub fn prepare(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.library_dir)?;
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.covers_dir)?;
        fs::create_dir_all(&self.urls_dir)?;
        fs::create_dir_all(&self.plugins_dir)?;
        fs::create_dir_all(&self.plugin_data_dir)
    }

    pub fn resolve_track_path(&self, stored: &str) -> PathBuf {
        PathBuf::from(stored)
    }

    pub fn store_track_path(&self, path: &Path) -> String {
        // 外部引用的音频实体文件一律存绝对路径，避免依赖数据目录位置
        path.to_string_lossy().into_owned()
    }
}

pub fn app_paths_from_app(app: &AppHandle) -> AppPaths {
    let bootstrap = crate::settings::bootstrap_data_dir(app);
    resolve(bootstrap.as_deref())
}
