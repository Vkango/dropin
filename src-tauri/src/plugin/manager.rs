use crate::{
    core::bass_bridge::BassService,
    core::paths::AppPaths,
    media::media_library::MediaService,
    plugin::manifest::{PluginManifest, TagProviderManifest},
    plugin::permissions::{
        PermissionState, API_VERSION, LIBRARY_AUDIO_READ, LIBRARY_READ, NOTIFICATION_SHOW,
        PLAYER_CONTROL, PLAYER_READ, STORAGE_PLUGIN, TAG_PROVIDER, UI_PANEL,
    },
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek},
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, State};
use wasmtime::{
    Caller, Config, Engine, Instance, Linker, Module, Store, StoreLimits, StoreLimitsBuilder,
};
use zip::ZipArchive;

const MAX_FILES: usize = 256;
const MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const WASM_MEMORY_BYTES: usize = 32 * 1024 * 1024;
const WASM_FUEL: u64 = 2_000_000;
// Tag 分析逐首调用，插件可能整曲多点取样（多次音频读取 + JSON 解析），预算需远大于普通调用
const TAG_ANALYSIS_FUEL: u64 = 200_000_000;
const STATE_VERSION: u32 = 2;
const EVENT_PLUGIN_NOTIFICATION: &str = "plugin/notification";
const EVENT_TAG_ANALYSIS_PROGRESS: &str = "tag/analysis-progress";
const EVENT_TAG_ANALYSIS_FINISHED: &str = "tag/analysis-finished";
const EVENT_TAG_ANALYSIS_ERROR: &str = "tag/analysis-error";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PersistedPlugin {
    id: String,
    version: String,
    enabled: bool,
    #[serde(default)]
    granted_permissions: Vec<String>,
    #[serde(default)]
    faulted: bool,
    #[serde(default)]
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub author: String,
    pub description: String,
    pub categories: Vec<String>,
    pub ui: Option<String>,
    pub tag_provider: Option<TagProviderManifest>,
    pub icon: Option<String>,
    pub icon_url: Option<String>,
    pub permissions: PermissionState,
    pub installed: bool,
    pub enabled: bool,
    pub faulted: bool,
    pub last_error: Option<String>,
}

#[derive(Clone)]
struct PluginAccess {
    id: String,
    name: String,
    is_tag_provider: bool,
    declared_permissions: Vec<String>,
    granted_permissions: Vec<String>,
}

impl PluginAccess {
    fn from_plugin(plugin: &InstalledPlugin) -> Self {
        Self {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
            is_tag_provider: plugin.manifest.tag_provider.is_some(),
            declared_permissions: plugin.manifest.permissions.clone(),
            granted_permissions: plugin.state.granted_permissions.clone(),
        }
    }
}

#[derive(Clone)]
struct HostApiContext {
    paths: AppPaths,
    host_state: Arc<RwLock<Value>>,
    bass: BassService,
    media: MediaService,
    app: AppHandle,
}

#[derive(Clone)]
struct WasmHostContext {
    access: PluginAccess,
    host: HostApiContext,
}

struct WasmStoreData {
    limits: StoreLimits,
    host_context: Option<WasmHostContext>,
}

struct WasmPlugin {
    store: Store<WasmStoreData>,
    instance: Instance,
}

struct InstalledPlugin {
    manifest: PluginManifest,
    root: PathBuf,
    state: PersistedPlugin,
    wasm: Option<WasmPlugin>,
    last_background_tick_ms: u64,
}

struct Runtime {
    engine: Engine,
    plugins: HashMap<String, InstalledPlugin>,
}

#[derive(Clone)]
pub struct PluginManager {
    paths: AppPaths,
    runtime: Arc<Mutex<Runtime>>,
    host_state: Arc<RwLock<Value>>,
    analysis_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl PluginManager {
    pub fn new(paths: AppPaths) -> Self {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_multi_memory(false);
        config.wasm_threads(false);
        let engine = Engine::new(&config).expect("failed to initialize Wasmtime");
        let manager = Self {
            paths,
            runtime: Arc::new(Mutex::new(Runtime {
                engine,
                plugins: HashMap::new(),
            })),
            host_state: Arc::new(RwLock::new(json!({}))),
            analysis_flags: Arc::new(Mutex::new(HashMap::new())),
        };
        manager.load_installed();
        manager
    }

    fn load_installed(&self) {
        let Ok(entries) = fs::read_dir(&self.paths.plugins_dir) else {
            return;
        };
        let mut runtime = self.runtime.lock().expect("plugin runtime poisoned");
        for entry in entries.flatten() {
            let root = entry.path().join("current");
            let manifest_path = root.join("plugin.json");
            let Ok(contents) = fs::read_to_string(&manifest_path) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<PluginManifest>(&contents) else {
                continue;
            };
            if manifest.validate().is_err() {
                continue;
            }
            let state = load_state(&self.paths.plugins_file, &manifest).unwrap_or_else(|| {
                PersistedPlugin {
                    id: manifest.id.clone(),
                    version: manifest.version.clone(),
                    ..Default::default()
                }
            });
            let enabled = state.enabled && !state.faulted;
            let mut plugin = InstalledPlugin {
                manifest,
                root,
                state: PersistedPlugin { enabled, ..state },
                wasm: None,
                last_background_tick_ms: 0,
            };
            if enabled {
                if let Err(error) = start_wasm(&runtime.engine, &mut plugin) {
                    plugin.state.enabled = false;
                    plugin.state.faulted = true;
                    plugin.state.last_error = Some(error);
                }
            }
            runtime.plugins.insert(plugin.manifest.id.clone(), plugin);
        }
        let _ = persist_states(&self.paths.plugins_file, &runtime.plugins);
    }

    pub fn list(&self) -> Result<Vec<PluginInfo>, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let mut result = runtime
            .plugins
            .values()
            .map(plugin_info)
            .collect::<Vec<_>>();
        result.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(result)
    }

