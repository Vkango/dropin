use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use bass_rs::{
    raw, ActiveState, BassEngine, BassEngineOptions, BassError, BassFxEffect, Channel,
    ChannelKind, DspCallback, DspInfo, Effect, EffectKind, InitOptions, OutputBackend,
    Plugin, RemoteProgress, SourceOptions, SyncEvent, SyncKind, TagKind, TempoChannel,
    UrlOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

const EVENT_DOWNLOAD: &str = "bass/download";
const EVENT_SYNC: &str = "bass/sync";
const EVENT_DSP: &str = "bass/dsp";
const EVENT_STATE: &str = "bass/channel-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeError {
    pub kind: String,
    pub operation: String,
    pub message: String,
    pub debug: String,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.operation, self.message)
    }
}

impl std::error::Error for BridgeError {}

fn bridge_error(operation: impl Into<String>, message: impl Into<String>) -> BridgeError {
    let operation = operation.into();
    let message = message.into();
    BridgeError {
        kind: "bridge".into(),
        operation,
        debug: message.clone(),
        message,
    }
}

fn bass_error(operation: &str, error: BassError) -> BridgeError {
    let kind = match &error {
        BassError::LibraryLoad { .. } => "libraryLoad",
        BassError::MissingSymbol { .. } => "missingSymbol",
        BassError::Api { .. } => "api",
        BassError::VersionMismatch { .. } => "versionMismatch",
        BassError::FxUnavailable => "fxUnavailable",
        BassError::AddonUnavailable { .. } => "addonUnavailable",
        BassError::InvalidInput { .. } => "invalidInput",
        BassError::Unsupported { .. } => "unsupported",
        BassError::CallbackPanicked => "callbackPanicked",
    };
    BridgeError {
        kind: kind.into(),
        operation: operation.into(),
        message: error.to_string(),
        debug: format!("{error:?}"),
    }
}

type Reply = mpsc::Sender<Result<Value, BridgeError>>;

struct Request {
    operation: String,
    args: Value,
    reply: Reply,
}

pub struct BassService {
    sender: mpsc::Sender<Request>,
}

impl BassService {
    pub fn new(app: AppHandle) -> Self {
        let default_dirs = default_dll_directories(&app);
        let (sender, receiver) = mpsc::channel::<Request>();
        thread::Builder::new()
            .name("bass-engine".into())
            .spawn(move || {
                let mut runtime = BassRuntime::new(app, default_dirs);
                while let Ok(request) = receiver.recv() {
                    let result = runtime.dispatch(&request.operation, request.args);
                    let _ = request.reply.send(result);
                }
                runtime.clear_handles();
            })
            .expect("failed to start BASS worker thread");
        Self { sender }
    }

    fn call(&self, operation: String, args: Value) -> Result<Value, BridgeError> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(Request {
                operation,
                args,
                reply,
            })
            .map_err(|_| bridge_error("bass_call", "BASS worker thread is not running"))?;
        receiver
            .recv()
            .map_err(|_| bridge_error("bass_call", "BASS worker dropped the response"))?
    }
}

#[tauri::command]
pub fn bass_call(
    service: State<'_, BassService>,
    operation: String,
    args: Value,
) -> Result<Value, BridgeError> {
    service.call(operation, args)
}

fn default_dll_directories(app: &AppHandle) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(path) = app.path().resource_dir() {
        result.push(path.join("bass").join("x64"));
    }
    if let Ok(path) = std::env::current_exe() {
        if let Some(parent) = path.parent() {
            result.push(parent.join("bass").join("x64"));
        }
    }
    result.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("bass")
            .join("x64"),
    );
    result
}

struct BassRuntime {
    app: AppHandle,
    default_dirs: Vec<PathBuf>,
    engine: Option<BassEngine>,
    channels: HashMap<u64, ChannelObject>,
    plugins: HashMap<u64, Plugin>,
    effects: HashMap<u64, EffectRecord>,
    syncs: HashMap<u64, Registration<bass_rs::SyncRegistration>>,
    dsps: HashMap<u64, Registration<bass_rs::DspRegistration>>,
    next_id: u64,
}

enum ChannelObject {
    Plain(Channel),
    Tempo(TempoChannel),
    Reverse(bass_rs::ReverseChannel),
}

struct EffectRecord {
    effect: Effect,
    channel_id: u64,
    kind: EffectKind,
}

#[allow(dead_code)]
struct Registration<T> {
    channel_id: u64,
    registration: T,
}

impl BassRuntime {
    fn new(app: AppHandle, default_dirs: Vec<PathBuf>) -> Self {
        Self {
            app,
            default_dirs,
            engine: None,
            channels: HashMap::new(),
            plugins: HashMap::new(),
            effects: HashMap::new(),
            syncs: HashMap::new(),
            dsps: HashMap::new(),
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn clear_handles(&mut self) {
        self.dsps.clear();
        self.syncs.clear();
        self.effects.clear();
        self.plugins.clear();
        self.channels.clear();
    }

    fn engine(&self, operation: &str) -> Result<&BassEngine, BridgeError> {
        self.engine
            .as_ref()
            .ok_or_else(|| bridge_error(operation, "BASS is not loaded"))
    }

    fn dispatch(&mut self, operation: &str, args: Value) -> Result<Value, BridgeError> {
        match operation {
            "bass_load" => self.load(args),
            "bass_load_fx" => self.load_fx(args),
            "bass_unload" => self.unload(),
            "bass_status" => self.status(),
            "bass_devices" => self.devices(),
            "bass_device_info" => self.device_info(args),
            "bass_initialize" => self.initialize(args),
            "bass_free" => self.free(),
            "bass_output_info" => self.output_info(),
            "bass_set_config" => self.set_config(args),
            "bass_get_config" => self.get_config(args),
            "bass_set_global_volume" => self.set_global_volume(args),
            "bass_get_global_volume" => self.get_global_volume(),
            "bass_cpu_usage" => Ok(json!({ "cpu": self.engine("bass_cpu_usage")?.cpu_usage() })),
            "bass_start" => self.simple_engine_call(operation, |e| e.start()),
            "bass_stop" => self.simple_engine_call(operation, |e| e.stop()),
            "bass_pause" => self.simple_engine_call(operation, |e| e.pause()),
            "bass_is_started" => Ok(json!({ "started": self.engine(operation)?.is_started() })),
            "bass_load_file" => self.load_file(args),
            "bass_pick_file" => self.pick_file(),
            "bass_load_url" => self.load_url(args),
            "bass_load_plugin" => self.load_plugin(args),
            "bass_load_plugins" => self.load_plugins(args),
            "bass_plugin_info" => self.plugin_info(args),
            "bass_plugin_enable" => self.plugin_enable(args),
            "bass_plugin_close" => self.plugin_close(args),
            "bass_channel_close" => self.channel_close(args),
            "bass_channel_snapshot" => self.channel_snapshot(args),
            "bass_channel_play" => self.channel_play(args),
            "bass_channel_pause" => self.channel_pause(args),
            "bass_channel_stop" => self.channel_stop(args),
            "bass_channel_set_attribute" => self.channel_set_attribute(args),
            "bass_channel_get_attribute" => self.channel_get_attribute(args),
            "bass_channel_set_volume" => self.channel_set_volume(args),
            "bass_channel_set_pan" => self.channel_set_pan(args),
            "bass_channel_set_frequency" => self.channel_set_frequency(args),
            "bass_channel_seek" => self.channel_seek(args),
            "bass_channel_seek_bytes" => self.channel_seek_bytes(args),
            "bass_channel_set_device" => self.channel_set_device(args),
            "bass_channel_info" => self.channel_info(args),
            "bass_channel_level" => self.channel_level(args),
            "bass_channel_level_ex" => self.channel_level_ex(args),
            "bass_channel_read_data" => self.channel_read_data(args),
            "bass_channel_read_float_data" => self.channel_read_float_data(args),
            "bass_channel_tags" => self.channel_tags(args),
            "bass_channel_remote_progress" => self.channel_remote_progress(args),
            "bass_channel_add_sync" => self.channel_add_sync(args),
            "bass_channel_remove_sync" => self.channel_remove_sync(args),
            "bass_channel_add_dsp" => self.channel_add_dsp(args),
            "bass_channel_remove_dsp" => self.channel_remove_dsp(args),
            "bass_channel_to_tempo" => self.channel_to_tempo(args),
            "bass_channel_to_reverse" => self.channel_to_reverse(args),
            "bass_tempo_get" => self.tempo_get(args),
            "bass_tempo_set" => self.tempo_set(args),
            "bass_reverse_get" => self.reverse_get(args),
            "bass_reverse_set" => self.reverse_set(args),
            "bass_add_effect" => self.add_effect(args),
            "bass_add_loudness" => self.add_loudness(args),
            "bass_effect_close" => self.effect_close(args),
            "bass_effect_set_parameters" => self.effect_set_parameters(args),
            "bass_effect_get_parameters" => self.effect_get_parameters(args),
            "bass_effect_set_priority" => self.effect_set_priority(args),
            "bass_effect_set_bypass" => self.effect_set_bypass(args),
            "bass_effect_reset" => self.effect_reset(args),
            "bass_raw_catalog" => Ok(raw_catalog()),
            "bass_midi_load" => self.midi_load(args),
            "bass_midi_load_from_directory" => self.midi_load_from_directory(args),
            "bass_midi_set_max_polyphony" => self.midi_set_max_polyphony(args),
            _ => Err(bridge_error(operation, "unknown BASS operation")),
        }
    }

    fn load(&mut self, args: Value) -> Result<Value, BridgeError> {
        let bass_path = optional_string(&args, "bassPath")?;
        let fx_path = optional_string(&args, "fxPath")?;
        let dll_dir = optional_string(&args, "dllDir")?.map(PathBuf::from);
        let require_fx = args
            .get("requireFx")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let directory = dll_dir.or_else(|| {
            self.default_dirs
                .iter()
                .find(|path| path.join("bass.dll").is_file())
                .cloned()
        });
        let options = BassEngineOptions {
            fx_path: fx_path.map(PathBuf::from).or_else(|| {
                directory
                    .as_ref()
                    .map(|path| path.join("bass_fx.dll"))
                    .filter(|path| path.is_file())
            }),
            require_fx,
        };
        let engine = if let Some(path) = bass_path {
            BassEngine::load_with_options(path, options)
        } else if let Some(directory) = directory {
            BassEngine::load_from_directory_with_options(directory, options)
        } else {
            return Err(bridge_error(
                "bass_load",
                "could not locate bass.dll; provide dllDir or bassPath",
            ));
        }
        .map_err(|error| bass_error("bass_load", error))?;

        self.clear_handles();
        self.engine = Some(engine);
        self.status()
    }

    fn unload(&mut self) -> Result<Value, BridgeError> {
        if let Some(engine) = self.engine.take() {
            self.clear_handles();
            if engine.is_initialized() {
                let _ = engine.free();
            }
        } else {
            self.clear_handles();
        }
        Ok(json!({ "loaded": false }))
    }

    fn load_fx(&self, args: Value) -> Result<Value, BridgeError> {
        let path = required_string(&args, "path", "bass_load_fx")?;
        self.engine("bass_load_fx")?
            .load_fx(path)
            .map_err(|error| bass_error("bass_load_fx", error))?;
        Ok(self.status()?)
    }

    fn status(&self) -> Result<Value, BridgeError> {
        let Some(engine) = self.engine.as_ref() else {
            return Ok(json!({ "loaded": false, "fxLoaded": false }));
        };
        Ok(json!({
            "loaded": true,
            "initialized": engine.is_initialized(),
            "started": engine.is_started(),
            "bassVersion": engine.bass_version(),
            "fxLoaded": engine.has_fx(),
            "fxVersion": engine.fx_version(),
            "channels": self.channels.keys().copied().collect::<Vec<_>>(),
            "plugins": self.plugins.keys().copied().collect::<Vec<_>>(),
            "effects": self.effects.keys().copied().collect::<Vec<_>>(),
        }))
    }

    fn devices(&self) -> Result<Value, BridgeError> {
        let devices = self
            .engine("bass_devices")?
            .devices()
            .map_err(|error| bass_error("bass_devices", error))?;
        Ok(json!(devices
            .into_iter()
            .map(device_json)
            .collect::<Vec<_>>()))
    }

    fn device_info(&self, args: Value) -> Result<Value, BridgeError> {
        let index = required_u32(&args, "index", "bass_device_info")?;
        let device = self
            .engine("bass_device_info")?
            .device_info(index)
            .map_err(|error| bass_error("bass_device_info", error))?;
        Ok(device.map(device_json).unwrap_or(Value::Null))
    }

    fn initialize(&self, args: Value) -> Result<Value, BridgeError> {
        let input: InitInput = parse_args(args, "bass_initialize")?;
        let options = InitOptions {
            device: input.device,
            sample_rate: input.sample_rate,
            backend: parse_backend(&input.backend)?,
            mono: input.mono,
            exclusive: input.exclusive,
            force_frequency: input.force_frequency,
            float_processing: input.float_processing,
        };
        self.engine("bass_initialize")?
            .initialize(options)
            .map_err(|error| bass_error("bass_initialize", error))?;
        Ok(json!({ "initialized": true }))
    }

    fn free(&mut self) -> Result<Value, BridgeError> {
        self.clear_handles();
        self.engine("bass_free")?
            .free()
            .map_err(|error| bass_error("bass_free", error))?;
        Ok(json!({ "initialized": false }))
    }

    fn output_info(&self) -> Result<Value, BridgeError> {
        let info = self
            .engine("bass_output_info")?
            .output_info()
            .map_err(|error| bass_error("bass_output_info", error))?;
        Ok(json!({
            "flags": info.flags,
            "minBufferMs": info.min_buffer_ms,
            "latencyMs": info.latency_ms,
            "sampleRate": info.sample_rate,
            "speakers": info.speakers,
            "directSoundVersion": info.direct_sound_version,
        }))
    }

    fn set_config(&self, args: Value) -> Result<Value, BridgeError> {
        let option = required_u32(&args, "option", "bass_set_config")?;
        let value = required_u32(&args, "value", "bass_set_config")?;
        self.engine("bass_set_config")?
            .set_config(option, value)
            .map_err(|error| bass_error("bass_set_config", error))?;
        Ok(json!({ "option": option, "value": value }))
    }

    fn get_config(&self, args: Value) -> Result<Value, BridgeError> {
        let option = required_u32(&args, "option", "bass_get_config")?;
        Ok(json!({ "option": option, "value": self.engine("bass_get_config")?.get_config(option) }))
    }

    fn set_global_volume(&self, args: Value) -> Result<Value, BridgeError> {
        let volume = required_f32(&args, "volume", "bass_set_global_volume")?;
        self.engine("bass_set_global_volume")?
            .set_global_volume(volume)
            .map_err(|error| bass_error("bass_set_global_volume", error))?;
        Ok(json!({ "volume": volume }))
    }

    fn get_global_volume(&self) -> Result<Value, BridgeError> {
        Ok(json!({ "volume": self.engine("bass_get_global_volume")?.global_volume() }))
    }

    fn simple_engine_call<F>(&self, operation: &str, call: F) -> Result<Value, BridgeError>
    where
        F: FnOnce(&BassEngine) -> bass_rs::Result<()>,
    {
        call(self.engine(operation)?).map_err(|error| bass_error(operation, error))?;
        Ok(json!({ "ok": true }))
    }

    fn load_file(&mut self, args: Value) -> Result<Value, BridgeError> {
        let path = required_string(&args, "path", "bass_load_file")?;
        let options = source_options(args.get("options").cloned().unwrap_or_else(|| json!({})))?;
        let channel = self
            .engine("bass_load_file")?
            .load_file(&path, options)
            .map_err(|error| bass_error("bass_load_file", error))?;
        let id = self.next_id();
        let object = ChannelObject::Plain(channel);
        let result = channel_json(id, &object);
        self.channels.insert(id, object);
        Ok(result)
    }

    fn pick_file(&self) -> Result<Value, BridgeError> {
        let path = rfd::FileDialog::new()
            .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "m4a", "aac", "wma", "mod", "xm", "it", "s3m"])
            .pick_file();
        Ok(json!({ "path": path.map(|value| value.to_string_lossy().into_owned()) }))
    }

