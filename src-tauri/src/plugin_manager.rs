use crate::{
    bass_bridge::BassService,
    media_library::MediaService,
    paths::AppPaths,
    plugin_manifest::PluginManifest,
    plugin_permissions::{
        PermissionState, API_VERSION, LIBRARY_READ, NOTIFICATION_SHOW, PLAYER_CONTROL,
        PLAYER_READ, STORAGE_PLUGIN, UI_PANEL,
    },
};
use base64::Engine as _;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
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
const STATE_VERSION: u32 = 2;
const EVENT_PLUGIN_NOTIFICATION: &str = "plugin/notification";

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
    pub icon: Option<String>,
    pub icon_data_url: Option<String>,
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
    declared_permissions: Vec<String>,
    granted_permissions: Vec<String>,
}

impl PluginAccess {
    fn from_plugin(plugin: &InstalledPlugin) -> Self {
        Self {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
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
            if !manifest.backend_path(&temporary).is_file()
                || !manifest.ui_path(&temporary).is_file()
            {
                return Err("plugin backend or UI entry is missing".into());
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
            crate::plugin_permissions::validate_permissions(&granted)?;
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
        if !access_permission(&access, UI_PANEL) {
            return permission_error(UI_PANEL);
        }
        let request = json!({ "method": method, "args": args }).to_string();
        let wasm_context = WasmHostContext { access, host };
        match call_wasm(plugin, request.as_bytes(), wasm_context) {
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
            if let Err(error) = call_wasm(plugin, request.as_bytes(), wasm_context) {
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
        let path = format!("/{}/{}", plugin.manifest.id, plugin.manifest.ui);
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
        crate::plugin_manifest::validate_id(id)?;
        crate::plugin_manifest::validate_relative_file(&relative, "resource")?;
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
        if (!plugin.state.enabled || plugin.state.faulted || !plugin_permission(plugin, UI_PANEL)) && !is_icon {
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
    host_context: WasmHostContext,
) -> Result<Value, String> {
    let wasm = plugin
        .wasm
        .as_mut()
        .ok_or_else(|| "plugin backend is not running".to_string())?;
    wasm.store
        .set_fuel(WASM_FUEL)
        .map_err(|error| error.to_string())?;
    wasm.store.data_mut().host_context = Some(host_context);

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
        "library.list" => host
            .media
            .call("media_library_tracks", args)
            .map_err(|error| error.to_string()),
        "notification.show" => notification_show(access, args, &host.app),
        "storage.get" | "storage.set" | "storage.remove" => {
            storage_call(&host.paths, &access.id, method, args)
        }
        _ => Err(format!("unsupported plugin method: {method}")),
    }
}

fn player_control_call(
    method: &str,
    args: Value,
    host: &HostApiContext,
) -> Result<Value, String> {
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

fn notification_show(
    access: &PluginAccess,
    args: Value,
    app: &AppHandle,
) -> Result<Value, String> {
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
        icon: plugin.manifest.icon.clone(),
        icon_data_url: plugin_icon_data_url(plugin),
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

fn plugin_icon_data_url(plugin: &InstalledPlugin) -> Option<String> {
    let relative = plugin.manifest.icon.as_deref()?;
    let path = plugin.root.join(relative);
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let mime = mime_for(Path::new(relative));
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
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
pub fn plugin_list(manager: State<'_, PluginManager>) -> Result<Vec<PluginInfo>, String> {
    manager.list()
}

#[tauri::command]
pub fn plugin_pick_package(manager: State<'_, PluginManager>) -> Option<String> {
    manager.pick_package()
}

#[tauri::command]
pub fn plugin_install(
    manager: State<'_, PluginManager>,
    path: String,
) -> Result<PluginInfo, String> {
    manager.install(path)
}

#[tauri::command]
pub fn plugin_uninstall(manager: State<'_, PluginManager>, id: String) -> Result<Value, String> {
    manager.uninstall(id)
}

#[tauri::command]
pub fn plugin_enable(manager: State<'_, PluginManager>, id: String) -> Result<PluginInfo, String> {
    manager.set_enabled(id, true)
}

#[tauri::command]
pub fn plugin_disable(manager: State<'_, PluginManager>, id: String) -> Result<PluginInfo, String> {
    manager.set_enabled(id, false)
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