    fn provider_values(&self) -> Result<Vec<Value>, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let mut providers = runtime
            .plugins
            .values()
            .filter_map(|plugin| {
                let provider = plugin.manifest.tag_provider.as_ref()?;
                Some(json!({
                    "key": provider.key,
                    "name": provider.name,
                    "help": provider.help,
                    "valueType": provider.value_type,
                    "supportsSegments": provider.supports_segments,
                    "pluginId": plugin.manifest.id,
                    "sourceVersion": plugin.manifest.version,
                    "enabled": plugin.state.enabled && !plugin.state.faulted,
                }))
            })
            .collect::<Vec<_>>();
        providers.sort_by(|left, right| {
            let left_key = left.get("key").and_then(Value::as_str).unwrap_or_default();
            let right_key = right.get("key").and_then(Value::as_str).unwrap_or_default();
            left_key.cmp(right_key).then_with(|| {
                let left_enabled = left.get("enabled").and_then(Value::as_bool).unwrap_or(false);
                let right_enabled = right.get("enabled").and_then(Value::as_bool).unwrap_or(false);
                right_enabled.cmp(&left_enabled).then_with(|| {
                    left.get("pluginId").and_then(Value::as_str).unwrap_or_default()
                        .cmp(right.get("pluginId").and_then(Value::as_str).unwrap_or_default())
                })
            })
        });
        let mut seen_keys = std::collections::HashSet::new();
        providers.retain(|provider| {
            seen_keys.insert(
                provider
                    .get("key")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            )
        });
        Ok(providers)
    }

    pub fn sync_tag_providers(&self, media: &MediaService) -> Result<Value, String> {
        media
            .call(
                "media_tag_provider_reconcile",
                json!({ "providers": self.provider_values()? }),
            )
            .map_err(|error| error.to_string())
    }

    pub fn tag_provider_list(&self, media: &MediaService) -> Result<Value, String> {
        self.sync_tag_providers(media)?;
        media
            .call("media_tag_provider_list", json!({}))
            .map_err(|error| error.to_string())
    }

    pub fn tag_provider_refresh(
        &self,
        provider_key: String,
        media: &MediaService,
        bass: &BassService,
        app: &AppHandle,
    ) -> Result<Value, String> {
        self.sync_tag_providers(media)?;
        let rows = media
            .call("media_tag_provider_list", json!({}))
            .map_err(|error| error.to_string())?
            .get("providers")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let provider = rows
            .iter()
            .find(|provider| provider.get("key").and_then(Value::as_str) == Some(provider_key.as_str()))
            .cloned()
            .ok_or_else(|| "tag provider does not exist".to_string())?;
        if !provider.get("enabled").and_then(Value::as_bool).unwrap_or(false) {
            return Err("tag provider is not enabled".into());
        }
        let plugin_id = provider
            .get("pluginId")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "tag provider has no plugin source".to_string())?
            .to_string();
        if !self.is_plugin_running(&plugin_id) {
            return Err("the plugin backing this tag provider is not enabled".into());
        }
        let source_key = provider
            .get("sourceProviderKey")
            .and_then(Value::as_str)
            .unwrap_or(&provider_key)
            .to_string();
        let source_version = self
            .provider_values()?
            .iter()
            .find(|candidate| candidate.get("key").and_then(Value::as_str) == Some(source_key.as_str())
                && candidate.get("pluginId").and_then(Value::as_str) == Some(plugin_id.as_str()))
            .and_then(|candidate| candidate.get("sourceVersion").and_then(Value::as_str))
            .unwrap_or("")
            .to_string();
        let count = media
            .call("media_tag_catalog_list", json!({ "limit": 1, "offset": 0 }))
            .map_err(|error| error.to_string())?
            .get("total")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0) as usize;
        let job = media
            .call("media_tag_analysis_start", json!({ "providerKey": provider_key, "total": count }))
            .map_err(|error| error.to_string())?;
        let job_id = job
            .get("jobId")
            .and_then(Value::as_str)
            .ok_or_else(|| "analysis job did not return an id".to_string())?
            .to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        self.analysis_flags
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?
            .insert(job_id.clone(), cancel.clone());
        let manager = self.clone();
        let media = media.clone();
        let bass = bass.clone();
        let app = app.clone();
        let key_for_worker = provider_key.clone();
        let plugin_for_worker = plugin_id.clone();
        std::thread::spawn(move || {
            let mut offset = 0usize;
            let mut failed = None;
            while offset < count {
                if cancel.load(Ordering::Acquire) {
                    let _ = media.call("media_tag_analysis_finish", json!({ "jobId": job_id, "state": "cancelled" }));
                    let _ = app.emit(EVENT_TAG_ANALYSIS_ERROR, json!({ "jobId": job_id, "providerKey": key_for_worker, "state": "cancelled" }));
                    manager.remove_analysis_flag(&job_id);
                    return;
                }
                let catalog = match media.call("media_tag_catalog_list", json!({ "limit": 1, "offset": offset })) {
                    Ok(value) => value,
                    Err(error) => { failed = Some(error.to_string()); break; }
                };
                let Some(track) = catalog
                    .get("tracks")
                    .and_then(Value::as_array)
                    .and_then(|tracks| tracks.first())
                    .cloned()
                else {
                    break;
                };
                let response = match manager.call(
                    plugin_for_worker.clone(),
                    "backend.tag.analyze".into(),
                    json!({ "providerKey": source_key, "trackIndex": offset, "track": track }),
                    &bass,
                    &media,
                    &app,
                ) {
                    Ok(value) => value,
                    Err(error) => { failed = Some(error); break; }
                };
                let response = if response.get("ok").and_then(Value::as_bool) == Some(true) {
                    response.get("result").cloned().unwrap_or_else(|| json!({}))
                } else {
                    response
                };
                let results = response.get("results").cloned().unwrap_or_else(|| json!([]));
                if results.as_array().is_some_and(|values| !values.is_empty()) {
                    if let Err(error) = media.call("media_tag_results_write", json!({
                        "providerKey": key_for_worker,
                        "pluginId": plugin_for_worker,
                        "sourceVersion": source_version,
                        "results": results,
                    })) {
                        failed = Some(error.to_string());
                        break;
                    }
                }
                offset = offset.saturating_add(1);
                let _ = media.call("media_tag_analysis_progress", json!({ "jobId": job_id, "completed": offset }));
                let _ = app.emit(EVENT_TAG_ANALYSIS_PROGRESS, json!({ "jobId": job_id, "providerKey": key_for_worker, "completed": offset, "total": count }));
            }
            if let Some(error) = failed {
                let _ = media.call("media_tag_analysis_finish", json!({ "jobId": job_id, "state": "failed", "error": error }));
                let _ = app.emit(EVENT_TAG_ANALYSIS_ERROR, json!({ "jobId": job_id, "providerKey": key_for_worker, "error": error }));
            } else {
                let _ = media.call("media_tag_analysis_finish", json!({ "jobId": job_id, "state": "finished" }));
                let _ = app.emit(EVENT_TAG_ANALYSIS_FINISHED, json!({ "jobId": job_id, "providerKey": key_for_worker }));
            }
            manager.remove_analysis_flag(&job_id);
        });
        Ok(job)
    }

    fn remove_analysis_flag(&self, job_id: &str) {
        if let Ok(mut flags) = self.analysis_flags.lock() {
            flags.remove(job_id);
        }
    }

    pub fn tag_provider_cancel(&self, job_id: String, media: &MediaService) -> Result<Value, String> {
        let cancelled = self
            .analysis_flags
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?
            .get(&job_id)
            .map(|flag| { flag.store(true, Ordering::Release); true })
            .unwrap_or(false);
        let result = media
            .call("media_tag_provider_cancel", json!({ "jobId": job_id }))
            .map_err(|error| error.to_string())?;
        Ok(json!({ "cancelled": cancelled || result.get("cancelled").and_then(Value::as_bool).unwrap_or(false), "jobId": job_id }))
    }

    pub fn is_plugin_running(&self, plugin_id: &str) -> bool {
        self.runtime
            .lock()
            .map(|runtime| {
                runtime
                    .plugins
                    .get(plugin_id)
                    .map(|plugin| plugin.state.enabled && !plugin.state.faulted && plugin.wasm.is_some())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn tag_provider_wiki(
        &self,
        provider_key: String,
        locale: String,
        provided: bool,
        media: &MediaService,
    ) -> Result<Value, String> {
        let rows = media
            .call("media_tag_provider_list", json!({}))
            .map_err(|error| error.to_string())?
            .get("providers")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let provider = rows
            .iter()
            .find(|provider| provider.get("key").and_then(Value::as_str) == Some(provider_key.as_str()));
        let Some(provider) = provider else {
            return Ok(json!({ "wiki": "" }));
        };
        // A user-created tag owns its Wiki, including an intentionally empty one.
        if !provided && provider.get("tagId").and_then(Value::as_str).is_some() {
            return Ok(json!({ "wiki": provider.get("wiki").and_then(Value::as_str).unwrap_or(""), "source": "stored" }));
        }
        let Some(plugin_id) = provider.get("pluginId").and_then(Value::as_str).filter(|value| !value.trim().is_empty()) else {
            return Ok(json!({ "wiki": "" }));
        };
        let source_key = provider.get("sourceProviderKey").and_then(Value::as_str).unwrap_or(&provider_key);
        let result = self.read_plugin_wiki(plugin_id, "backend.tag.wiki", json!({ "providerKey": source_key, "locale": locale }))?;
        Ok(json!({ "wiki": result.get("wiki").and_then(Value::as_str).unwrap_or(""), "source": "plugin" }))
    }

    pub fn plugin_wiki(
        &self,
        plugin_id: String,
        locale: String,
    ) -> Result<Value, String> {
        let result = self.read_plugin_wiki(&plugin_id, "backend.wiki", json!({ "locale": locale }))?;
        Ok(json!({ "wiki": result.get("wiki").and_then(Value::as_str).unwrap_or("") }))
    }

    fn read_plugin_wiki(&self, id: &str, method: &str, args: Value) -> Result<Value, String> {
        let (engine, mut isolated) = {
            let runtime = self.runtime.lock().map_err(|_| "plugin runtime poisoned".to_string())?;
            let plugin = runtime.plugins.get(id).ok_or_else(|| "plugin is not installed".to_string())?;
            (runtime.engine.clone(), InstalledPlugin {
                manifest: plugin.manifest.clone(),
                root: plugin.root.clone(),
                state: PersistedPlugin::default(),
                wasm: None,
                last_background_tick_ms: 0,
            })
        };
        // Documentation is available before enabling a plugin or granting permissions.
        // Use a separate instance without host access so reading it cannot change a
        // running plugin's state, use its permissions, or fault its backend.
        start_wasm(&engine, &mut isolated)?;
        let request = json!({ "method": method, "args": args }).to_string();
        let response = call_wasm(&mut isolated, request.as_bytes(), None, WASM_FUEL);
        stop_wasm(&mut isolated);
        let response = response?;
        if response.get("ok").and_then(Value::as_bool) == Some(false)
            && response.get("error").and_then(Value::as_str).is_some_and(|error| {
                error.starts_with("unknown ") || error.starts_with("unsupported ")
            })
        {
            return Ok(json!({ "wiki": "" }));
        }
        unwrap_plugin_response(response)
    }

    pub fn pick_package(&self) -> Option<String> {
        FileDialog::new()
            .add_filter("Dropin plugin", &["dropin"])
            .pick_file()
            .map(|path| path.to_string_lossy().into_owned())
    }

    pub fn install(&self, source: String) -> Result<PluginInfo, String> {
        let source_path = PathBuf::from(source);
        if source_path.extension().and_then(|value| value.to_str()) != Some("dropin") {
            return Err("plugin package must use the .dropin extension".into());
        }
        let file = File::open(&source_path).map_err(|error| error.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
        let temporary = self
            .paths
            .plugins_dir
            .join(format!(".install-{}", std::process::id()));
        if temporary.exists() {
            fs::remove_dir_all(&temporary).map_err(|error| error.to_string())?;
        }
        fs::create_dir_all(&temporary).map_err(|error| error.to_string())?;
        let result = (|| {
            extract_archive(&mut archive, &temporary)?;
            let manifest_path = temporary.join("plugin.json");
            let contents = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
            let manifest: PluginManifest =
                serde_json::from_str(&contents).map_err(|error| error.to_string())?;
            manifest.validate()?;
            if !manifest.backend_path(&temporary).is_file() {
                return Err("plugin backend entry is missing".into());
            }
            if let Some(ui) = manifest.ui_path(&temporary) {
                if !ui.is_file() {
                    return Err("plugin UI entry is missing".into());
                }
            }
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "plugin runtime poisoned".to_string())?;
            if runtime.plugins.contains_key(&manifest.id)
                || self.paths.plugins_dir.join(&manifest.id).exists()
            {
                return Err(format!("plugin {} is already installed", manifest.id));
            }
            let destination = self.paths.plugins_dir.join(&manifest.id);
            fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
            fs::rename(&temporary, destination.join("current"))
                .map_err(|error| error.to_string())?;
            let state = PersistedPlugin {
                id: manifest.id.clone(),
                version: manifest.version.clone(),
                ..Default::default()
            };
            let plugin_id = manifest.id.clone();
            runtime.plugins.insert(
                plugin_id.clone(),
                InstalledPlugin {
                    manifest,
                    root: destination.join("current"),
                    state,
                    wasm: None,
                    last_background_tick_ms: 0,
                },
            );
            persist_states(&self.paths.plugins_file, &runtime.plugins)?;
            let info = plugin_info(runtime.plugins.get(&plugin_id).expect("inserted plugin"));
            Ok(info)
        })();
        if result.is_err() && temporary.exists() {
            let _ = fs::remove_dir_all(&temporary);
        }
        result
    }

    pub fn uninstall(&self, id: String) -> Result<Value, String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let plugin = runtime
            .plugins
            .remove(&id)
            .ok_or_else(|| "plugin is not installed".to_string())?;
        let install_root = self.paths.plugins_dir.join(&id);
        if install_root.exists() {
            fs::remove_dir_all(install_root).map_err(|error| error.to_string())?;
        }
        persist_states(&self.paths.plugins_file, &runtime.plugins)?;
        Ok(
            json!({ "id": id, "uninstalled": true, "dataPreserved": self.paths.plugin_data_dir.join(plugin.manifest.id).exists() }),
        )
    }

    pub fn set_enabled(&self, id: String, enabled: bool) -> Result<PluginInfo, String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let engine = runtime.engine.clone();
        if enabled {
            let requested_key = runtime
                .plugins
                .get(&id)
                .and_then(|plugin| plugin.manifest.tag_provider.as_ref())
                .map(|provider| provider.key.clone());
            if let Some(requested_key) = requested_key {
                let conflict = runtime.plugins.values().any(|plugin| {
                    plugin.manifest.id != id
                        && plugin.state.enabled
                        && plugin.manifest.tag_provider.as_ref().is_some_and(|provider| provider.key == requested_key)
                });
                if conflict {
                    return Err(format!("tag provider key is already enabled: {requested_key}"));
                }
            }
        }
        let info = {
            let plugin = runtime
                .plugins
                .get_mut(&id)
                .ok_or_else(|| "plugin is not installed".to_string())?;
            if enabled {
                if plugin.state.faulted {
                    return Err("faulted plugin must be reinstalled before enabling".into());
                }
                start_wasm(&engine, plugin)?;
                plugin.state.enabled = true;
            } else {
                stop_wasm(plugin);
                plugin.state.enabled = false;
            }
            plugin_info(plugin)
        };
        persist_states(&self.paths.plugins_file, &runtime.plugins)?;
        Ok(info)
    }

    pub fn permissions(&self, id: String) -> Result<Value, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let plugin = runtime
            .plugins
            .get(&id)
            .ok_or_else(|| "plugin is not installed".to_string())?;
        Ok(
            json!({ "id": id, "declared": plugin.manifest.permissions, "granted": plugin.state.granted_permissions }),
        )
    }

    pub fn set_permissions(&self, id: String, granted: Vec<String>) -> Result<PluginInfo, String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let info = {
            let plugin = runtime
                .plugins
                .get_mut(&id)
                .ok_or_else(|| "plugin is not installed".to_string())?;
            crate::plugin::permissions::validate_permissions(&granted)?;
            if granted
                .iter()
                .any(|permission| !plugin.manifest.permissions.contains(permission))
            {
                return Err("cannot grant a permission not declared by the plugin".into());
            }
            plugin.state.granted_permissions = granted;
            plugin_info(plugin)
        };
        persist_states(&self.paths.plugins_file, &runtime.plugins)?;
        Ok(info)
    }

    pub fn update_host_state(&self, state: Value) -> Result<(), String> {
        let mut host_state = self
            .host_state
            .write()
            .map_err(|_| "plugin host state poisoned".to_string())?;
        *host_state = state;
        Ok(())
    }

    pub fn call(
        &self,
        id: String,
        method: String,
        args: Value,
        bass: &BassService,
        media: &MediaService,
        app: &AppHandle,
    ) -> Result<Value, String> {
        if serde_json::to_vec(&args)
            .map_err(|error| error.to_string())?
            .len()
            > MAX_REQUEST_BYTES
        {
            return Err("plugin request exceeds 1 MiB".into());
        }
        let host = HostApiContext {
            paths: self.paths.clone(),
            host_state: self.host_state.clone(),
            bass: bass.clone(),
            media: media.clone(),
            app: app.clone(),
        };
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let plugin = runtime
            .plugins
            .get_mut(&id)
            .ok_or_else(|| "plugin is not installed".to_string())?;
        if !plugin.state.enabled || plugin.state.faulted {
            return Err("plugin is not enabled".into());
        }
        let access = PluginAccess::from_plugin(plugin);
        if !method.starts_with("backend.") {
            return dispatch_host_api(&access, &method, args, &host);
        }
        let permission = if method.starts_with("backend.tag.") {
            TAG_PROVIDER
        } else {
            UI_PANEL
        };
        if !access_permission(&access, permission) {
            return permission_error(permission);
        }
        let request = json!({ "method": method, "args": args }).to_string();
        let fuel = if method.starts_with("backend.tag.analyze") {
            TAG_ANALYSIS_FUEL
        } else {
            WASM_FUEL
        };
        let wasm_context = WasmHostContext { access, host };
        match call_wasm(plugin, request.as_bytes(), Some(wasm_context), fuel) {
            Ok(value) => Ok(value),
            Err(error) => {
                stop_wasm(plugin);
                plugin.state.enabled = false;
                plugin.state.faulted = true;
                plugin.state.last_error = Some(error.clone());
                let _ = persist_states(&self.paths.plugins_file, &runtime.plugins);
                Err(error)
            }
        }
    }

    pub fn tick_background(
        &self,
        bass: &BassService,
        media: &MediaService,
        app: &AppHandle,
    ) -> Result<(), String> {
        let now_ms = unix_time_ms();
        let host = HostApiContext {
            paths: self.paths.clone(),
            host_state: self.host_state.clone(),
            bass: bass.clone(),
            media: media.clone(),
            app: app.clone(),
        };
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let mut persist_needed = false;
        for plugin in runtime.plugins.values_mut() {
            let Some(interval_ms) = plugin.manifest.background.tick_interval_ms else {
                continue;
            };
            if !plugin.state.enabled || plugin.state.faulted {
                continue;
            }
            if plugin.last_background_tick_ms != 0
                && now_ms.saturating_sub(plugin.last_background_tick_ms) < interval_ms
            {
                continue;
            }
            plugin.last_background_tick_ms = now_ms;
            let access = PluginAccess::from_plugin(plugin);
            if !access_permission(&access, UI_PANEL) {
                continue;
            }
            let request = json!({
                "method": "backend.tick",
                "args": { "nowMs": now_ms }
            })
            .to_string();
            let wasm_context = WasmHostContext {
                access,
                host: host.clone(),
            };
            if let Err(error) = call_wasm(plugin, request.as_bytes(), Some(wasm_context), WASM_FUEL) {
                stop_wasm(plugin);
                plugin.state.enabled = false;
                plugin.state.faulted = true;
                plugin.state.last_error = Some(error);
                persist_needed = true;
            }
        }
        if persist_needed {
            persist_states(&self.paths.plugins_file, &runtime.plugins)?;
        }
        Ok(())
    }

    pub fn ui_url(&self, id: String) -> Result<String, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let plugin = runtime
            .plugins
            .get(&id)
            .ok_or_else(|| "plugin is not installed".to_string())?;
        if !plugin.state.enabled || plugin.state.faulted || !plugin_permission(plugin, UI_PANEL) {
            return Err("plugin UI permission is not granted".into());
        }
        let ui = plugin
            .manifest
            .ui
            .as_deref()
            .ok_or_else(|| "plugin does not provide a UI".to_string())?;
        let path = format!("/{}/{}", plugin.manifest.id, ui);
        if cfg!(any(target_os = "windows", target_os = "android")) {
            Ok(format!("http://dropin-plugin.localhost{path}"))
        } else {
            Ok(format!("dropin-plugin://localhost{path}"))
        }
    }

    pub fn serve(&self, request_path: &str) -> Result<(Vec<u8>, String), String> {
        let mut parts = request_path.trim_start_matches('/').split('/');
        let id = parts
            .next()
            .ok_or_else(|| "missing plugin id".to_string())?;
        let relative = parts.collect::<Vec<_>>().join("/");
        crate::plugin::manifest::validate_id(id)?;
        crate::plugin::manifest::validate_relative_file(&relative, "resource")?;
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "plugin runtime poisoned".to_string())?;
        let plugin = runtime
            .plugins
            .get(id)
            .ok_or_else(|| "plugin is not installed".to_string())?;
        let root = plugin
            .root
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let path = root
            .join(&relative)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !path.starts_with(&root) {
            return Err("plugin resource escaped its directory".into());
        }
        let is_icon = plugin.manifest.icon.as_deref() == Some(relative.as_str());
        if (!plugin.state.enabled || plugin.state.faulted || !plugin_permission(plugin, UI_PANEL))
            && !is_icon
        {
            return Err("plugin UI permission is not granted".into());
        }
        let data = fs::read(&path).map_err(|error| error.to_string())?;
        Ok((data, mime_for(&path)))
    }
}