    fn load_url(&mut self, args: Value) -> Result<Value, BridgeError> {
        let url = required_string(&args, "url", "bass_load_url")?;
        let input: UrlInput = parse_args(normalize_object(args.get("options").cloned().unwrap_or_else(|| json!({}))), "bass_load_url")?;
        let id = self.next_id();
        let app = self.app.clone();
        let callback: bass_rs::DownloadCallback = Box::new(move |event| {
            let payload = match event {
                bass_rs::DownloadEvent::Data { length } => json!({ "channelId": id, "kind": "data", "length": length }),
                bass_rs::DownloadEvent::Status(status) => json!({ "channelId": id, "kind": "status", "status": status }),
                bass_rs::DownloadEvent::Finished => json!({ "channelId": id, "kind": "finished" }),
            };
            let _ = app.emit(EVENT_DOWNLOAD, payload);
        });
        let options = UrlOptions {
            offset: input.offset,
            float: input.float,
            flags: input.flags,
            callback: Some(callback),
        };
        let channel = self
            .engine("bass_load_url")?
            .load_url(&url, options)
            .map_err(|error| bass_error("bass_load_url", error))?;
        let object = ChannelObject::Plain(channel);
        let response = channel_json(id, &object);
        self.channels.insert(id, object);
        Ok(response)
    }

    fn load_plugin(&mut self, args: Value) -> Result<Value, BridgeError> {
        let path = required_string(&args, "path", "bass_load_plugin")?;
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let plugin = self
            .engine("bass_load_plugin")?
            .load_plugin_with_flags(&path, flags)
            .map_err(|error| bass_error("bass_load_plugin", error))?;
        let id = self.next_id();
        let response = plugin_json(id, &plugin);
        self.plugins.insert(id, plugin);
        Ok(response)
    }

    fn load_plugins(&mut self, args: Value) -> Result<Value, BridgeError> {
        let paths = args
            .get("paths")
            .and_then(Value::as_array)
            .ok_or_else(|| bridge_error("bass_load_plugins", "paths must be an array"))?
            .iter()
            .map(|value| value.as_str().map(ToOwned::to_owned).ok_or_else(|| bridge_error("bass_load_plugins", "paths must contain strings")))
            .collect::<Result<Vec<_>, _>>()?;
        let plugins = self
            .engine("bass_load_plugins")?
            .load_plugins(paths)
            .map_err(|error| bass_error("bass_load_plugins", error))?;
        let mut result = Vec::new();
        for plugin in plugins {
            let id = self.next_id();
            result.push(plugin_json(id, &plugin));
            self.plugins.insert(id, plugin);
        }
        Ok(Value::Array(result))
    }

