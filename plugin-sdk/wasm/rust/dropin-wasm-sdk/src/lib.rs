use serde::Deserialize;
use serde_json::{json, Value};

pub const API_VERSION: i32 = 1;
pub type PluginResult = Result<Value, String>;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub method: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Deserialize)]
struct HostResponse {
    ok: bool,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<String>,
}

#[link(wasm_import_module = "dropin")]
extern "C" {
    #[link_name = "host_call"]
    fn raw_host_call(ptr: i32, len: i32) -> i64;
}

#[macro_export]
macro_rules! dropin_plugin {
    ($handler:path) => {
        #[no_mangle]
        pub extern "C" fn plugin_init(api_version: i32) -> i32 {
            if api_version == $crate::API_VERSION {
                0
            } else {
                -1
            }
        }

        #[no_mangle]
        pub extern "C" fn plugin_alloc(len: i32) -> i32 {
            $crate::alloc(len)
        }

        #[no_mangle]
        pub extern "C" fn plugin_dealloc(ptr: i32, len: i32) {
            unsafe { $crate::dealloc(ptr, len) }
        }

        #[no_mangle]
        pub extern "C" fn plugin_call(ptr: i32, len: i32) -> i64 {
            unsafe { $crate::handle_plugin_call(ptr, len, $handler) }
        }

        #[no_mangle]
        pub extern "C" fn plugin_free_response(ptr: i32, len: i32) {
            unsafe { $crate::dealloc(ptr, len) }
        }

        #[no_mangle]
        pub extern "C" fn plugin_shutdown() {}
    };
}

pub fn alloc(len: i32) -> i32 {
    if len < 0 {
        return -1;
    }
    let capacity = len as usize;
    if capacity == 0 {
        return 0;
    }
    let mut buffer = Vec::<u8>::with_capacity(capacity);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr as i32
}

pub unsafe fn dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len <= 0 {
        return;
    }
    let _ = Vec::from_raw_parts(ptr as *mut u8, 0, len as usize);
}

pub unsafe fn handle_plugin_call<F>(ptr: i32, len: i32, handler: F) -> i64
where
    F: FnOnce(Request) -> PluginResult,
{
    let response = match read_request(ptr, len).and_then(handler) {
        Ok(result) => json!({ "ok": true, "result": result }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    write_json(&response)
}

pub mod host {
    use super::*;

    pub fn call(method: &str, args: Value) -> PluginResult {
        let request = serde_json::to_vec(&json!({ "method": method, "args": args }))
            .map_err(|error| error.to_string())?;
        let packed = unsafe { raw_host_call(request.as_ptr() as i32, request.len() as i32) };
        let (ptr, len) = unpack_response(packed);
        if ptr < 0 || len <= 0 {
            return Err("invalid host response".into());
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() };
        unsafe { dealloc(ptr, len) };
        let response: HostResponse = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid host response JSON: {error}"))?;
        if response.ok {
            Ok(response.result.unwrap_or(Value::Null))
        } else {
            Err(response
                .error
                .unwrap_or_else(|| "host call failed".to_string()))
        }
    }

    pub fn player_get_state() -> PluginResult {
        call("player.getState", json!({}))
    }

    pub fn player_play() -> PluginResult {
        call("player.play", json!({}))
    }

    pub fn player_pause() -> PluginResult {
        call("player.pause", json!({}))
    }

    pub fn notification_show(title: &str, body: &str, duration_ms: u64) -> PluginResult {
        call(
            "notification.show",
            json!({ "title": title, "body": body, "duration": duration_ms }),
        )
    }

    pub fn storage_get(key: &str) -> PluginResult {
        call("storage.get", json!({ "key": key }))
    }

    pub fn storage_set(key: &str, value: Value) -> PluginResult {
        call("storage.set", json!({ "key": key, "value": value }))
    }

    pub fn storage_remove(key: &str) -> PluginResult {
        call("storage.remove", json!({ "key": key }))
    }
}

unsafe fn read_request(ptr: i32, len: i32) -> Result<Request, String> {
    if ptr < 0 || len < 0 {
        return Err("invalid request range".into());
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    serde_json::from_slice(bytes).map_err(|error| format!("invalid request JSON: {error}"))
}

fn write_json(value: &Value) -> i64 {
    match serde_json::to_vec(value) {
        Ok(bytes) => write_bytes(bytes),
        Err(error) => write_bytes(
            json!({ "ok": false, "error": format!("response serialization failed: {error}") })
                .to_string()
                .into_bytes(),
        ),
    }
}

fn write_bytes(bytes: Vec<u8>) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let len = bytes.len();
    if len > i32::MAX as usize {
        return 0;
    }
    let mut buffer = bytes.into_boxed_slice();
    let ptr = buffer.as_mut_ptr() as i32;
    std::mem::forget(buffer);
    pack_response(ptr, len)
}

fn pack_response(ptr: i32, len: usize) -> i64 {
    ((ptr as i64) << 32) | (len as i64 & 0xffff_ffff)
}

fn unpack_response(packed: i64) -> (i32, i32) {
    ((packed >> 32) as i32, (packed & 0xffff_ffff) as u32 as i32)
}