fn start_wasm(engine: &Engine, plugin: &mut InstalledPlugin) -> Result<(), String> {
    let bytes =
        fs::read(plugin.manifest.backend_path(&plugin.root)).map_err(|error| error.to_string())?;
    let module = Module::new(engine, bytes).map_err(|error| error.to_string())?;
    let limits = StoreLimitsBuilder::new()
        .memory_size(WASM_MEMORY_BYTES)
        .instances(1)
        .tables(8)
        .memories(1)
        .build();
    let mut store = Store::new(
        engine,
        WasmStoreData {
            limits,
            host_context: None,
        },
    );
    store.limiter(|data| &mut data.limits);
    store
        .set_fuel(WASM_FUEL)
        .map_err(|error| error.to_string())?;
    let mut linker = Linker::<WasmStoreData>::new(engine);
    linker
        .func_wrap(
            "dropin",
            "host_call",
            |caller: Caller<'_, WasmStoreData>, ptr: i32, len: i32| -> i64 {
                wasm_host_call(caller, ptr, len)
            },
        )
        .map_err(|error| error.to_string())?;
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|error| error.to_string())?;
    let init = instance
        .get_typed_func::<i32, i32>(&mut store, "plugin_init")
        .map_err(|error| error.to_string())?;
    if init
        .call(&mut store, API_VERSION as i32)
        .map_err(|error| error.to_string())?
        != 0
    {
        return Err("plugin_init returned an error".into());
    }
    for name in [
        "plugin_alloc",
        "plugin_dealloc",
        "plugin_call",
        "plugin_free_response",
        "plugin_shutdown",
    ] {
        if instance.get_func(&mut store, name).is_none() {
            return Err(format!("WASM export is missing: {name}"));
        }
    }
    plugin.wasm = Some(WasmPlugin { store, instance });
    Ok(())
}