    fn plugin_info(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "pluginId", "bass_plugin_info")?;
        let plugin = self.plugins.get(&id).ok_or_else(|| missing_handle("plugin", id))?;
        Ok(plugin_json(id, plugin))
    }

    fn plugin_enable(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "pluginId", "bass_plugin_enable")?;
        let enabled = args.get("enabled").and_then(Value::as_bool).unwrap_or(true);
        self.plugins
            .get(&id)
            .ok_or_else(|| missing_handle("plugin", id))?
            .enable(enabled)
            .map_err(|error| bass_error("bass_plugin_enable", error))?;
        Ok(json!({ "pluginId": id, "enabled": enabled }))
    }

    fn plugin_close(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "pluginId", "bass_plugin_close")?;
        self.plugins
            .remove(&id)
            .ok_or_else(|| missing_handle("plugin", id))?;
        Ok(json!({ "pluginId": id, "closed": true }))
    }

    fn channel_close(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_close")?;
        let sync_ids = self
            .syncs
            .iter()
            .filter_map(|(registration_id, registration)| (registration.channel_id == id).then_some(*registration_id))
            .collect::<Vec<_>>();
        for registration_id in sync_ids {
            self.syncs.remove(&registration_id);
        }
        let dsp_ids = self
            .dsps
            .iter()
            .filter_map(|(registration_id, registration)| (registration.channel_id == id).then_some(*registration_id))
            .collect::<Vec<_>>();
        for registration_id in dsp_ids {
            self.dsps.remove(&registration_id);
        }
        let effect_ids = self
            .effects
            .iter()
            .filter_map(|(effect_id, effect)| (effect.channel_id == id).then_some(*effect_id))
            .collect::<Vec<_>>();
        for effect_id in effect_ids {
            self.effects.remove(&effect_id);
        }
        self.channels
            .remove(&id)
            .ok_or_else(|| missing_handle("channel", id))?;
        let _ = self.app.emit(EVENT_STATE, json!({ "channelId": id, "state": "closed" }));
        Ok(json!({ "channelId": id, "closed": true }))
    }

    fn channel_snapshot(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_snapshot")?;
        let object = self.channel(id, "bass_channel_snapshot")?;
        let channel = object.as_channel();
        let position = channel.position().map_err(|error| bass_error("bass_channel_snapshot", error))?;
        let length = channel.length().map_err(|error| bass_error("bass_channel_snapshot", error))?;
        Ok(json!({
            "channelId": id,
            "rawHandle": channel.raw_handle(),
            "kind": channel_kind_name(channel.kind()),
            "state": active_state_name(channel.active_state()),
            "positionSeconds": position.as_secs_f64(),
            "lengthSeconds": length.map(|value| value.as_secs_f64()),
            "volume": channel.volume().ok(),
            "pan": channel.pan().ok(),
            "frequency": channel.frequency().ok(),
            "device": channel.device(),
        }))
    }

    fn channel_play(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_play")?;
        let restart = args.get("restart").and_then(Value::as_bool).unwrap_or(false);
        self.channel(id, "bass_channel_play")?
            .as_channel()
            .play(restart)
            .map_err(|error| bass_error("bass_channel_play", error))?;
        self.emit_state(id);
        Ok(json!({ "channelId": id, "state": "playing" }))
    }

    fn channel_pause(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_pause")?;
        self.channel(id, "bass_channel_pause")?
            .as_channel()
            .pause()
            .map_err(|error| bass_error("bass_channel_pause", error))?;
        self.emit_state(id);
        Ok(json!({ "channelId": id, "state": "paused" }))
    }

    fn channel_stop(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_stop")?;
        self.channel(id, "bass_channel_stop")?
            .as_channel()
            .stop()
            .map_err(|error| bass_error("bass_channel_stop", error))?;
        self.emit_state(id);
        Ok(json!({ "channelId": id, "state": "stopped" }))
    }

    fn channel_set_attribute(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_set_attribute")?;
        let attribute = required_u32(&args, "attribute", "bass_channel_set_attribute")?;
        let value = required_f32(&args, "value", "bass_channel_set_attribute")?;
        self.channel(id, "bass_channel_set_attribute")?
            .as_channel()
            .set_attribute(attribute, value)
            .map_err(|error| bass_error("bass_channel_set_attribute", error))?;
        Ok(json!({ "channelId": id, "attribute": attribute, "value": value }))
    }

    fn channel_get_attribute(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_get_attribute")?;
        let attribute = required_u32(&args, "attribute", "bass_channel_get_attribute")?;
        let value = self
            .channel(id, "bass_channel_get_attribute")?
            .as_channel()
            .attribute(attribute)
            .map_err(|error| bass_error("bass_channel_get_attribute", error))?;
        Ok(json!({ "channelId": id, "attribute": attribute, "value": value }))
    }

    fn channel_set_volume(&self, args: Value) -> Result<Value, BridgeError> {
        self.channel_attribute_shortcut(args, "volume", |channel, value| channel.set_volume(value))
    }

    fn channel_set_pan(&self, args: Value) -> Result<Value, BridgeError> {
        self.channel_attribute_shortcut(args, "pan", |channel, value| channel.set_pan(value))
    }

    fn channel_set_frequency(&self, args: Value) -> Result<Value, BridgeError> {
        self.channel_attribute_shortcut(args, "frequency", |channel, value| channel.set_frequency(value))
    }

    fn channel_attribute_shortcut<F>(&self, args: Value, field: &str, call: F) -> Result<Value, BridgeError>
    where
        F: FnOnce(&Channel, f32) -> bass_rs::Result<()>,
    {
        let id = required_id(&args, "channelId", "bass_channel_set_attribute")?;
        let value = required_f32(&args, field, "bass_channel_set_attribute")?;
        call(self.channel(id, "bass_channel_set_attribute")?.as_channel(), value)
            .map_err(|error| bass_error("bass_channel_set_attribute", error))?;
        Ok(json!({ "channelId": id, field: value }))
    }

    fn channel_seek(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_seek")?;
        let seconds = required_f64(&args, "seconds", "bass_channel_seek")?;
        self.channel(id, "bass_channel_seek")?
            .as_channel()
            .seek(Duration::from_secs_f64(seconds.max(0.0)))
            .map_err(|error| bass_error("bass_channel_seek", error))?;
        Ok(json!({ "channelId": id, "seconds": seconds }))
    }

    fn channel_seek_bytes(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_seek_bytes")?;
        let position = required_u64(&args, "position", "bass_channel_seek_bytes")?;
        let mode = optional_u32(&args, "mode")?.unwrap_or(raw::BASS_POS_BYTE);
        self.channel(id, "bass_channel_seek_bytes")?
            .as_channel()
            .seek_bytes(position, mode)
            .map_err(|error| bass_error("bass_channel_seek_bytes", error))?;
        Ok(json!({ "channelId": id, "position": position, "mode": mode }))
    }

    fn channel_set_device(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_set_device")?;
        let device = required_u32(&args, "device", "bass_channel_set_device")?;
        self.channel(id, "bass_channel_set_device")?
            .as_channel()
            .set_device(device)
            .map_err(|error| bass_error("bass_channel_set_device", error))?;
        Ok(json!({ "channelId": id, "device": device }))
    }

    fn channel_info(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_info")?;
        let info = self
            .channel(id, "bass_channel_info")?
            .as_channel()
            .info()
            .map_err(|error| bass_error("bass_channel_info", error))?;
        Ok(json!({
            "frequency": info.frequency,
            "channels": info.channels,
            "flags": info.flags,
            "channelType": info.channel_type,
            "originalResolution": info.original_resolution,
            "plugin": info.plugin,
            "filename": info.filename,
        }))
    }

    fn channel_level(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_level")?;
        Ok(json!({ "level": self.channel(id, "bass_channel_level")?.as_channel().get_level() }))
    }

    fn channel_level_ex(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_level_ex")?;
        let seconds = optional_f32(&args, "seconds")?.unwrap_or(0.05);
        let flags = optional_u32(&args, "flags")?.unwrap_or(raw::BASS_LEVEL_RMS);
        let level = self
            .channel(id, "bass_channel_level_ex")?
            .as_channel()
            .get_level_ex(seconds, flags)
            .map_err(|error| bass_error("bass_channel_level_ex", error))?;
        Ok(json!({ "level": level, "seconds": seconds, "flags": flags }))
    }

    fn channel_read_data(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_read_data")?;
        let bytes = required_usize(&args, "bytes", "bass_channel_read_data")?;
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let data = self
            .channel(id, "bass_channel_read_data")?
            .as_channel()
            .read_data(bytes, flags)
            .map_err(|error| bass_error("bass_channel_read_data", error))?;
        Ok(json!({ "bytes": data }))
    }

    fn channel_read_float_data(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_read_float_data")?;
        let samples = required_usize(&args, "samples", "bass_channel_read_float_data")?;
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let data = self
            .channel(id, "bass_channel_read_float_data")?
            .as_channel()
            .read_float_data(samples, flags)
            .map_err(|error| bass_error("bass_channel_read_float_data", error))?;
        Ok(json!({ "samples": data }))
    }

    fn channel_tags(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_tags")?;
        let tag = parse_tag_kind(required_string(&args, "tag", "bass_channel_tags")?.as_str())?;
        Ok(json!({ "tags": self.channel(id, "bass_channel_tags")?.as_channel().tags(tag) }))
    }

    fn channel_remote_progress(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_remote_progress")?;
        let progress = self
            .channel(id, "bass_channel_remote_progress")?
            .as_channel()
            .remote_progress()
            .map_err(|error| bass_error("bass_channel_remote_progress", error))?;
        Ok(remote_progress_json(progress))
    }

    fn channel_add_sync(&mut self, args: Value) -> Result<Value, BridgeError> {
        let channel_id = required_id(&args, "channelId", "bass_channel_add_sync")?;
        let kind = parse_sync_kind(&args)?;
        let registration_id = self.next_id();
        let app = self.app.clone();
        let callback: bass_rs::SyncCallback = Box::new(move |event: SyncEvent| {
            let _ = app.emit(
                EVENT_SYNC,
                json!({
                    "registrationId": registration_id,
                    "channelId": channel_id,
                    "syncHandle": event.sync_handle,
                    "channel": event.channel,
                    "data": event.data,
                }),
            );
        });
        let registration = self
            .channel(channel_id, "bass_channel_add_sync")?
            .as_channel()
            .set_sync(kind, callback)
            .map_err(|error| bass_error("bass_channel_add_sync", error))?;
        self.syncs.insert(registration_id, Registration { channel_id, registration });
        Ok(json!({ "registrationId": registration_id }))
    }

    fn channel_remove_sync(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "registrationId", "bass_channel_remove_sync")?;
        self.syncs
            .remove(&id)
            .ok_or_else(|| missing_handle("sync registration", id))?;
        Ok(json!({ "registrationId": id, "removed": true }))
    }

    fn channel_add_dsp(&mut self, args: Value) -> Result<Value, BridgeError> {
        let channel_id = required_id(&args, "channelId", "bass_channel_add_dsp")?;
        let priority = optional_i32(&args, "priority")?.unwrap_or(0);
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let mode = optional_string(&args, "mode")?.unwrap_or_else(|| "passthrough".into());
        let callback_mode = mode.clone();
        let registration_id = self.next_id();
        let app = self.app.clone();
        let last_event = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));
        let last_event_for_callback = last_event.clone();
        let callback: DspCallback = Box::new(move |buffer, info: DspInfo| {
            if callback_mode == "mute" {
                buffer.fill(0);
            }
            if let Ok(mut last) = last_event_for_callback.lock() {
                if last.elapsed() >= Duration::from_millis(100) {
                    *last = Instant::now();
                    let _ = app.emit(
                        EVENT_DSP,
                        json!({
                            "registrationId": registration_id,
                            "channelId": channel_id,
                            "dspHandle": info.dsp_handle,
                            "channel": info.channel,
                            "byteLength": info.byte_length,
                            "mode": callback_mode.clone(),
                        }),
                    );
                }
            }
        });
        let registration = self
            .channel(channel_id, "bass_channel_add_dsp")?
            .as_channel()
            .add_dsp_ex(callback, priority, flags)
            .map_err(|error| bass_error("bass_channel_add_dsp", error))?;
        self.dsps.insert(registration_id, Registration { channel_id, registration });
        Ok(json!({ "registrationId": registration_id, "mode": mode }))
    }

    fn channel_remove_dsp(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "registrationId", "bass_channel_remove_dsp")?;
        self.dsps
            .remove(&id)
            .ok_or_else(|| missing_handle("dsp registration", id))?;
        Ok(json!({ "registrationId": id, "removed": true }))
    }

    fn channel_to_tempo(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_to_tempo")?;
        self.ensure_no_dependents(id, "bass_channel_to_tempo")?;
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let object = self
            .channels
            .remove(&id)
            .ok_or_else(|| missing_handle("channel", id))?;
        let ChannelObject::Plain(channel) = object else {
            return Err(bridge_error("bass_channel_to_tempo", "only a plain channel can become tempo"));
        };
        let tempo = channel
            .into_tempo(flags)
            .map_err(|error| bass_error("bass_channel_to_tempo", error))?;
        let object = ChannelObject::Tempo(tempo);
        let response = channel_json(id, &object);
        self.channels.insert(id, object);
        Ok(response)
    }

    fn channel_to_reverse(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_channel_to_reverse")?;
        self.ensure_no_dependents(id, "bass_channel_to_reverse")?;
        let flags = optional_u32(&args, "flags")?.unwrap_or(0);
        let dec_block = optional_f32(&args, "decBlock")?.unwrap_or(2.0);
        let object = self
            .channels
            .remove(&id)
            .ok_or_else(|| missing_handle("channel", id))?;
        let ChannelObject::Plain(channel) = object else {
            return Err(bridge_error("bass_channel_to_reverse", "only a plain channel can become reverse"));
        };
        let reverse = channel
            .into_reverse(dec_block, flags)
            .map_err(|error| bass_error("bass_channel_to_reverse", error))?;
        let object = ChannelObject::Reverse(reverse);
        let response = channel_json(id, &object);
        self.channels.insert(id, object);
        Ok(response)
    }

    fn tempo_get(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_tempo_get")?;
        let ChannelObject::Tempo(channel) = self.channel(id, "bass_tempo_get")? else {
            return Err(bridge_error("bass_tempo_get", "channel is not a tempo channel"));
        };
        Ok(json!({ "tempo": channel.tempo().map_err(|error| bass_error("bass_tempo_get", error))?, "pitch": channel.pitch().map_err(|error| bass_error("bass_tempo_get", error))?, "frequency": channel.tempo_frequency().map_err(|error| bass_error("bass_tempo_get", error))?, "rateRatio": channel.rate_ratio(), "sourceHandle": channel.source_handle() }))
    }

    fn tempo_set(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_tempo_set")?;
        let field = required_string(&args, "field", "bass_tempo_set")?;
        let value = required_f32(&args, "value", "bass_tempo_set")?;
        let ChannelObject::Tempo(channel) = self.channel(id, "bass_tempo_set")? else {
            return Err(bridge_error("bass_tempo_set", "channel is not a tempo channel"));
        };
        match field.as_str() {
            "tempo" => channel.set_tempo(value),
            "pitch" => channel.set_pitch(value),
            "frequency" => channel.set_tempo_frequency(value),
            _ => return Err(bridge_error("bass_tempo_set", "field must be tempo, pitch, or frequency")),
        }
        .map_err(|error| bass_error("bass_tempo_set", error))?;
        Ok(json!({ "channelId": id, "field": field, "value": value }))
    }

    fn reverse_get(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_reverse_get")?;
        let ChannelObject::Reverse(channel) = self.channel(id, "bass_reverse_get")? else {
            return Err(bridge_error("bass_reverse_get", "channel is not a reverse channel"));
        };
        Ok(json!({ "direction": channel.direction().map_err(|error| bass_error("bass_reverse_get", error))?, "sourceHandle": channel.source_handle() }))
    }

    fn reverse_set(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "channelId", "bass_reverse_set")?;
        let direction = required_f32(&args, "direction", "bass_reverse_set")?;
        let ChannelObject::Reverse(channel) = self.channel(id, "bass_reverse_set")? else {
            return Err(bridge_error("bass_reverse_set", "channel is not a reverse channel"));
        };
        channel
            .set_direction(direction)
            .map_err(|error| bass_error("bass_reverse_set", error))?;
        Ok(json!({ "channelId": id, "direction": direction }))
    }

    fn add_effect(&mut self, args: Value) -> Result<Value, BridgeError> {
        let channel_id = required_id(&args, "channelId", "bass_add_effect")?;
        let kind = parse_effect_kind(required_string(&args, "kind", "bass_add_effect")?.as_str())?;
        let priority = optional_i32(&args, "priority")?.unwrap_or(0);
        let effect = self
            .channel(channel_id, "bass_add_effect")?
            .as_channel()
            .add_effect(kind, priority)
            .map_err(|error| bass_error("bass_add_effect", error))?;
        let effect_id = self.next_id();
        let response = json!({ "effectId": effect_id, "channelId": channel_id, "kind": effect_kind_name(kind), "rawHandle": effect.raw_handle() });
        self.effects.insert(effect_id, EffectRecord { effect, channel_id, kind });
        Ok(response)
    }

    fn add_loudness(&mut self, args: Value) -> Result<Value, BridgeError> {
        let channel_id = required_id(&args, "channelId", "bass_add_loudness")?;
        let priority = optional_i32(&args, "priority")?.unwrap_or(0);
        let input: LoudnessInput = parse_args(normalize_object(args.get("options").cloned().unwrap_or_else(|| json!({}))), "bass_add_loudness")?;
        let options = bass_rs::LoudnessOptions {
            gain_db: input.gain_db,
            threshold_db: input.threshold_db,
            ratio: input.ratio,
            attack_ms: input.attack_ms,
            release_ms: input.release_ms,
        };
        let chain = self
            .channel(channel_id, "bass_add_loudness")?
            .as_channel()
            .add_loudness(options, priority)
            .map_err(|error| bass_error("bass_add_loudness", error))?;
        let effect_id = self.next_id();
        let kind = EffectKind::BassFx(BassFxEffect::Compressor2);
        let effect = chain.compressor;
        let response = json!({ "effectId": effect_id, "channelId": channel_id, "kind": effect_kind_name(kind), "rawHandle": effect.raw_handle() });
        self.effects.insert(effect_id, EffectRecord { effect, channel_id, kind });
        Ok(response)
    }

    fn effect_close(&mut self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_close")?;
        self.effects
            .remove(&id)
            .ok_or_else(|| missing_handle("effect", id))?;
        Ok(json!({ "effectId": id, "closed": true }))
    }

    fn effect_set_parameters(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_set_parameters")?;
        let record = self.effects.get(&id).ok_or_else(|| missing_handle("effect", id))?;
        let parameters = args.get("parameters").cloned().unwrap_or_default();
        set_effect_parameters(&record.effect, record.kind, &parameters)
            .map_err(|error| bass_error("bass_effect_set_parameters", error))?;
        Ok(json!({ "effectId": id, "kind": effect_kind_name(record.kind), "parameters": parameters }))
    }

    fn effect_get_parameters(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_get_parameters")?;
        let record = self.effects.get(&id).ok_or_else(|| missing_handle("effect", id))?;
        let parameters = get_effect_parameters(&record.effect, record.kind)
            .map_err(|error| bass_error("bass_effect_get_parameters", error))?;
        Ok(json!({ "effectId": id, "kind": effect_kind_name(record.kind), "parameters": parameters }))
    }

    fn effect_set_priority(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_set_priority")?;
        let priority = required_i32(&args, "priority", "bass_effect_set_priority")?;
        self.effects
            .get(&id)
            .ok_or_else(|| missing_handle("effect", id))?
            .effect
            .set_priority(priority)
            .map_err(|error| bass_error("bass_effect_set_priority", error))?;
        Ok(json!({ "effectId": id, "priority": priority }))
    }

    fn effect_set_bypass(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_set_bypass")?;
        let bypass = args.get("bypass").and_then(Value::as_bool).unwrap_or(false);
        self.effects
            .get(&id)
            .ok_or_else(|| missing_handle("effect", id))?
            .effect
            .set_bypass(bypass)
            .map_err(|error| bass_error("bass_effect_set_bypass", error))?;
        Ok(json!({ "effectId": id, "bypass": bypass }))
    }

    fn effect_reset(&self, args: Value) -> Result<Value, BridgeError> {
        let id = required_id(&args, "effectId", "bass_effect_reset")?;
        self.effects
            .get(&id)
            .ok_or_else(|| missing_handle("effect", id))?
            .effect
            .reset()
            .map_err(|error| bass_error("bass_effect_reset", error))?;
        Ok(json!({ "effectId": id, "reset": true }))
    }

    fn midi_load(&self, args: Value) -> Result<Value, BridgeError> {
        let path = required_string(&args, "path", "bass_midi_load")?;
        bass_rs::midi::MidiAddon::load(path)
            .map(|addon| json!({ "path": addon.path }))
            .map_err(|error| bass_error("bass_midi_load", error))
    }

    fn midi_load_from_directory(&self, args: Value) -> Result<Value, BridgeError> {
        let directory = required_string(&args, "directory", "bass_midi_load_from_directory")?;
        bass_rs::midi::MidiAddon::load_from_directory(directory)
            .map(|addon| json!({ "path": addon.path }))
            .map_err(|error| bass_error("bass_midi_load_from_directory", error))
    }

    fn midi_set_max_polyphony(&self, args: Value) -> Result<Value, BridgeError> {
        let path = required_string(&args, "path", "bass_midi_set_max_polyphony")?;
        let max_polyphony = required_u32(&args, "maxPolyphony", "bass_midi_set_max_polyphony")?;
        let addon = bass_rs::midi::MidiAddon::load(path)
            .map_err(|error| bass_error("bass_midi_set_max_polyphony", error))?;
        addon
            .set_max_polyphony(bass_rs::midi::MidiOptions { max_polyphony: Some(max_polyphony) })
            .map(|_| json!({ "maxPolyphony": max_polyphony }))
            .map_err(|error| bass_error("bass_midi_set_max_polyphony", error))
    }

    fn channel(&self, id: u64, operation: &str) -> Result<&ChannelObject, BridgeError> {
        self.channels
            .get(&id)
            .ok_or_else(|| BridgeError { operation: operation.into(), ..missing_handle("channel", id) })
    }

    fn ensure_no_dependents(&self, id: u64, operation: &str) -> Result<(), BridgeError> {
        if self.effects.values().any(|effect| effect.channel_id == id)
            || self.syncs.values().any(|registration| registration.channel_id == id)
            || self.dsps.values().any(|registration| registration.channel_id == id)
        {
            return Err(bridge_error(operation, "remove effects and callbacks before transforming the channel"));
        }
        Ok(())
    }

    fn emit_state(&self, id: u64) {
        if let Some(channel) = self.channels.get(&id) {
            let _ = self.app.emit(
                EVENT_STATE,
                json!({ "channelId": id, "state": active_state_name(channel.as_channel().active_state()) }),
            );
        }
    }
}

impl ChannelObject {
    fn as_channel(&self) -> &Channel {
        match self {
            Self::Plain(channel) => channel,
            Self::Tempo(channel) => channel,
            Self::Reverse(channel) => channel,
        }
    }
}

fn parse_args<T: DeserializeOwned>(args: Value, operation: &str) -> Result<T, BridgeError> {
    serde_json::from_value(args).map_err(|error| bridge_error(operation, error.to_string()))
}