fn stop_wasm(plugin: &mut InstalledPlugin) {
    if let Some(mut wasm) = plugin.wasm.take() {
        let _ = wasm.store.set_fuel(WASM_FUEL);
        if let Some(shutdown) = wasm.instance.get_func(&mut wasm.store, "plugin_shutdown") {
            if let Ok(shutdown) = shutdown.typed::<(), ()>(&wasm.store) {
                let _ = shutdown.call(&mut wasm.store, ());
            }
        }
    }
}

fn call_wasm(
    plugin: &mut InstalledPlugin,
    request: &[u8],
    host_context: Option<WasmHostContext>,
    fuel: u64,
) -> Result<Value, String> {
    let wasm = plugin
        .wasm
        .as_mut()
        .ok_or_else(|| "plugin backend is not running".to_string())?;
    wasm.store
        .set_fuel(fuel)
        .map_err(|error| error.to_string())?;
    wasm.store.data_mut().host_context = host_context;

    let result = (|| {
        let memory = wasm
            .instance
            .get_memory(&mut wasm.store, "memory")
            .ok_or_else(|| "WASM memory export is missing".to_string())?;
        let alloc = wasm
            .instance
            .get_typed_func::<i32, i32>(&mut wasm.store, "plugin_alloc")
            .map_err(|error| error.to_string())?;
        let dealloc = wasm
            .instance
            .get_typed_func::<(i32, i32), ()>(&mut wasm.store, "plugin_dealloc")
            .map_err(|error| error.to_string())?;
        let call = wasm
            .instance
            .get_typed_func::<(i32, i32), i64>(&mut wasm.store, "plugin_call")
            .map_err(|error| error.to_string())?;
        let ptr = alloc
            .call(&mut wasm.store, request.len() as i32)
            .map_err(|error| error.to_string())?;
        if ptr < 0 {
            return Err("WASM allocation failed".into());
        }
        memory
            .write(&mut wasm.store, ptr as usize, request)
            .map_err(|error| error.to_string())?;
        let packed = call
            .call(&mut wasm.store, (ptr, request.len() as i32))
            .map_err(|error| error.to_string())?;
        let _ = dealloc.call(&mut wasm.store, (ptr, request.len() as i32));
        let response_ptr = (packed >> 32) as i32;
        let response_len = (packed & 0xffff_ffff) as i32;
        if response_ptr < 0 || response_len < 0 || response_len as usize > MAX_REQUEST_BYTES {
            return Err("invalid WASM response".into());
        }
        let mut bytes = vec![0u8; response_len as usize];
        memory
            .read(&mut wasm.store, response_ptr as usize, &mut bytes)
            .map_err(|error| error.to_string())?;
        if let Some(free) = wasm
            .instance
            .get_func(&mut wasm.store, "plugin_free_response")
        {
            let free = free
                .typed::<(i32, i32), ()>(&wasm.store)
                .map_err(|error| error.to_string())?;
            free.call(&mut wasm.store, (response_ptr, response_len))
                .map_err(|error| error.to_string())?;
        }
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid WASM JSON response: {error}"))
    })();

    wasm.store.data_mut().host_context = None;
    result
}