fn required_string(args: &Value, field: &str, operation: &str) -> Result<String, BridgeError> {
    args.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| bridge_error(operation, format!("missing string field: {field}")))
}

fn optional_string(args: &Value, field: &str) -> Result<Option<String>, BridgeError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(bridge_error("bass_call", format!("field {field} must be a string"))),
    }
}

fn required_id(args: &Value, field: &str, operation: &str) -> Result<u64, BridgeError> {
    required_u64(args, field, operation)
}

fn required_u64(args: &Value, field: &str, operation: &str) -> Result<u64, BridgeError> {
    args.get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| bridge_error(operation, format!("missing u64 field: {field}")))
}

fn required_usize(args: &Value, field: &str, operation: &str) -> Result<usize, BridgeError> {
    let value = required_u64(args, field, operation)?;
    usize::try_from(value).map_err(|_| bridge_error(operation, format!("field {field} is too large")))
}

fn optional_u32(args: &Value, field: &str) -> Result<Option<u32>, BridgeError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| bridge_error("bass_call", format!("field {field} must be a u32"))),
    }
}

fn required_u32(args: &Value, field: &str, operation: &str) -> Result<u32, BridgeError> {
    optional_u32(args, field)?.ok_or_else(|| bridge_error(operation, format!("missing u32 field: {field}")))
}

fn optional_i32(args: &Value, field: &str) -> Result<Option<i32>, BridgeError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| bridge_error("bass_call", format!("field {field} must be an i32"))),
    }
}

fn required_i32(args: &Value, field: &str, operation: &str) -> Result<i32, BridgeError> {
    optional_i32(args, field)?.ok_or_else(|| bridge_error(operation, format!("missing i32 field: {field}")))
}

fn optional_f32(args: &Value, field: &str) -> Result<Option<f32>, BridgeError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_f64()
            .map(|value| value as f32)
            .map(Some)
            .ok_or_else(|| bridge_error("bass_call", format!("field {field} must be a number"))),
    }
}

fn required_f32(args: &Value, field: &str, operation: &str) -> Result<f32, BridgeError> {
    optional_f32(args, field)?.ok_or_else(|| bridge_error(operation, format!("missing number field: {field}")))
}

fn required_f64(args: &Value, field: &str, operation: &str) -> Result<f64, BridgeError> {
    args.get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| bridge_error(operation, format!("missing number field: {field}")))
}

fn missing_handle(kind: &str, id: u64) -> BridgeError {
    bridge_error("bass_handle", format!("unknown {kind} handle: {id}"))
}

fn parse_backend(value: &str) -> Result<OutputBackend, BridgeError> {
    match value.to_ascii_lowercase().as_str() {
        "wasapi" => Ok(OutputBackend::Wasapi),
        "directsound" | "dsound" => Ok(OutputBackend::DirectSound),
        _ => Err(bridge_error("bass_initialize", "backend must be wasapi or directSound")),
    }
}

fn parse_tag_kind(value: &str) -> Result<TagKind, BridgeError> {
    match value.to_ascii_lowercase().as_str() {
        "id3" => Ok(TagKind::Id3),
        "id3v2" => Ok(TagKind::Id3v2),
        "ogg" => Ok(TagKind::Ogg),
        "http" => Ok(TagKind::Http),
        "icy" => Ok(TagKind::Icy),
        "meta" => Ok(TagKind::Meta),
        "ape" => Ok(TagKind::Ape),
        "mp4" => Ok(TagKind::Mp4),
        "wma" => Ok(TagKind::Wma),
        "vendor" => Ok(TagKind::Vendor),
        "mediafoundation" | "mf" => Ok(TagKind::MediaFoundation),
        _ => Err(bridge_error("bass_channel_tags", "unknown tag kind")),
    }
}

fn parse_sync_kind(args: &Value) -> Result<SyncKind, BridgeError> {
    let kind = required_string(args, "kind", "bass_channel_add_sync")?;
    match kind.to_ascii_lowercase().as_str() {
        "position" => Ok(SyncKind::Position(required_u64(args, "parameter", "bass_channel_add_sync")?)),
        "end" => Ok(SyncKind::End),
        "meta" => Ok(SyncKind::Meta),
        "stall" => Ok(SyncKind::Stall),
        "download" => Ok(SyncKind::Download),
        "free" => Ok(SyncKind::Free),
        "oggChange" | "oggchange" => Ok(SyncKind::OggChange),
        "other" => Ok(SyncKind::Other { kind: required_u32(args, "syncType", "bass_channel_add_sync")?, parameter: optional_u64(args, "parameter")?.unwrap_or(0) }),
        _ => Err(bridge_error("bass_channel_add_sync", "unknown sync kind")),
    }
}

fn optional_u64(args: &Value, field: &str) -> Result<Option<u64>, BridgeError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| bridge_error("bass_call", format!("field {field} must be a u64"))),
    }
}

fn source_options(value: Value) -> Result<SourceOptions, BridgeError> {
    let input: SourceInput = parse_args(normalize_object(value), "bass_load_file")?;
    Ok(SourceOptions {
        float: input.float,
        mono: input.mono,
        looped: input.looped,
        decode_only: input.decode_only,
        prescan: input.prescan,
        stream_flags: input.stream_flags,
        music_flags: input.music_flags,
        music_frequency: input.music_frequency,
    })
}