fn wasm_host_call(mut caller: Caller<'_, WasmStoreData>, ptr: i32, len: i32) -> i64 {
    let payload = match wasm_host_call_inner(&mut caller, ptr, len) {
        Ok(result) => json!({ "ok": true, "result": result }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    let bytes = serde_json::to_vec(&payload).unwrap_or_else(|_| {
        b"{\"ok\":false,\"error\":\"host response serialization failed\"}".to_vec()
    });
    write_wasm_response(&mut caller, &bytes).unwrap_or(0)
}

fn wasm_host_call_inner(
    caller: &mut Caller<'_, WasmStoreData>,
    ptr: i32,
    len: i32,
) -> Result<Value, String> {
    if ptr < 0 || len < 0 || len as usize > MAX_REQUEST_BYTES {
        return Err("invalid host_call request range".into());
    }
    let memory = wasm_memory(caller)?;
    let mut bytes = vec![0u8; len as usize];
    memory
        .read(&mut *caller, ptr as usize, &mut bytes)
        .map_err(|error| error.to_string())?;
    let request: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid host_call JSON request: {error}"))?;
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| "host_call method is required".to_string())?;
    let args = request.get("args").cloned().unwrap_or_else(|| json!({}));
    let context = caller
        .data()
        .host_context
        .clone()
        .ok_or_else(|| "host_call is only available during plugin_call".to_string())?;
    dispatch_host_api(&context.access, method, args, &context.host)
}

fn write_wasm_response(
    caller: &mut Caller<'_, WasmStoreData>,
    response: &[u8],
) -> Result<i64, String> {
    if response.len() > MAX_REQUEST_BYTES {
        return Err("host_call response exceeds 1 MiB".into());
    }
    let memory = wasm_memory(caller)?;
    let alloc = caller
        .get_export("plugin_alloc")
        .and_then(|item| item.into_func())
        .ok_or_else(|| "WASM export is missing: plugin_alloc".to_string())?
        .typed::<i32, i32>(&mut *caller)
        .map_err(|error| error.to_string())?;
    let ptr = alloc
        .call(&mut *caller, response.len() as i32)
        .map_err(|error| error.to_string())?;
    if ptr < 0 {
        return Err("WASM allocation failed".into());
    }
    memory
        .write(&mut *caller, ptr as usize, response)
        .map_err(|error| error.to_string())?;
    Ok(pack_response(ptr, response.len()))
}

fn wasm_memory(caller: &mut Caller<'_, WasmStoreData>) -> Result<wasmtime::Memory, String> {
    caller
        .get_export("memory")
        .and_then(|item| item.into_memory())
        .ok_or_else(|| "WASM memory export is missing".to_string())
}

fn pack_response(ptr: i32, len: usize) -> i64 {
    ((ptr as i64) << 32) | (len as i64 & 0xffff_ffff)
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

fn unwrap_plugin_response(response: Value) -> Result<Value, String> {
    if response.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(response.get("result").cloned().unwrap_or_else(|| json!({})))
    } else {
        Err(response
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("plugin call failed")
            .to_string())
    }
}

fn dispatch_host_api(
    access: &PluginAccess,
    method: &str,
    args: Value,
    host: &HostApiContext,
) -> Result<Value, String> {
    let required = required_permission(method)
        .ok_or_else(|| format!("unsupported plugin method: {method}"))?;
    if !access_permission(access, required) {
        return permission_error(required);
    }

    match method {
        "player.getState" => host
            .host_state
            .read()
            .map(|state| state.clone())
            .map_err(|_| "plugin host state poisoned".to_string()),
        "player.play" | "player.pause" => player_control_call(method, args, host),
        "library.list" => {
            if access.is_tag_provider {
                return Err("tag providers must use library.catalog.list and cannot read playlist sources".into());
            }
            host.media
                .call("media_library_tracks", args)
                .map_err(|error| error.to_string())
        }
        "library.catalog.list" => {
            let catalog = host
                .media
                .call("media_tag_catalog_list", args)
                .map_err(|error| error.to_string())?;
            Ok(sanitize_catalog(catalog))
        }
        "library.audio.read" => read_library_audio(args, host),
        "notification.show" => notification_show(access, args, &host.app),
        "storage.get" | "storage.set" | "storage.remove" => {
            storage_call(&host.paths, &access.id, method, args)
        }
        _ => Err(format!("unsupported plugin method: {method}")),
    }
}

fn sanitize_catalog(mut catalog: Value) -> Value {
    if let Some(tracks) = catalog.get_mut("tracks").and_then(Value::as_array_mut) {
        for track in tracks {
            if let Some(object) = track.as_object_mut() {
                object.remove("path");
                object.remove("url");
                object.remove("source");
                object.remove("fileHash");
            }
        }
    }
    catalog
}

fn player_control_call(method: &str, args: Value, host: &HostApiContext) -> Result<Value, String> {
    let state_channel_id = || -> Result<Option<u64>, String> {
        let state = host
            .host_state
            .read()
            .map_err(|_| "plugin host state poisoned".to_string())?;
        Ok(state.get("channelId").and_then(Value::as_u64))
    };
    let channel_id = args
        .get("channelId")
        .and_then(Value::as_u64)
        .or(state_channel_id()?)
        .ok_or_else(|| "no active player channel".to_string())?;
    let operation = if method == "player.play" {
        "bass_channel_play"
    } else {
        "bass_channel_pause"
    };
    let mut payload = json!({ "channelId": channel_id });
    if let Some(restart) = args.get("restart").and_then(Value::as_bool) {
        payload["restart"] = Value::Bool(restart);
    }
    host.bass
        .call_operation(operation, payload)
        .map_err(|error| error.to_string())
}

fn read_library_audio(args: Value, host: &HostApiContext) -> Result<Value, String> {
    let track_id = args
        .get("trackId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "trackId is required".to_string())?;
    let start_ms = args.get("startMs").and_then(Value::as_u64).unwrap_or(0);
    let end_ms = args.get("endMs").and_then(Value::as_u64).unwrap_or(start_ms.saturating_add(10_000));
    if end_ms < start_ms || end_ms.saturating_sub(start_ms) > 60_000 {
        return Err("audio read range must be between 0 and 60000 ms".into());
    }
    let source = host
        .media
        .call("media_track_source", json!({ "trackId": track_id }))
        .map_err(|error| error.to_string())?;
    let path = source
        .get("analysisPath")
        .or_else(|| source.get("path"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "track audio is not available for analysis".to_string())?;
    let max_samples = args.get("maxSamples").and_then(Value::as_u64).unwrap_or(48_000).clamp(1, 80_000);
    // 多区间流式读取：host 单次打开文件顺序采样。限制总采样数以保证响应远小于 1 MiB 上限
    let ranges = match args.get("ranges").and_then(Value::as_array) {
        Some(items) if !items.is_empty() => {
            let mut total_samples: u64 = 0;
            for item in items {
                total_samples += item
                    .get("maxSamples")
                    .and_then(Value::as_u64)
                    .unwrap_or(48_000)
                    .clamp(1, 80_000);
            }
            if items.len() > 32 {
                return Err("too many audio read ranges (max 32)".to_string());
            }
            if total_samples > 64_000 {
                return Err("audio read ranges exceed 64000 samples in total".to_string());
            }
            args.get("ranges").cloned().unwrap_or(Value::Null)
        }
        _ => Value::Null,
    };
    let mut request = json!({ "path": path, "startMs": start_ms, "endMs": end_ms, "maxSamples": max_samples });
    if !ranges.is_null() {
        request["ranges"] = ranges;
    }
    let result = host
        .bass
        .call_operation("bass_analysis_read", request)
        .map_err(|error| error.to_string())?;
    Ok(json!({ "trackId": track_id, "startMs": start_ms, "endMs": end_ms,
        "sampleRate": result.get("sampleRate").cloned().unwrap_or(json!(0)),
        "channels": result.get("channels").cloned().unwrap_or(json!(0)),
        "rangeCount": result.get("rangeCount").cloned().unwrap_or(json!(1)),
        "samples": result.get("samples").cloned().unwrap_or_else(|| json!([])),
    }))
}

fn notification_show(access: &PluginAccess, args: Value, app: &AppHandle) -> Result<Value, String> {
    let title = args
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Plugin notification");
    let body = args
        .get("body")
        .or_else(|| args.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if title.len() > 200 || body.len() > 4096 {
        return Err("notification payload is too large".into());
    }
    let duration = args
        .get("duration")
        .or_else(|| args.get("durationMs"))
        .and_then(Value::as_i64)
        .unwrap_or(5000)
        .clamp(0, 600_000);
    let source = if access.name.trim().is_empty() {
        access.id.as_str()
    } else {
        access.name.as_str()
    };
    app.emit(
        EVENT_PLUGIN_NOTIFICATION,
        json!({
            "pluginId": access.id,
            "pluginName": access.name,
            "source": source,
            "title": title,
            "body": body,
            "duration": duration,
        }),
    )
    .map_err(|error| error.to_string())?;
    Ok(json!({ "shown": true }))
}

fn required_permission(method: &str) -> Option<&'static str> {
    if method == "ui.panel" || method == "ui.getInfo" {
        Some(UI_PANEL)
    } else if method.starts_with("player.get") {
        Some(PLAYER_READ)
    } else if method.starts_with("player.") {
        Some(PLAYER_CONTROL)
    } else if method == "library.catalog.list" {
        Some(LIBRARY_READ)
    } else if method == "library.audio.read" {
        Some(LIBRARY_AUDIO_READ)
    } else if method.starts_with("library.") {
        Some(LIBRARY_READ)
    } else if method == "notification.show" {
        Some(NOTIFICATION_SHOW)
    } else if method.starts_with("storage.") {
        Some(STORAGE_PLUGIN)
    } else {
        None
    }
}

fn access_permission(access: &PluginAccess, permission: &str) -> bool {
    has_permission(
        &access.declared_permissions,
        &access.granted_permissions,
        permission,
    )
}

fn plugin_permission(plugin: &InstalledPlugin, permission: &str) -> bool {
    has_permission(
        &plugin.manifest.permissions,
        &plugin.state.granted_permissions,
        permission,
    )
}

fn has_permission(declared: &[String], granted: &[String], permission: &str) -> bool {
    declared.iter().any(|item| item == permission) && granted.iter().any(|item| item == permission)
}

fn permission_error(permission: &str) -> Result<Value, String> {
    Err(json!({ "code": "permission_denied", "permission": permission }).to_string())
}

fn storage_call(
    paths: &AppPaths,
    plugin_id: &str,
    method: &str,
    args: Value,
) -> Result<Value, String> {
    let dir = paths.plugin_data_dir.join(plugin_id);
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let file = dir.join("storage.json");
    let mut data = if file.exists() {
        serde_json::from_str::<serde_json::Map<String, Value>>(
            &fs::read_to_string(&file).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?
    } else {
        serde_json::Map::new()
    };
    let key = args
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| "storage key is required".to_string())?;
    if key.len() > 256 || key.contains('/') || key.contains('\\') {
        return Err("invalid storage key".into());
    }
    match method {
        "storage.get" => Ok(data.get(key).cloned().unwrap_or(Value::Null)),
        "storage.set" => {
            data.insert(
                key.into(),
                args.get("value").cloned().unwrap_or(Value::Null),
            );
            fs::write(
                &file,
                serde_json::to_vec_pretty(&data).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            Ok(json!({ "saved": true }))
        }
        "storage.remove" => {
            data.remove(key);
            fs::write(
                &file,
                serde_json::to_vec_pretty(&data).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            Ok(json!({ "removed": true }))
        }
        _ => Err("unsupported storage method".into()),
    }
}

fn plugin_info(plugin: &InstalledPlugin) -> PluginInfo {
    PluginInfo {
        id: plugin.manifest.id.clone(),
        name: plugin.manifest.name.clone(),
        version: plugin.manifest.version.clone(),
        api_version: plugin.manifest.api_version,
        author: plugin.manifest.author.clone(),
        description: plugin.manifest.description.clone(),
        categories: plugin.manifest.categories.clone(),
        ui: plugin.manifest.ui.clone(),
        tag_provider: plugin.manifest.tag_provider.clone(),
        icon: plugin.manifest.icon.clone(),
        icon_url: plugin_icon_url(plugin),
        permissions: PermissionState::new(
            plugin.manifest.permissions.clone(),
            plugin.state.granted_permissions.clone(),
        ),
        installed: true,
        enabled: plugin.state.enabled,
        faulted: plugin.state.faulted,
        last_error: plugin.state.last_error.clone(),
    }
}

fn plugin_icon_url(plugin: &InstalledPlugin) -> Option<String> {
    let relative = plugin.manifest.icon.as_deref()?;
    let path = plugin.root.join(relative);
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return None;
    }
    let path = format!("/{}/{}", plugin.manifest.id, relative);
    #[cfg(any(target_os = "windows", target_os = "android"))]
    { Some(format!("http://dropin-plugin.localhost{path}")) }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    { Some(format!("dropin-plugin://localhost{path}")) }
}

fn extract_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    destination: &Path,
) -> Result<(), String> {
    if archive.len() > MAX_FILES {
        return Err("plugin package contains too many files".into());
    }
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut item = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = item
            .name()
            .map_err(|error| error.to_string())?
            .replace('\\', "/");
        if name.is_empty()
            || name.starts_with('/')
            || name.contains(':')
            || name.split('/').any(|part| part == "..")
            || item
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("plugin package contains an unsafe path".into());
        }
        if item.is_dir() {
            continue;
        }
        let size = item.size();
        if size > MAX_FILE_BYTES || total.saturating_add(size) > MAX_ARCHIVE_BYTES {
            return Err("plugin package is too large".into());
        }
        total += size;
        let path = destination.join(&name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut output = File::create(path).map_err(|error| error.to_string())?;
        std::io::copy(&mut item, &mut output).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn load_state(path: &Path, manifest: &PluginManifest) -> Option<PersistedPlugin> {
    let contents = fs::read_to_string(path).ok()?;
    let payload = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
    if payload.get("version").and_then(Value::as_u64) != Some(STATE_VERSION as u64) {
        return None;
    }
    let states = payload
        .get("plugins")?
        .as_array()?
        .iter()
        .filter_map(|value| serde_json::from_value::<PersistedPlugin>(value.clone()).ok())
        .collect::<Vec<_>>();
    states
        .into_iter()
        .find(|state| state.id == manifest.id && state.version == manifest.version)
}

fn persist_states(path: &Path, plugins: &HashMap<String, InstalledPlugin>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let states = plugins
        .values()
        .map(|plugin| plugin.state.clone())
        .collect::<Vec<_>>();
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&json!({ "version": STATE_VERSION, "plugins": states }))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn mime_for(path: &Path) -> String {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
    .into()
}

#[tauri::command]
pub fn plugin_list(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
) -> Result<Vec<PluginInfo>, String> {
    manager.sync_tag_providers(&media)?;
    manager.list()
}

#[tauri::command]
pub fn plugin_pick_package(manager: State<'_, PluginManager>) -> Option<String> {
    manager.pick_package()
}

#[tauri::command]
pub fn plugin_install(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    path: String,
) -> Result<PluginInfo, String> {
    let plugin = manager.install(path)?;
    manager.sync_tag_providers(&media)?;
    Ok(plugin)
}

#[tauri::command]
pub fn plugin_uninstall(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    id: String,
) -> Result<Value, String> {
    let result = manager.uninstall(id)?;
    manager.sync_tag_providers(&media)?;
    Ok(result)
}

#[tauri::command]
pub fn plugin_enable(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    id: String,
) -> Result<PluginInfo, String> {
    let plugin = manager.set_enabled(id, true)?;
    manager.sync_tag_providers(&media)?;
    Ok(plugin)
}

#[tauri::command]
pub fn plugin_disable(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    id: String,
) -> Result<PluginInfo, String> {
    let plugin = manager.set_enabled(id, false)?;
    manager.sync_tag_providers(&media)?;
    Ok(plugin)
}

#[tauri::command]
pub fn plugin_get_permissions(
    manager: State<'_, PluginManager>,
    id: String,
) -> Result<Value, String> {
    manager.permissions(id)
}

#[tauri::command]
pub fn plugin_set_permissions(
    manager: State<'_, PluginManager>,
    id: String,
    granted: Vec<String>,
) -> Result<PluginInfo, String> {
    manager.set_permissions(id, granted)
}

#[tauri::command]
pub fn plugin_call(
    manager: State<'_, PluginManager>,
    bass: State<'_, BassService>,
    media: State<'_, MediaService>,
    app: AppHandle,
    id: String,
    method: String,
    args: Value,
) -> Result<Value, String> {
    manager.call(id, method, args, &bass, &media, &app)
}

#[tauri::command]
pub fn plugin_update_host_state(
    manager: State<'_, PluginManager>,
    state: Value,
) -> Result<(), String> {
    manager.update_host_state(state)
}

#[tauri::command]
pub fn plugin_get_ui_url(manager: State<'_, PluginManager>, id: String) -> Result<String, String> {
    manager.ui_url(id)
}

#[tauri::command]
pub fn tag_provider_list(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
) -> Result<Value, String> {
    manager.tag_provider_list(&media)
}

#[tauri::command]
pub fn tag_provider_refresh(
    manager: State<'_, PluginManager>,
    bass: State<'_, BassService>,
    media: State<'_, MediaService>,
    app: AppHandle,
    provider_key: String,
) -> Result<Value, String> {
    manager.tag_provider_refresh(provider_key, &media, &bass, &app)
}

#[tauri::command]
pub fn tag_provider_cancel(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    job_id: String,
) -> Result<Value, String> {
    manager.tag_provider_cancel(job_id, &media)
}

#[tauri::command]
pub fn tag_provider_wiki(
    manager: State<'_, PluginManager>,
    media: State<'_, MediaService>,
    provider_key: String,
    locale: Option<String>,
    provided: Option<bool>,
) -> Result<Value, String> {
    manager.tag_provider_wiki(provider_key, locale.unwrap_or_default(), provided.unwrap_or(false), &media)
}

#[tauri::command]
pub fn plugin_wiki(
    manager: State<'_, PluginManager>,
    plugin_id: String,
    locale: Option<String>,
) -> Result<Value, String> {
    manager.plugin_wiki(plugin_id, locale.unwrap_or_default())
}

#[cfg(test)]
mod wiki_tests {
    use super::*;

    fn manager_with_examples() -> PluginManager {
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugin-sdk/examples");
        let manager = PluginManager::new(crate::core::paths::resolve(None));
        {
            let mut runtime = manager.runtime.lock().expect("runtime");
            for name in ["tag-provider", "sleep-timer"] {
                let root = examples.join(name);
                let manifest: PluginManifest = serde_json::from_str(&fs::read_to_string(root.join("plugin.json")).expect("manifest file"))
                    .expect("manifest");
                runtime.plugins.insert(manifest.id.clone(), InstalledPlugin {
                    manifest, root, state: PersistedPlugin::default(), wasm: None, last_background_tick_ms: 0,
                });
            }
        }
        manager
    }

    #[test]
    fn disabled_headless_and_ui_plugins_supply_localized_wikis_without_permissions() {
        let manager = manager_with_examples();
        for id in ["com.dropin.tag-energy", "com.dropin.sleep-timer"] {
            assert!(!manager.is_plugin_running(id));
            for locale in ["en-US", "zh-CN", "zh-CLASSICAL"] {
                let wiki = manager.plugin_wiki(id.into(), locale.into()).expect("plugin wiki");
                assert!(wiki["wiki"].as_str().is_some_and(|value| !value.is_empty()));
            }
            let runtime = manager.runtime.lock().expect("runtime");
            let plugin = runtime.plugins.get(id).expect("installed plugin");
            assert!(!plugin.state.enabled && !plugin.state.faulted);
            assert!(plugin.wasm.is_none() && plugin.state.granted_permissions.is_empty());
        }
        let wiki = manager.read_plugin_wiki("com.dropin.tag-energy", "backend.tag.wiki", json!({ "providerKey": "energy", "locale": "zh-CN" }))
            .expect("tag wiki");
        assert!(wiki["wiki"].as_str().expect("markdown").contains("Energy 是什么"));
        let fallback = manager.plugin_wiki("com.dropin.tag-energy".into(), "fr-FR".into()).expect("fallback");
        assert!(fallback["wiki"].as_str().expect("markdown").contains("What is Energy"));
    }

    #[test]
    fn wiki_instances_are_isolated_and_optional_methods_do_not_fault_plugins() {
        let manager = manager_with_examples();
        let id = "com.dropin.sleep-timer";
        let started = manager.read_plugin_wiki(id, "backend.start", json!({ "nowMs": 1000, "durationMs": 5000 }))
            .expect("isolated timer start");
        assert_eq!(started["active"], true);
        let fresh = manager.read_plugin_wiki(id, "backend.state", json!({ "nowMs": 1000 })).expect("fresh instance");
        assert_eq!(fresh["active"], false);
        let missing = manager.read_plugin_wiki(id, "backend.tag.wiki", json!({})).expect("optional wiki");
        assert_eq!(missing["wiki"], "");
        let inaccessible = manager.read_plugin_wiki("com.dropin.tag-energy", "backend.tag.analyze", json!({ "track": { "id": "track-a" } }))
            .expect("host access denied and track skipped");
        assert_eq!(inaccessible["results"], json!([]));
        assert!(!manager.is_plugin_running(id));
    }
}