fn normalize_object(value: Value) -> Value {
    if value.is_null() { json!({}) } else { value }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct SourceInput {
    float: bool,
    mono: bool,
    looped: bool,
    decode_only: bool,
    prescan: bool,
    stream_flags: u32,
    music_flags: u32,
    music_frequency: u32,
}

impl Default for SourceInput {
    fn default() -> Self {
        let defaults = SourceOptions::default();
        Self {
            float: defaults.float,
            mono: defaults.mono,
            looped: defaults.looped,
            decode_only: defaults.decode_only,
            prescan: defaults.prescan,
            stream_flags: defaults.stream_flags,
            music_flags: defaults.music_flags,
            music_frequency: defaults.music_frequency,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct UrlInput {
    offset: u32,
    float: bool,
    flags: u32,
}

impl Default for UrlInput {
    fn default() -> Self {
        Self {
            offset: 0,
            float: false,
            flags: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct InitInput {
    device: i32,
    sample_rate: u32,
    backend: String,
    mono: bool,
    exclusive: bool,
    force_frequency: bool,
    float_processing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct LoudnessInput {
    gain_db: f32,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
}

impl Default for LoudnessInput {
    fn default() -> Self {
        let defaults = bass_rs::LoudnessOptions::default();
        Self {
            gain_db: defaults.gain_db,
            threshold_db: defaults.threshold_db,
            ratio: defaults.ratio,
            attack_ms: defaults.attack_ms,
            release_ms: defaults.release_ms,
        }
    }
}

impl Default for InitInput {
    fn default() -> Self {
        Self {
            device: -1,
            sample_rate: 44_100,
            backend: "wasapi".into(),
            mono: false,
            exclusive: false,
            force_frequency: false,
            float_processing: false,
        }
    }
}

fn device_json(device: bass_rs::DeviceInfo) -> Value {
    json!({
        "index": device.index,
        "name": device.name,
        "driver": device.driver,
        "flags": device.flags,
        "deviceType": format!("{:?}", device.device_type),
        "default": device.is_default(),
        "enabled": device.is_enabled(),
        "initialized": device.is_initialized(),
        "loopback": device.is_loopback(),
    })
}

fn plugin_json(id: u64, plugin: &Plugin) -> Value {
    let info = plugin.info();
    json!({
        "pluginId": id,
        "rawHandle": plugin.raw_handle(),
        "version": info.version,
        "formats": info.formats.iter().map(|format| json!({ "channelType": format.channel_type, "name": format.name, "extensions": format.extensions })).collect::<Vec<_>>(),
    })
}

fn channel_json(id: u64, channel: &ChannelObject) -> Value {
    let channel = channel.as_channel();
    json!({ "channelId": id, "rawHandle": channel.raw_handle(), "kind": channel_kind_name(channel.kind()) })
}

fn channel_kind_name(kind: ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Stream => "stream",
        ChannelKind::Music => "music",
        ChannelKind::Url => "url",
        ChannelKind::Derived => "derived",
    }
}

fn active_state_name(state: ActiveState) -> &'static str {
    match state {
        ActiveState::Stopped => "stopped",
        ActiveState::Playing => "playing",
        ActiveState::Stalled => "stalled",
        ActiveState::Paused => "paused",
        ActiveState::PausedDevice => "pausedDevice",
    }
}

fn remote_progress_json(progress: RemoteProgress) -> Value {
    json!({
        "state": active_state_name(progress.state),
        "bufferingPercent": progress.buffering_percent,
        "downloadedBytes": progress.downloaded_bytes,
        "bufferedBytes": progress.buffered_bytes,
        "availableBytes": progress.available_bytes,
        "bytesPerSecond": progress.bytes_per_second,
    })
}

fn effect_kind_name(kind: EffectKind) -> String {
    match kind {
        EffectKind::Dx8(value) => format!("dx8.{value}"),
        EffectKind::BassFx(value) => format!("bassFx.{value:?}"),
        EffectKind::Volume => "volume".into(),
    }
}

fn parse_effect_kind(value: &str) -> Result<EffectKind, BridgeError> {
    let lower = value.to_ascii_lowercase();
    let value = match lower.as_str() {
        "dx8.chorus" => EffectKind::Dx8(raw::BASS_FX_DX8_CHORUS),
        "dx8.compressor" => EffectKind::Dx8(raw::BASS_FX_DX8_COMPRESSOR),
        "dx8.distortion" => EffectKind::Dx8(raw::BASS_FX_DX8_DISTORTION),
        "dx8.echo" => EffectKind::Dx8(raw::BASS_FX_DX8_ECHO),
        "dx8.flanger" => EffectKind::Dx8(raw::BASS_FX_DX8_FLANGER),
        "dx8.gargle" => EffectKind::Dx8(raw::BASS_FX_DX8_GARGLE),
        "dx8.i3dl2reverb" => EffectKind::Dx8(raw::BASS_FX_DX8_I3DL2REVERB),
        "dx8.parameq" => EffectKind::Dx8(raw::BASS_FX_DX8_PARAMEQ),
        "dx8.reverb" => EffectKind::Dx8(raw::BASS_FX_DX8_REVERB),
        "volume" => EffectKind::Volume,
        "bassfx.rotate" => EffectKind::BassFx(BassFxEffect::Rotate),
        "bassfx.echo" => EffectKind::BassFx(BassFxEffect::Echo),
        "bassfx.flanger" => EffectKind::BassFx(BassFxEffect::Flanger),
        "bassfx.volume" => EffectKind::BassFx(BassFxEffect::Volume),
        "bassfx.peakeq" => EffectKind::BassFx(BassFxEffect::PeakEq),
        "bassfx.reverb" => EffectKind::BassFx(BassFxEffect::Reverb),
        "bassfx.lowpassfilter" => EffectKind::BassFx(BassFxEffect::LowPassFilter),
        "bassfx.mix" => EffectKind::BassFx(BassFxEffect::Mix),
        "bassfx.damp" => EffectKind::BassFx(BassFxEffect::Damp),
        "bassfx.autowah" => EffectKind::BassFx(BassFxEffect::AutoWah),
        "bassfx.echo2" => EffectKind::BassFx(BassFxEffect::Echo2),
        "bassfx.phaser" => EffectKind::BassFx(BassFxEffect::Phaser),
        "bassfx.echo3" => EffectKind::BassFx(BassFxEffect::Echo3),
        "bassfx.chorus" => EffectKind::BassFx(BassFxEffect::Chorus),
        "bassfx.allpassfilter" => EffectKind::BassFx(BassFxEffect::AllPassFilter),
        "bassfx.compressor" => EffectKind::BassFx(BassFxEffect::Compressor),
        "bassfx.distortion" => EffectKind::BassFx(BassFxEffect::Distortion),
        "bassfx.compressor2" => EffectKind::BassFx(BassFxEffect::Compressor2),
        "bassfx.volumeenvelope" => EffectKind::BassFx(BassFxEffect::VolumeEnvelope),
        "bassfx.biquadfilter" => EffectKind::BassFx(BassFxEffect::BiquadFilter),
        "bassfx.echo4" => EffectKind::BassFx(BassFxEffect::Echo4),
        "bassfx.pitchshift" => EffectKind::BassFx(BassFxEffect::PitchShift),
        "bassfx.freeverb" => EffectKind::BassFx(BassFxEffect::Freeverb),
        _ => return Err(bridge_error("bass_add_effect", format!("unknown effect kind: {value}"))),
    };
    Ok(value)
}

fn raw_catalog() -> Value {
    let mut constants = serde_json::Map::new();
    constants.insert("BASSVERSION".into(), json!(raw::BASSVERSION));
    constants.insert("BASS_OK".into(), json!(raw::BASS_OK));
    constants.insert("BASS_ERROR_MEM".into(), json!(raw::BASS_ERROR_MEM));
    constants.insert("BASS_ERROR_FILEOPEN".into(), json!(raw::BASS_ERROR_FILEOPEN));
    constants.insert("BASS_ERROR_DRIVER".into(), json!(raw::BASS_ERROR_DRIVER));
    constants.insert("BASS_ERROR_BUFLOST".into(), json!(raw::BASS_ERROR_BUFLOST));
    constants.insert("BASS_ERROR_HANDLE".into(), json!(raw::BASS_ERROR_HANDLE));
    constants.insert("BASS_ERROR_FORMAT".into(), json!(raw::BASS_ERROR_FORMAT));
    constants.insert("BASS_ERROR_POSITION".into(), json!(raw::BASS_ERROR_POSITION));
    constants.insert("BASS_ERROR_INIT".into(), json!(raw::BASS_ERROR_INIT));
    constants.insert("BASS_ERROR_START".into(), json!(raw::BASS_ERROR_START));
    constants.insert("BASS_ERROR_SSL".into(), json!(raw::BASS_ERROR_SSL));
    constants.insert("BASS_ERROR_REINIT".into(), json!(raw::BASS_ERROR_REINIT));
    constants.insert("BASS_ERROR_TRACK".into(), json!(raw::BASS_ERROR_TRACK));
    constants.insert("BASS_ERROR_ALREADY".into(), json!(raw::BASS_ERROR_ALREADY));
    constants.insert("BASS_ERROR_NOTAUDIO".into(), json!(raw::BASS_ERROR_NOTAUDIO));
    constants.insert("BASS_ERROR_NOCHAN".into(), json!(raw::BASS_ERROR_NOCHAN));
    constants.insert("BASS_ERROR_ILLTYPE".into(), json!(raw::BASS_ERROR_ILLTYPE));
    constants.insert("BASS_ERROR_ILLPARAM".into(), json!(raw::BASS_ERROR_ILLPARAM));
    constants.insert("BASS_ERROR_NO3D".into(), json!(raw::BASS_ERROR_NO3D));
    constants.insert("BASS_ERROR_NOEAX".into(), json!(raw::BASS_ERROR_NOEAX));
    constants.insert("BASS_ERROR_DEVICE".into(), json!(raw::BASS_ERROR_DEVICE));
    constants.insert("BASS_ERROR_NOPLAY".into(), json!(raw::BASS_ERROR_NOPLAY));
    constants.insert("BASS_ERROR_FREQ".into(), json!(raw::BASS_ERROR_FREQ));
    constants.insert("BASS_ERROR_NOTFILE".into(), json!(raw::BASS_ERROR_NOTFILE));
    constants.insert("BASS_ERROR_NOHW".into(), json!(raw::BASS_ERROR_NOHW));
    constants.insert("BASS_ERROR_EMPTY".into(), json!(raw::BASS_ERROR_EMPTY));
    constants.insert("BASS_ERROR_NONET".into(), json!(raw::BASS_ERROR_NONET));
    constants.insert("BASS_ERROR_CREATE".into(), json!(raw::BASS_ERROR_CREATE));
    constants.insert("BASS_ERROR_NOFX".into(), json!(raw::BASS_ERROR_NOFX));
    constants.insert("BASS_ERROR_NOTAVAIL".into(), json!(raw::BASS_ERROR_NOTAVAIL));
    constants.insert("BASS_ERROR_DECODE".into(), json!(raw::BASS_ERROR_DECODE));
    constants.insert("BASS_ERROR_DX".into(), json!(raw::BASS_ERROR_DX));
    constants.insert("BASS_ERROR_TIMEOUT".into(), json!(raw::BASS_ERROR_TIMEOUT));
    constants.insert("BASS_ERROR_FILEFORM".into(), json!(raw::BASS_ERROR_FILEFORM));
    constants.insert("BASS_ERROR_SPEAKER".into(), json!(raw::BASS_ERROR_SPEAKER));
    constants.insert("BASS_ERROR_VERSION".into(), json!(raw::BASS_ERROR_VERSION));
    constants.insert("BASS_ERROR_CODEC".into(), json!(raw::BASS_ERROR_CODEC));
    constants.insert("BASS_ERROR_ENDED".into(), json!(raw::BASS_ERROR_ENDED));
    constants.insert("BASS_ERROR_BUSY".into(), json!(raw::BASS_ERROR_BUSY));
    constants.insert("BASS_ERROR_UNSTREAMABLE".into(), json!(raw::BASS_ERROR_UNSTREAMABLE));
    constants.insert("BASS_ERROR_PROTOCOL".into(), json!(raw::BASS_ERROR_PROTOCOL));
    constants.insert("BASS_ERROR_DENIED".into(), json!(raw::BASS_ERROR_DENIED));
    constants.insert("BASS_ERROR_FREEING".into(), json!(raw::BASS_ERROR_FREEING));
    constants.insert("BASS_ERROR_CANCEL".into(), json!(raw::BASS_ERROR_CANCEL));
    constants.insert("BASS_CONFIG_BUFFER".into(), json!(raw::BASS_CONFIG_BUFFER));
    constants.insert("BASS_CONFIG_UPDATEPERIOD".into(), json!(raw::BASS_CONFIG_UPDATEPERIOD));
    constants.insert("BASS_CONFIG_GVOL_SAMPLE".into(), json!(raw::BASS_CONFIG_GVOL_SAMPLE));
    constants.insert("BASS_CONFIG_GVOL_STREAM".into(), json!(raw::BASS_CONFIG_GVOL_STREAM));
    constants.insert("BASS_CONFIG_GVOL_MUSIC".into(), json!(raw::BASS_CONFIG_GVOL_MUSIC));
    constants.insert("BASS_CONFIG_CURVE_VOL".into(), json!(raw::BASS_CONFIG_CURVE_VOL));
    constants.insert("BASS_CONFIG_CURVE_PAN".into(), json!(raw::BASS_CONFIG_CURVE_PAN));
    constants.insert("BASS_CONFIG_FLOATDSP".into(), json!(raw::BASS_CONFIG_FLOATDSP));
    constants.insert("BASS_CONFIG_3DALGORITHM".into(), json!(raw::BASS_CONFIG_3DALGORITHM));
    constants.insert("BASS_CONFIG_NET_TIMEOUT".into(), json!(raw::BASS_CONFIG_NET_TIMEOUT));
    constants.insert("BASS_CONFIG_NET_BUFFER".into(), json!(raw::BASS_CONFIG_NET_BUFFER));
    constants.insert("BASS_CONFIG_PAUSE_NOPLAY".into(), json!(raw::BASS_CONFIG_PAUSE_NOPLAY));
    constants.insert("BASS_CONFIG_NET_PREBUF".into(), json!(raw::BASS_CONFIG_NET_PREBUF));
    constants.insert("BASS_CONFIG_NET_PASSIVE".into(), json!(raw::BASS_CONFIG_NET_PASSIVE));
    constants.insert("BASS_CONFIG_REC_BUFFER".into(), json!(raw::BASS_CONFIG_REC_BUFFER));
    constants.insert("BASS_CONFIG_NET_PLAYLIST".into(), json!(raw::BASS_CONFIG_NET_PLAYLIST));
    constants.insert("BASS_CONFIG_MUSIC_VIRTUAL".into(), json!(raw::BASS_CONFIG_MUSIC_VIRTUAL));
    constants.insert("BASS_CONFIG_VERIFY".into(), json!(raw::BASS_CONFIG_VERIFY));
    constants.insert("BASS_CONFIG_UPDATETHREADS".into(), json!(raw::BASS_CONFIG_UPDATETHREADS));
    constants.insert("BASS_CONFIG_DEV_BUFFER".into(), json!(raw::BASS_CONFIG_DEV_BUFFER));
    constants.insert("BASS_CONFIG_REC_LOOPBACK".into(), json!(raw::BASS_CONFIG_REC_LOOPBACK));
    constants.insert("BASS_CONFIG_DEV_DEFAULT".into(), json!(raw::BASS_CONFIG_DEV_DEFAULT));
    constants.insert("BASS_CONFIG_NET_READTIMEOUT".into(), json!(raw::BASS_CONFIG_NET_READTIMEOUT));
    constants.insert("BASS_CONFIG_VISTA_SPEAKERS".into(), json!(raw::BASS_CONFIG_VISTA_SPEAKERS));
    constants.insert("BASS_CONFIG_MF_DISABLE".into(), json!(raw::BASS_CONFIG_MF_DISABLE));
    constants.insert("BASS_CONFIG_HANDLES".into(), json!(raw::BASS_CONFIG_HANDLES));
    constants.insert("BASS_CONFIG_UNICODE".into(), json!(raw::BASS_CONFIG_UNICODE));
    constants.insert("BASS_CONFIG_SRC".into(), json!(raw::BASS_CONFIG_SRC));
    constants.insert("BASS_CONFIG_SRC_SAMPLE".into(), json!(raw::BASS_CONFIG_SRC_SAMPLE));
    constants.insert("BASS_CONFIG_ASYNCFILE_BUFFER".into(), json!(raw::BASS_CONFIG_ASYNCFILE_BUFFER));
    constants.insert("BASS_CONFIG_OGG_PRESCAN".into(), json!(raw::BASS_CONFIG_OGG_PRESCAN));
    constants.insert("BASS_CONFIG_VIDEO".into(), json!(raw::BASS_CONFIG_VIDEO));
    constants.insert("BASS_CONFIG_DEV_NONSTOP".into(), json!(raw::BASS_CONFIG_DEV_NONSTOP));
    constants.insert("BASS_CONFIG_VERIFY_NET".into(), json!(raw::BASS_CONFIG_VERIFY_NET));
    constants.insert("BASS_CONFIG_DEV_PERIOD".into(), json!(raw::BASS_CONFIG_DEV_PERIOD));
    constants.insert("BASS_CONFIG_FLOAT".into(), json!(raw::BASS_CONFIG_FLOAT));
    constants.insert("BASS_CONFIG_NET_SEEK".into(), json!(raw::BASS_CONFIG_NET_SEEK));
    constants.insert("BASS_CONFIG_NET_PLAYLIST_DEPTH".into(), json!(raw::BASS_CONFIG_NET_PLAYLIST_DEPTH));
    constants.insert("BASS_CONFIG_NET_PREBUF_WAIT".into(), json!(raw::BASS_CONFIG_NET_PREBUF_WAIT));
    constants.insert("BASS_CONFIG_WASAPI_PERSIST".into(), json!(raw::BASS_CONFIG_WASAPI_PERSIST));
    constants.insert("BASS_CONFIG_REC_WASAPI".into(), json!(raw::BASS_CONFIG_REC_WASAPI));
    constants.insert("BASS_CONFIG_SAMPLE_ONEHANDLE".into(), json!(raw::BASS_CONFIG_SAMPLE_ONEHANDLE));
    constants.insert("BASS_CONFIG_NET_META".into(), json!(raw::BASS_CONFIG_NET_META));
    constants.insert("BASS_CONFIG_NET_RESTRATE".into(), json!(raw::BASS_CONFIG_NET_RESTRATE));
    constants.insert("BASS_CONFIG_REC_DEFAULT".into(), json!(raw::BASS_CONFIG_REC_DEFAULT));
    constants.insert("BASS_CONFIG_NORAMP".into(), json!(raw::BASS_CONFIG_NORAMP));
    constants.insert("BASS_CONFIG_NOSOUND_MAXDELAY".into(), json!(raw::BASS_CONFIG_NOSOUND_MAXDELAY));
    constants.insert("BASS_CONFIG_DOWNMIX".into(), json!(raw::BASS_CONFIG_DOWNMIX));
    constants.insert("BASS_CONFIG_NET_AGENT".into(), json!(raw::BASS_CONFIG_NET_AGENT));
    constants.insert("BASS_CONFIG_NET_PROXY".into(), json!(raw::BASS_CONFIG_NET_PROXY));
    constants.insert("BASS_CONFIG_DEV_NOTIFY".into(), json!(raw::BASS_CONFIG_DEV_NOTIFY));
    constants.insert("BASS_CONFIG_FILENAME".into(), json!(raw::BASS_CONFIG_FILENAME));
    constants.insert("BASS_CONFIG_THREAD".into(), json!(raw::BASS_CONFIG_THREAD));
    constants.insert("BASS_DEVICE_MONO".into(), json!(raw::BASS_DEVICE_MONO));
    constants.insert("BASS_DEVICE_REINIT".into(), json!(raw::BASS_DEVICE_REINIT));
    constants.insert("BASS_DEVICE_SPEAKERS".into(), json!(raw::BASS_DEVICE_SPEAKERS));
    constants.insert("BASS_DEVICE_NOSPEAKER".into(), json!(raw::BASS_DEVICE_NOSPEAKER));
    constants.insert("BASS_DEVICE_FREQ".into(), json!(raw::BASS_DEVICE_FREQ));
    constants.insert("BASS_DEVICE_STEREO".into(), json!(raw::BASS_DEVICE_STEREO));
    constants.insert("BASS_DEVICE_HOG".into(), json!(raw::BASS_DEVICE_HOG));
    constants.insert("BASS_DEVICE_DSOUND".into(), json!(raw::BASS_DEVICE_DSOUND));
    constants.insert("BASS_DEVICE_SOFTWARE".into(), json!(raw::BASS_DEVICE_SOFTWARE));
    constants.insert("BASS_DEVICE_ENABLED".into(), json!(raw::BASS_DEVICE_ENABLED));
    constants.insert("BASS_DEVICE_DEFAULT".into(), json!(raw::BASS_DEVICE_DEFAULT));
    constants.insert("BASS_DEVICE_INIT".into(), json!(raw::BASS_DEVICE_INIT));
    constants.insert("BASS_DEVICE_LOOPBACK".into(), json!(raw::BASS_DEVICE_LOOPBACK));
    constants.insert("BASS_DEVICE_DEFAULTCOM".into(), json!(raw::BASS_DEVICE_DEFAULTCOM));
    constants.insert("BASS_DEVICE_TYPE_MASK".into(), json!(raw::BASS_DEVICE_TYPE_MASK));
    constants.insert("BASS_DEVICE_TYPE_NETWORK".into(), json!(raw::BASS_DEVICE_TYPE_NETWORK));
    constants.insert("BASS_DEVICE_TYPE_SPEAKERS".into(), json!(raw::BASS_DEVICE_TYPE_SPEAKERS));
    constants.insert("BASS_DEVICE_TYPE_LINE".into(), json!(raw::BASS_DEVICE_TYPE_LINE));
    constants.insert("BASS_DEVICE_TYPE_HEADPHONES".into(), json!(raw::BASS_DEVICE_TYPE_HEADPHONES));
    constants.insert("BASS_DEVICE_TYPE_MICROPHONE".into(), json!(raw::BASS_DEVICE_TYPE_MICROPHONE));
    constants.insert("BASS_DEVICE_TYPE_HEADSET".into(), json!(raw::BASS_DEVICE_TYPE_HEADSET));
    constants.insert("BASS_DEVICE_TYPE_HANDSET".into(), json!(raw::BASS_DEVICE_TYPE_HANDSET));
    constants.insert("BASS_DEVICE_TYPE_DIGITAL".into(), json!(raw::BASS_DEVICE_TYPE_DIGITAL));
    constants.insert("BASS_DEVICE_TYPE_SPDIF".into(), json!(raw::BASS_DEVICE_TYPE_SPDIF));
    constants.insert("BASS_DEVICE_TYPE_HDMI".into(), json!(raw::BASS_DEVICE_TYPE_HDMI));
    constants.insert("BASS_DEVICE_TYPE_DISPLAYPORT".into(), json!(raw::BASS_DEVICE_TYPE_DISPLAYPORT));
    constants.insert("BASS_FILE_NAME".into(), json!(raw::BASS_FILE_NAME));
    constants.insert("BASS_FILE_MEM".into(), json!(raw::BASS_FILE_MEM));
    constants.insert("BASS_FILE_MEMCOPY".into(), json!(raw::BASS_FILE_MEMCOPY));
    constants.insert("BASS_FILE_HANDLE".into(), json!(raw::BASS_FILE_HANDLE));
    constants.insert("BASS_SAMPLE_MONO".into(), json!(raw::BASS_SAMPLE_MONO));
    constants.insert("BASS_SAMPLE_LOOP".into(), json!(raw::BASS_SAMPLE_LOOP));
    constants.insert("BASS_SAMPLE_FLOAT".into(), json!(raw::BASS_SAMPLE_FLOAT));
    constants.insert("BASS_SAMPLE_FX".into(), json!(raw::BASS_SAMPLE_FX));
    constants.insert("BASS_STREAM_PRESCAN".into(), json!(raw::BASS_STREAM_PRESCAN));
    constants.insert("BASS_STREAM_AUTOFREE".into(), json!(raw::BASS_STREAM_AUTOFREE));
    constants.insert("BASS_STREAM_RESTRATE".into(), json!(raw::BASS_STREAM_RESTRATE));
    constants.insert("BASS_STREAM_BLOCK".into(), json!(raw::BASS_STREAM_BLOCK));
    constants.insert("BASS_STREAM_DECODE".into(), json!(raw::BASS_STREAM_DECODE));
    constants.insert("BASS_STREAM_STATUS".into(), json!(raw::BASS_STREAM_STATUS));
    constants.insert("BASS_MUSIC_FLOAT".into(), json!(raw::BASS_MUSIC_FLOAT));
    constants.insert("BASS_MUSIC_MONO".into(), json!(raw::BASS_MUSIC_MONO));
    constants.insert("BASS_MUSIC_LOOP".into(), json!(raw::BASS_MUSIC_LOOP));
    constants.insert("BASS_MUSIC_DECODE".into(), json!(raw::BASS_MUSIC_DECODE));
    constants.insert("BASS_MUSIC_PRESCAN".into(), json!(raw::BASS_MUSIC_PRESCAN));
    constants.insert("BASS_MUSIC_RAMP".into(), json!(raw::BASS_MUSIC_RAMP));
    constants.insert("BASS_MUSIC_RAMPS".into(), json!(raw::BASS_MUSIC_RAMPS));
    constants.insert("BASS_MUSIC_SURROUND".into(), json!(raw::BASS_MUSIC_SURROUND));
    constants.insert("BASS_MUSIC_SURROUND2".into(), json!(raw::BASS_MUSIC_SURROUND2));
    constants.insert("BASS_MUSIC_SINCINTER".into(), json!(raw::BASS_MUSIC_SINCINTER));
    constants.insert("BASS_MUSIC_AUTOFREE".into(), json!(raw::BASS_MUSIC_AUTOFREE));
    constants.insert("BASS_UNICODE".into(), json!(raw::BASS_UNICODE));
    constants.insert("BASS_FX_FREESOURCE".into(), json!(raw::BASS_FX_FREESOURCE));
    constants.insert("BASS_FILEPOS_CURRENT".into(), json!(raw::BASS_FILEPOS_CURRENT));
    constants.insert("BASS_FILEPOS_DOWNLOAD".into(), json!(raw::BASS_FILEPOS_DOWNLOAD));
    constants.insert("BASS_FILEPOS_END".into(), json!(raw::BASS_FILEPOS_END));
    constants.insert("BASS_FILEPOS_START".into(), json!(raw::BASS_FILEPOS_START));
    constants.insert("BASS_FILEPOS_CONNECTED".into(), json!(raw::BASS_FILEPOS_CONNECTED));
    constants.insert("BASS_FILEPOS_BUFFER".into(), json!(raw::BASS_FILEPOS_BUFFER));
    constants.insert("BASS_FILEPOS_SOCKET".into(), json!(raw::BASS_FILEPOS_SOCKET));
    constants.insert("BASS_FILEPOS_ASYNCBUF".into(), json!(raw::BASS_FILEPOS_ASYNCBUF));
    constants.insert("BASS_FILEPOS_SIZE".into(), json!(raw::BASS_FILEPOS_SIZE));
    constants.insert("BASS_FILEPOS_BUFFERING".into(), json!(raw::BASS_FILEPOS_BUFFERING));
    constants.insert("BASS_FILEPOS_AVAILABLE".into(), json!(raw::BASS_FILEPOS_AVAILABLE));
    constants.insert("BASS_SYNC_POS".into(), json!(raw::BASS_SYNC_POS));
    constants.insert("BASS_SYNC_END".into(), json!(raw::BASS_SYNC_END));
    constants.insert("BASS_SYNC_META".into(), json!(raw::BASS_SYNC_META));
    constants.insert("BASS_SYNC_STALL".into(), json!(raw::BASS_SYNC_STALL));
    constants.insert("BASS_SYNC_DOWNLOAD".into(), json!(raw::BASS_SYNC_DOWNLOAD));
    constants.insert("BASS_SYNC_FREE".into(), json!(raw::BASS_SYNC_FREE));
    constants.insert("BASS_SYNC_OGG_CHANGE".into(), json!(raw::BASS_SYNC_OGG_CHANGE));
    constants.insert("BASS_SYNC_THREAD".into(), json!(raw::BASS_SYNC_THREAD));
    constants.insert("BASS_SYNC_MIXTIME".into(), json!(raw::BASS_SYNC_MIXTIME));
    constants.insert("BASS_SYNC_ONETIME".into(), json!(raw::BASS_SYNC_ONETIME));
    constants.insert("BASS_ACTIVE_STOPPED".into(), json!(raw::BASS_ACTIVE_STOPPED));
    constants.insert("BASS_ACTIVE_PLAYING".into(), json!(raw::BASS_ACTIVE_PLAYING));
    constants.insert("BASS_ACTIVE_STALLED".into(), json!(raw::BASS_ACTIVE_STALLED));
    constants.insert("BASS_ACTIVE_PAUSED".into(), json!(raw::BASS_ACTIVE_PAUSED));
    constants.insert("BASS_ACTIVE_PAUSED_DEVICE".into(), json!(raw::BASS_ACTIVE_PAUSED_DEVICE));
    constants.insert("BASS_ATTRIB_FREQ".into(), json!(raw::BASS_ATTRIB_FREQ));
    constants.insert("BASS_ATTRIB_VOL".into(), json!(raw::BASS_ATTRIB_VOL));
    constants.insert("BASS_ATTRIB_PAN".into(), json!(raw::BASS_ATTRIB_PAN));
    constants.insert("BASS_ATTRIB_BUFFER".into(), json!(raw::BASS_ATTRIB_BUFFER));
    constants.insert("BASS_ATTRIB_DOWNLOADPROC".into(), json!(raw::BASS_ATTRIB_DOWNLOADPROC));
    constants.insert("BASS_ATTRIB_DOWNMIX".into(), json!(raw::BASS_ATTRIB_DOWNMIX));
    constants.insert("BASS_ATTRIB_TEMPO".into(), json!(raw::BASS_ATTRIB_TEMPO));
    constants.insert("BASS_ATTRIB_TEMPO_PITCH".into(), json!(raw::BASS_ATTRIB_TEMPO_PITCH));
    constants.insert("BASS_ATTRIB_TEMPO_FREQ".into(), json!(raw::BASS_ATTRIB_TEMPO_FREQ));
    constants.insert("BASS_ATTRIB_REVERSE_DIR".into(), json!(raw::BASS_ATTRIB_REVERSE_DIR));
    constants.insert("BASS_POS_BYTE".into(), json!(raw::BASS_POS_BYTE));
    constants.insert("BASS_POS_MUSIC_ORDER".into(), json!(raw::BASS_POS_MUSIC_ORDER));
    constants.insert("BASS_POS_DSP".into(), json!(raw::BASS_POS_DSP));
    constants.insert("BASS_POS_FLUSH".into(), json!(raw::BASS_POS_FLUSH));
    constants.insert("BASS_POS_RELATIVE".into(), json!(raw::BASS_POS_RELATIVE));
    constants.insert("BASS_POS_INEXACT".into(), json!(raw::BASS_POS_INEXACT));
    constants.insert("BASS_POS_DECODE".into(), json!(raw::BASS_POS_DECODE));
    constants.insert("BASS_DATA_AVAILABLE".into(), json!(raw::BASS_DATA_AVAILABLE));
    constants.insert("BASS_DATA_FLOAT".into(), json!(raw::BASS_DATA_FLOAT));
    constants.insert("BASS_DATA_FFT256".into(), json!(raw::BASS_DATA_FFT256));
    constants.insert("BASS_DATA_FFT512".into(), json!(raw::BASS_DATA_FFT512));
    constants.insert("BASS_DATA_FFT1024".into(), json!(raw::BASS_DATA_FFT1024));
    constants.insert("BASS_DATA_FFT2048".into(), json!(raw::BASS_DATA_FFT2048));
    constants.insert("BASS_DATA_FFT4096".into(), json!(raw::BASS_DATA_FFT4096));
    constants.insert("BASS_DATA_FFT8192".into(), json!(raw::BASS_DATA_FFT8192));
    constants.insert("BASS_DATA_FFT_INDIVIDUAL".into(), json!(raw::BASS_DATA_FFT_INDIVIDUAL));
    constants.insert("BASS_DATA_FFT_NOWINDOW".into(), json!(raw::BASS_DATA_FFT_NOWINDOW));
    constants.insert("BASS_DATA_FFT_REMOVEDC".into(), json!(raw::BASS_DATA_FFT_REMOVEDC));
    constants.insert("BASS_DATA_FFT_COMPLEX".into(), json!(raw::BASS_DATA_FFT_COMPLEX));
    constants.insert("BASS_DATA_FFT_NYQUIST".into(), json!(raw::BASS_DATA_FFT_NYQUIST));
    constants.insert("BASS_LEVEL_MONO".into(), json!(raw::BASS_LEVEL_MONO));
    constants.insert("BASS_LEVEL_STEREO".into(), json!(raw::BASS_LEVEL_STEREO));
    constants.insert("BASS_LEVEL_RMS".into(), json!(raw::BASS_LEVEL_RMS));
    constants.insert("BASS_LEVEL_VOLPAN".into(), json!(raw::BASS_LEVEL_VOLPAN));
    constants.insert("BASS_TAG_ID3".into(), json!(raw::BASS_TAG_ID3));
    constants.insert("BASS_TAG_ID3V2".into(), json!(raw::BASS_TAG_ID3V2));
    constants.insert("BASS_TAG_OGG".into(), json!(raw::BASS_TAG_OGG));
    constants.insert("BASS_TAG_HTTP".into(), json!(raw::BASS_TAG_HTTP));
    constants.insert("BASS_TAG_ICY".into(), json!(raw::BASS_TAG_ICY));
    constants.insert("BASS_TAG_META".into(), json!(raw::BASS_TAG_META));
    constants.insert("BASS_TAG_APE".into(), json!(raw::BASS_TAG_APE));
    constants.insert("BASS_TAG_MP4".into(), json!(raw::BASS_TAG_MP4));
    constants.insert("BASS_TAG_WMA".into(), json!(raw::BASS_TAG_WMA));
    constants.insert("BASS_TAG_VENDOR".into(), json!(raw::BASS_TAG_VENDOR));
    constants.insert("BASS_TAG_MF".into(), json!(raw::BASS_TAG_MF));
    constants.insert("BASS_DSP_READONLY".into(), json!(raw::BASS_DSP_READONLY));
    constants.insert("BASS_DSP_FLOAT".into(), json!(raw::BASS_DSP_FLOAT));
    constants.insert("BASS_DSP_FREECALL".into(), json!(raw::BASS_DSP_FREECALL));
    constants.insert("BASS_DSP_BYPASS".into(), json!(raw::BASS_DSP_BYPASS));
    constants.insert("BASS_FX_DX8_CHORUS".into(), json!(raw::BASS_FX_DX8_CHORUS));
    constants.insert("BASS_FX_DX8_COMPRESSOR".into(), json!(raw::BASS_FX_DX8_COMPRESSOR));
    constants.insert("BASS_FX_DX8_DISTORTION".into(), json!(raw::BASS_FX_DX8_DISTORTION));
    constants.insert("BASS_FX_DX8_ECHO".into(), json!(raw::BASS_FX_DX8_ECHO));
    constants.insert("BASS_FX_DX8_FLANGER".into(), json!(raw::BASS_FX_DX8_FLANGER));
    constants.insert("BASS_FX_DX8_GARGLE".into(), json!(raw::BASS_FX_DX8_GARGLE));
    constants.insert("BASS_FX_DX8_I3DL2REVERB".into(), json!(raw::BASS_FX_DX8_I3DL2REVERB));
    constants.insert("BASS_FX_DX8_PARAMEQ".into(), json!(raw::BASS_FX_DX8_PARAMEQ));
    constants.insert("BASS_FX_DX8_REVERB".into(), json!(raw::BASS_FX_DX8_REVERB));
    constants.insert("BASS_FX_VOLUME".into(), json!(raw::BASS_FX_VOLUME));
    constants.insert("BASS_FX_BFX_ROTATE".into(), json!(raw::BASS_FX_BFX_ROTATE));
    constants.insert("BASS_FX_BFX_ECHO".into(), json!(raw::BASS_FX_BFX_ECHO));
    constants.insert("BASS_FX_BFX_FLANGER".into(), json!(raw::BASS_FX_BFX_FLANGER));
    constants.insert("BASS_FX_BFX_VOLUME".into(), json!(raw::BASS_FX_BFX_VOLUME));
    constants.insert("BASS_FX_BFX_PEAKEQ".into(), json!(raw::BASS_FX_BFX_PEAKEQ));
    constants.insert("BASS_FX_BFX_REVERB".into(), json!(raw::BASS_FX_BFX_REVERB));
    constants.insert("BASS_FX_BFX_LPF".into(), json!(raw::BASS_FX_BFX_LPF));
    constants.insert("BASS_FX_BFX_MIX".into(), json!(raw::BASS_FX_BFX_MIX));
    constants.insert("BASS_FX_BFX_DAMP".into(), json!(raw::BASS_FX_BFX_DAMP));
    constants.insert("BASS_FX_BFX_AUTOWAH".into(), json!(raw::BASS_FX_BFX_AUTOWAH));
    constants.insert("BASS_FX_BFX_ECHO2".into(), json!(raw::BASS_FX_BFX_ECHO2));
    constants.insert("BASS_FX_BFX_PHASER".into(), json!(raw::BASS_FX_BFX_PHASER));
    constants.insert("BASS_FX_BFX_ECHO3".into(), json!(raw::BASS_FX_BFX_ECHO3));
    constants.insert("BASS_FX_BFX_CHORUS".into(), json!(raw::BASS_FX_BFX_CHORUS));
    constants.insert("BASS_FX_BFX_APF".into(), json!(raw::BASS_FX_BFX_APF));
    constants.insert("BASS_FX_BFX_COMPRESSOR".into(), json!(raw::BASS_FX_BFX_COMPRESSOR));
    constants.insert("BASS_FX_BFX_DISTORTION".into(), json!(raw::BASS_FX_BFX_DISTORTION));
    constants.insert("BASS_FX_BFX_COMPRESSOR2".into(), json!(raw::BASS_FX_BFX_COMPRESSOR2));
    constants.insert("BASS_FX_BFX_VOLUME_ENV".into(), json!(raw::BASS_FX_BFX_VOLUME_ENV));
    constants.insert("BASS_FX_BFX_BQF".into(), json!(raw::BASS_FX_BFX_BQF));
    constants.insert("BASS_FX_BFX_ECHO4".into(), json!(raw::BASS_FX_BFX_ECHO4));
    constants.insert("BASS_FX_BFX_PITCHSHIFT".into(), json!(raw::BASS_FX_BFX_PITCHSHIFT));
    constants.insert("BASS_FX_BFX_FREEVERB".into(), json!(raw::BASS_FX_BFX_FREEVERB));
    constants.insert("BASS_BFX_CHANALL".into(), json!(raw::BASS_BFX_CHANALL));
    constants.insert("BASS_BFX_CHANNONE".into(), json!(raw::BASS_BFX_CHANNONE));
    constants.insert("BASS_BFX_CHAN1".into(), json!(raw::BASS_BFX_CHAN1));
    constants.insert("BASS_BFX_CHAN2".into(), json!(raw::BASS_BFX_CHAN2));
    constants.insert("BASS_BFX_CHAN3".into(), json!(raw::BASS_BFX_CHAN3));
    constants.insert("BASS_BFX_CHAN4".into(), json!(raw::BASS_BFX_CHAN4));
    constants.insert("BASS_BFX_CHAN5".into(), json!(raw::BASS_BFX_CHAN5));
    constants.insert("BASS_BFX_CHAN6".into(), json!(raw::BASS_BFX_CHAN6));
    constants.insert("BASS_BFX_CHAN7".into(), json!(raw::BASS_BFX_CHAN7));
    constants.insert("BASS_BFX_CHAN8".into(), json!(raw::BASS_BFX_CHAN8));
    constants.insert("BASS_BFX_BQF_LOWPASS".into(), json!(raw::BASS_BFX_BQF_LOWPASS));
    constants.insert("BASS_BFX_BQF_HIGHPASS".into(), json!(raw::BASS_BFX_BQF_HIGHPASS));
    constants.insert("BASS_BFX_BQF_BANDPASS".into(), json!(raw::BASS_BFX_BQF_BANDPASS));
    constants.insert("BASS_BFX_BQF_BANDPASS_Q".into(), json!(raw::BASS_BFX_BQF_BANDPASS_Q));
    constants.insert("BASS_BFX_BQF_NOTCH".into(), json!(raw::BASS_BFX_BQF_NOTCH));
    constants.insert("BASS_BFX_BQF_ALLPASS".into(), json!(raw::BASS_BFX_BQF_ALLPASS));
    constants.insert("BASS_BFX_BQF_PEAKINGEQ".into(), json!(raw::BASS_BFX_BQF_PEAKINGEQ));
    constants.insert("BASS_BFX_BQF_LOWSHELF".into(), json!(raw::BASS_BFX_BQF_LOWSHELF));
    constants.insert("BASS_BFX_BQF_HIGHSHELF".into(), json!(raw::BASS_BFX_BQF_HIGHSHELF));
    constants.insert("BASS_FX_TEMPO_ALGO_LINEAR".into(), json!(raw::BASS_FX_TEMPO_ALGO_LINEAR));
    constants.insert("BASS_FX_TEMPO_ALGO_CUBIC".into(), json!(raw::BASS_FX_TEMPO_ALGO_CUBIC));
    constants.insert("BASS_FX_TEMPO_ALGO_SHANNON".into(), json!(raw::BASS_FX_TEMPO_ALGO_SHANNON));
    constants.insert("BASS_FX_RVS_REVERSE".into(), json!(raw::BASS_FX_RVS_REVERSE));
    constants.insert("BASS_FX_RVS_FORWARD".into(), json!(raw::BASS_FX_RVS_FORWARD));
    json!({
        "bassApiVersion": bass_rs::BASS_API_VERSION,
        "bassFxApiVersion": bass_rs::BASS_FX_API_VERSION,
        "constants": constants,
        "constantCount": 283,
    })
}

fn set_effect_parameters(effect: &Effect, kind: EffectKind, value: &Value) -> bass_rs::Result<()> {
    match kind {
        EffectKind::Dx8(raw::BASS_FX_DX8_PARAMEQ) => {
            effect.set_parameters(&raw::BASS_DX8_PARAMEQ { fCenter: number(value, "fCenter")?, fBandwidth: number(value, "fBandwidth")?, fGain: number(value, "fGain")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_CHORUS) => {
            effect.set_parameters(&raw::BASS_DX8_CHORUS { fWetDryMix: number(value, "fWetDryMix")?, fDepth: number(value, "fDepth")?, fFeedback: number(value, "fFeedback")?, fFrequency: number(value, "fFrequency")?, lWaveform: unsigned(value, "lWaveform")?, fDelay: number(value, "fDelay")?, lPhase: unsigned(value, "lPhase")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_COMPRESSOR) => {
            effect.set_parameters(&raw::BASS_DX8_COMPRESSOR { fGain: number(value, "fGain")?, fAttack: number(value, "fAttack")?, fRelease: number(value, "fRelease")?, fThreshold: number(value, "fThreshold")?, fRatio: number(value, "fRatio")?, fPredelay: number(value, "fPredelay")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_DISTORTION) => {
            effect.set_parameters(&raw::BASS_DX8_DISTORTION { fGain: number(value, "fGain")?, fEdge: number(value, "fEdge")?, fPostEQCenterFrequency: number(value, "fPostEQCenterFrequency")?, fPostEQBandwidth: number(value, "fPostEQBandwidth")?, fPreLowpassCutoff: number(value, "fPreLowpassCutoff")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_ECHO) => {
            effect.set_parameters(&raw::BASS_DX8_ECHO { fWetDryMix: number(value, "fWetDryMix")?, fFeedback: number(value, "fFeedback")?, fLeftDelay: number(value, "fLeftDelay")?, fRightDelay: number(value, "fRightDelay")?, lPanDelay: integer(value, "lPanDelay")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_FLANGER) => {
            effect.set_parameters(&raw::BASS_DX8_FLANGER { fWetDryMix: number(value, "fWetDryMix")?, fDepth: number(value, "fDepth")?, fFeedback: number(value, "fFeedback")?, fFrequency: number(value, "fFrequency")?, lWaveform: unsigned(value, "lWaveform")?, fDelay: number(value, "fDelay")?, lPhase: unsigned(value, "lPhase")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_GARGLE) => {
            effect.set_parameters(&raw::BASS_DX8_GARGLE { dwRateHz: unsigned(value, "dwRateHz")?, dwWaveShape: unsigned(value, "dwWaveShape")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_I3DL2REVERB) => {
            effect.set_parameters(&raw::BASS_DX8_I3DL2REVERB { lRoom: integer(value, "lRoom")?, lRoomHF: integer(value, "lRoomHF")?, flRoomRolloffFactor: number(value, "flRoomRolloffFactor")?, flDecayTime: number(value, "flDecayTime")?, flDecayHFRatio: number(value, "flDecayHFRatio")?, lReflections: integer(value, "lReflections")?, flReflectionsDelay: number(value, "flReflectionsDelay")?, lReverb: integer(value, "lReverb")?, flReverbDelay: number(value, "flReverbDelay")?, flDiffusion: number(value, "flDiffusion")?, flDensity: number(value, "flDensity")?, flHFReference: number(value, "flHFReference")? })
        }
        EffectKind::Dx8(raw::BASS_FX_DX8_REVERB) => {
            effect.set_parameters(&raw::BASS_DX8_REVERB { fInGain: number(value, "fInGain")?, fReverbMix: number(value, "fReverbMix")?, fReverbTime: number(value, "fReverbTime")?, fHighFreqRTRatio: number(value, "fHighFreqRTRatio")? })
        }
        EffectKind::BassFx(BassFxEffect::Freeverb) => {
            effect.set_parameters(&raw::BASS_BFX_FREEVERB { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fRoomSize: number(value, "fRoomSize")?, fDamp: number(value, "fDamp")?, fWidth: number(value, "fWidth")?, lMode: unsigned(value, "lMode")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Rotate) => {
            effect.set_parameters(&raw::BASS_BFX_ROTATE { fRate: number(value, "fRate")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Echo) => {
            effect.set_parameters(&raw::BASS_BFX_ECHO { fLevel: number(value, "fLevel")?, lDelay: integer(value, "lDelay")? })
        }
        EffectKind::BassFx(BassFxEffect::Flanger) => {
            effect.set_parameters(&raw::BASS_BFX_FLANGER { fWetDry: number(value, "fWetDry")?, fSpeed: number(value, "fSpeed")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Volume) => {
            effect.set_parameters(&raw::BASS_BFX_VOLUME { lChannel: integer(value, "lChannel")?, fVolume: number(value, "fVolume")? })
        }
        EffectKind::BassFx(BassFxEffect::PeakEq) => {
            effect.set_parameters(&raw::BASS_BFX_PEAKEQ { lBand: integer(value, "lBand")?, fBandwidth: number(value, "fBandwidth")?, fQ: number(value, "fQ")?, fCenter: number(value, "fCenter")?, fGain: number(value, "fGain")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Reverb) => {
            effect.set_parameters(&raw::BASS_BFX_REVERB { fLevel: number(value, "fLevel")?, lDelay: integer(value, "lDelay")? })
        }
        EffectKind::BassFx(BassFxEffect::LowPassFilter) => {
            effect.set_parameters(&raw::BASS_BFX_LPF { fResonance: number(value, "fResonance")?, fCutOffFreq: number(value, "fCutOffFreq")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Mix) => {
            let channels = integers(value, "lChannel")?;
            let parameters = raw::BASS_BFX_MIX { lChannel: channels.as_ptr() };
            effect.set_parameters(&parameters)
        }
        EffectKind::BassFx(BassFxEffect::Damp) => {
            effect.set_parameters(&raw::BASS_BFX_DAMP { fTarget: number(value, "fTarget")?, fQuiet: number(value, "fQuiet")?, fRate: number(value, "fRate")?, fGain: number(value, "fGain")?, fDelay: number(value, "fDelay")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::AutoWah) => {
            effect.set_parameters(&raw::BASS_BFX_AUTOWAH { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fRate: number(value, "fRate")?, fRange: number(value, "fRange")?, fFreq: number(value, "fFreq")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Phaser) => {
            effect.set_parameters(&raw::BASS_BFX_PHASER { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fRate: number(value, "fRate")?, fRange: number(value, "fRange")?, fFreq: number(value, "fFreq")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Echo2) => {
            effect.set_parameters(&raw::BASS_BFX_ECHO2 { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fDelay: number(value, "fDelay")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Compressor2) => {
            effect.set_parameters(&raw::BASS_BFX_COMPRESSOR2 { fGain: number(value, "fGain")?, fThreshold: number(value, "fThreshold")?, fRatio: number(value, "fRatio")?, fAttack: number(value, "fAttack")?, fRelease: number(value, "fRelease")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Echo3) => {
            effect.set_parameters(&raw::BASS_BFX_ECHO3 { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fDelay: number(value, "fDelay")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Chorus) => {
            effect.set_parameters(&raw::BASS_BFX_CHORUS { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fMinSweep: number(value, "fMinSweep")?, fMaxSweep: number(value, "fMaxSweep")?, fRate: number(value, "fRate")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::AllPassFilter) => {
            effect.set_parameters(&raw::BASS_BFX_APF { fGain: number(value, "fGain")?, fDelay: number(value, "fDelay")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Compressor) => {
            effect.set_parameters(&raw::BASS_BFX_COMPRESSOR { fThreshold: number(value, "fThreshold")?, fAttacktime: number(value, "fAttacktime")?, fReleasetime: number(value, "fReleasetime")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Distortion) => {
            effect.set_parameters(&raw::BASS_BFX_DISTORTION { fDrive: number(value, "fDrive")?, fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fVolume: number(value, "fVolume")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::VolumeEnvelope) => {
            let nodes = env_nodes(value, "pNodes")?;
            let parameters = raw::BASS_BFX_VOLUME_ENV { lChannel: integer(value, "lChannel")?, lNodeCount: integer(value, "lNodeCount")?, pNodes: nodes.as_ptr(), bFollow: integer(value, "bFollow")? };
            effect.set_parameters(&parameters)
        }
        EffectKind::BassFx(BassFxEffect::BiquadFilter) => {
            effect.set_parameters(&raw::BASS_BFX_BQF { lFilter: integer(value, "lFilter")?, fCenter: number(value, "fCenter")?, fGain: number(value, "fGain")?, fBandwidth: number(value, "fBandwidth")?, fQ: number(value, "fQ")?, fS: number(value, "fS")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::Echo4) => {
            effect.set_parameters(&raw::BASS_BFX_ECHO4 { fDryMix: number(value, "fDryMix")?, fWetMix: number(value, "fWetMix")?, fFeedback: number(value, "fFeedback")?, fDelay: number(value, "fDelay")?, bStereo: integer(value, "bStereo")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::BassFx(BassFxEffect::PitchShift) => {
            effect.set_parameters(&raw::BASS_BFX_PITCHSHIFT { fPitchShift: number(value, "fPitchShift")?, fSemitones: number(value, "fSemitones")?, lFFTsize: integer(value, "lFFTsize")?, lOsamp: integer(value, "lOsamp")?, lChannel: integer(value, "lChannel")? })
        }
        EffectKind::Volume => {
            effect.set_parameters(&raw::BASS_FX_VOLUME_PARAM { fTarget: number(value, "fTarget")?, fCurrent: number(value, "fCurrent")?, fTime: number(value, "fTime")?, lCurve: unsigned(value, "lCurve")? })
        }
        _ => Err(BassError::Unsupported { operation: "effect parameter mapping for this raw structure" }),
    }
}

fn get_effect_parameters(effect: &Effect, kind: EffectKind) -> bass_rs::Result<Value> {
    match kind {
        EffectKind::Dx8(raw::BASS_FX_DX8_PARAMEQ) => { let p = effect.get_parameters::<raw::BASS_DX8_PARAMEQ>()?; Ok(json!({ "fCenter": p.fCenter, "fBandwidth": p.fBandwidth, "fGain": p.fGain })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_CHORUS) => { let p = effect.get_parameters::<raw::BASS_DX8_CHORUS>()?; Ok(json!({ "fWetDryMix": p.fWetDryMix, "fDepth": p.fDepth, "fFeedback": p.fFeedback, "fFrequency": p.fFrequency, "lWaveform": p.lWaveform, "fDelay": p.fDelay, "lPhase": p.lPhase })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_COMPRESSOR) => { let p = effect.get_parameters::<raw::BASS_DX8_COMPRESSOR>()?; Ok(json!({ "fGain": p.fGain, "fAttack": p.fAttack, "fRelease": p.fRelease, "fThreshold": p.fThreshold, "fRatio": p.fRatio, "fPredelay": p.fPredelay })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_DISTORTION) => { let p = effect.get_parameters::<raw::BASS_DX8_DISTORTION>()?; Ok(json!({ "fGain": p.fGain, "fEdge": p.fEdge, "fPostEQCenterFrequency": p.fPostEQCenterFrequency, "fPostEQBandwidth": p.fPostEQBandwidth, "fPreLowpassCutoff": p.fPreLowpassCutoff })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_ECHO) => { let p = effect.get_parameters::<raw::BASS_DX8_ECHO>()?; Ok(json!({ "fWetDryMix": p.fWetDryMix, "fFeedback": p.fFeedback, "fLeftDelay": p.fLeftDelay, "fRightDelay": p.fRightDelay, "lPanDelay": p.lPanDelay })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_FLANGER) => { let p = effect.get_parameters::<raw::BASS_DX8_FLANGER>()?; Ok(json!({ "fWetDryMix": p.fWetDryMix, "fDepth": p.fDepth, "fFeedback": p.fFeedback, "fFrequency": p.fFrequency, "lWaveform": p.lWaveform, "fDelay": p.fDelay, "lPhase": p.lPhase })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_GARGLE) => { let p = effect.get_parameters::<raw::BASS_DX8_GARGLE>()?; Ok(json!({ "dwRateHz": p.dwRateHz, "dwWaveShape": p.dwWaveShape })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_I3DL2REVERB) => { let p = effect.get_parameters::<raw::BASS_DX8_I3DL2REVERB>()?; Ok(json!({ "lRoom": p.lRoom, "lRoomHF": p.lRoomHF, "flRoomRolloffFactor": p.flRoomRolloffFactor, "flDecayTime": p.flDecayTime, "flDecayHFRatio": p.flDecayHFRatio, "lReflections": p.lReflections, "flReflectionsDelay": p.flReflectionsDelay, "lReverb": p.lReverb, "flReverbDelay": p.flReverbDelay, "flDiffusion": p.flDiffusion, "flDensity": p.flDensity, "flHFReference": p.flHFReference })) }
        EffectKind::Dx8(raw::BASS_FX_DX8_REVERB) => { let p = effect.get_parameters::<raw::BASS_DX8_REVERB>()?; Ok(json!({ "fInGain": p.fInGain, "fReverbMix": p.fReverbMix, "fReverbTime": p.fReverbTime, "fHighFreqRTRatio": p.fHighFreqRTRatio })) }
        EffectKind::BassFx(BassFxEffect::Rotate) => { let p = effect.get_parameters::<raw::BASS_BFX_ROTATE>()?; Ok(json!({ "fRate": p.fRate, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Echo) => { let p = effect.get_parameters::<raw::BASS_BFX_ECHO>()?; Ok(json!({ "fLevel": p.fLevel, "lDelay": p.lDelay })) }
        EffectKind::BassFx(BassFxEffect::Flanger) => { let p = effect.get_parameters::<raw::BASS_BFX_FLANGER>()?; Ok(json!({ "fWetDry": p.fWetDry, "fSpeed": p.fSpeed, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Volume) => { let p = effect.get_parameters::<raw::BASS_BFX_VOLUME>()?; Ok(json!({ "lChannel": p.lChannel, "fVolume": p.fVolume })) }
        EffectKind::BassFx(BassFxEffect::PeakEq) => { let p = effect.get_parameters::<raw::BASS_BFX_PEAKEQ>()?; Ok(json!({ "lBand": p.lBand, "fBandwidth": p.fBandwidth, "fQ": p.fQ, "fCenter": p.fCenter, "fGain": p.fGain, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Reverb) => { let p = effect.get_parameters::<raw::BASS_BFX_REVERB>()?; Ok(json!({ "fLevel": p.fLevel, "lDelay": p.lDelay })) }
        EffectKind::BassFx(BassFxEffect::LowPassFilter) => { let p = effect.get_parameters::<raw::BASS_BFX_LPF>()?; Ok(json!({ "fResonance": p.fResonance, "fCutOffFreq": p.fCutOffFreq, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Mix) => { let p = effect.get_parameters::<raw::BASS_BFX_MIX>()?; Ok(json!({ "lChannelPointer": p.lChannel as usize })) }
        EffectKind::BassFx(BassFxEffect::Damp) => { let p = effect.get_parameters::<raw::BASS_BFX_DAMP>()?; Ok(json!({ "fTarget": p.fTarget, "fQuiet": p.fQuiet, "fRate": p.fRate, "fGain": p.fGain, "fDelay": p.fDelay, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::AutoWah) => { let p = effect.get_parameters::<raw::BASS_BFX_AUTOWAH>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fRate": p.fRate, "fRange": p.fRange, "fFreq": p.fFreq, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Freeverb) => { let p = effect.get_parameters::<raw::BASS_BFX_FREEVERB>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fRoomSize": p.fRoomSize, "fDamp": p.fDamp, "fWidth": p.fWidth, "lMode": p.lMode, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Phaser) => { let p = effect.get_parameters::<raw::BASS_BFX_PHASER>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fRate": p.fRate, "fRange": p.fRange, "fFreq": p.fFreq, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Echo2) => { let p = effect.get_parameters::<raw::BASS_BFX_ECHO2>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fDelay": p.fDelay, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Echo3) => { let p = effect.get_parameters::<raw::BASS_BFX_ECHO3>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fDelay": p.fDelay, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Chorus) => { let p = effect.get_parameters::<raw::BASS_BFX_CHORUS>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fMinSweep": p.fMinSweep, "fMaxSweep": p.fMaxSweep, "fRate": p.fRate, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::AllPassFilter) => { let p = effect.get_parameters::<raw::BASS_BFX_APF>()?; Ok(json!({ "fGain": p.fGain, "fDelay": p.fDelay, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Compressor) => { let p = effect.get_parameters::<raw::BASS_BFX_COMPRESSOR>()?; Ok(json!({ "fThreshold": p.fThreshold, "fAttacktime": p.fAttacktime, "fReleasetime": p.fReleasetime, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Distortion) => { let p = effect.get_parameters::<raw::BASS_BFX_DISTORTION>()?; Ok(json!({ "fDrive": p.fDrive, "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fVolume": p.fVolume, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Compressor2) => { let p = effect.get_parameters::<raw::BASS_BFX_COMPRESSOR2>()?; Ok(json!({ "fGain": p.fGain, "fThreshold": p.fThreshold, "fRatio": p.fRatio, "fAttack": p.fAttack, "fRelease": p.fRelease, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::VolumeEnvelope) => { let p = effect.get_parameters::<raw::BASS_BFX_VOLUME_ENV>()?; let nodes = if p.pNodes.is_null() || p.lNodeCount <= 0 { Vec::new() } else { unsafe { std::slice::from_raw_parts(p.pNodes, p.lNodeCount as usize) }.iter().map(|node| { let pos = unsafe { std::ptr::addr_of!(node.pos).read_unaligned() }; let val = unsafe { std::ptr::addr_of!(node.val).read_unaligned() }; json!({ "pos": pos, "val": val }) }).collect::<Vec<_>>() }; Ok(json!({ "lChannel": p.lChannel, "lNodeCount": p.lNodeCount, "pNodes": nodes, "bFollow": p.bFollow })) }
        EffectKind::BassFx(BassFxEffect::BiquadFilter) => { let p = effect.get_parameters::<raw::BASS_BFX_BQF>()?; Ok(json!({ "lFilter": p.lFilter, "fCenter": p.fCenter, "fGain": p.fGain, "fBandwidth": p.fBandwidth, "fQ": p.fQ, "fS": p.fS, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::Echo4) => { let p = effect.get_parameters::<raw::BASS_BFX_ECHO4>()?; Ok(json!({ "fDryMix": p.fDryMix, "fWetMix": p.fWetMix, "fFeedback": p.fFeedback, "fDelay": p.fDelay, "bStereo": p.bStereo, "lChannel": p.lChannel })) }
        EffectKind::BassFx(BassFxEffect::PitchShift) => { let p = effect.get_parameters::<raw::BASS_BFX_PITCHSHIFT>()?; Ok(json!({ "fPitchShift": p.fPitchShift, "fSemitones": p.fSemitones, "lFFTsize": p.lFFTsize, "lOsamp": p.lOsamp, "lChannel": p.lChannel })) }
        EffectKind::Volume => { let p = effect.get_parameters::<raw::BASS_FX_VOLUME_PARAM>()?; Ok(json!({ "fTarget": p.fTarget, "fCurrent": p.fCurrent, "fTime": p.fTime, "lCurve": p.lCurve })) }
        _ => Err(BassError::Unsupported { operation: "effect parameter mapping for this raw structure" }),
    }
}

fn number(value: &Value, field: &str) -> bass_rs::Result<f32> {
    value.get(field).and_then(Value::as_f64).map(|value| value as f32).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("missing numeric field {field}") })
}

fn integer(value: &Value, field: &str) -> bass_rs::Result<i32> {
    value.get(field).and_then(Value::as_i64).and_then(|value| i32::try_from(value).ok()).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("missing integer field {field}") })
}

fn integers(value: &Value, field: &str) -> bass_rs::Result<Vec<i32>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("missing integer array {field}") })?
        .iter()
        .map(|value| value.as_i64().and_then(|value| i32::try_from(value).ok()).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("invalid integer array {field}") }))
        .collect()
}

fn env_nodes(value: &Value, field: &str) -> bass_rs::Result<Vec<raw::BASS_BFX_ENV_NODE>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("missing node array {field}") })?
        .iter()
        .map(|node| Ok(raw::BASS_BFX_ENV_NODE {
            pos: node.get("pos").and_then(Value::as_f64).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: "node.pos is required".into() })?,
            val: node.get("val").and_then(Value::as_f64).map(|value| value as f32).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: "node.val is required".into() })?,
        }))
        .collect()
}

fn unsigned(value: &Value, field: &str) -> bass_rs::Result<u32> {
    value.get(field).and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok()).ok_or_else(|| BassError::InvalidInput { kind: "effect parameter", message: format!("missing unsigned field {field}") })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_defaults_match_bass_rs_defaults() {
        let options = source_options(Value::Null).expect("default source options");
        assert!(options.float);
        assert!(options.prescan);
        assert_eq!(options.stream_flags, 0);
    }

    #[test]
    fn all_public_effect_names_parse() {
        for name in [
            "dx8.chorus", "dx8.compressor", "dx8.distortion", "dx8.echo", "dx8.flanger",
            "dx8.gargle", "dx8.i3dl2reverb", "dx8.parameq", "dx8.reverb", "volume",
            "bassFx.rotate", "bassFx.echo", "bassFx.flanger", "bassFx.volume", "bassFx.peakeq",
            "bassFx.reverb", "bassFx.lowpassfilter", "bassFx.mix", "bassFx.damp", "bassFx.autowah",
            "bassFx.echo2", "bassFx.phaser", "bassFx.echo3", "bassFx.chorus", "bassFx.allpassfilter",
            "bassFx.compressor", "bassFx.distortion", "bassFx.compressor2", "bassFx.volumeenvelope",
            "bassFx.biquadfilter", "bassFx.echo4", "bassFx.pitchshift", "bassFx.freeverb",
        ] {
            assert!(parse_effect_kind(name).is_ok(), "effect did not parse: {name}");
        }
    }

    #[test]
    fn raw_catalog_contains_runtime_constants() {
        let catalog = raw_catalog();
        assert_eq!(catalog["bassApiVersion"], json!(bass_rs::BASS_API_VERSION));
        assert_eq!(catalog["constantCount"], json!(283));
        assert!(catalog["constants"]["BASS_ATTRIB_VOL"].is_number());
    }
}
