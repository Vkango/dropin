use dropin_wasm_sdk::{dropin_plugin, host, PluginResult, Request};
use serde_json::{json, Value};

const TAG_KEY: &str = "energy";
const READ_WINDOW_MS: u64 = 5_000;
const MAX_SAMPLES: u32 = 2_000;
/// RMS values of decoded float audio rarely exceed 0.5; map 0.0..0.5 to 0..100.
const RMS_FULL_SCALE: f64 = 0.5;

dropin_plugin!(handle);

fn handle(request: Request) -> PluginResult {
    match request
        .method
        .strip_prefix("backend.")
        .unwrap_or(&request.method)
    {
        "tag.analyze" => analyze_entry(request.args),
        "tag.wiki" | "wiki" => wiki_entry(request.args),
        method => Err(format!("unknown energy tag method: {method}")),
    }
}

fn wiki_entry(args: Value) -> PluginResult {
    let locale = args
        .get("locale")
        .and_then(Value::as_str)
        .unwrap_or("en-US");
    let messages = match locale {
        "zh-CLASSICAL" => include_str!("../../i18n/zh-CLASSICAL.json"),
        language if language.to_ascii_lowercase().starts_with("zh") => include_str!("../../i18n/zh-CN.json"),
        _ => include_str!("../../i18n/en-US.json"),
    };
    let messages: Value = serde_json::from_str(messages).map_err(|error| error.to_string())?;
    Ok(json!({ "wiki": messages.get("wiki").and_then(Value::as_str).unwrap_or("") }))
}

/// Called once per track: args = { providerKey, trackIndex, track }.
fn analyze_entry(args: Value) -> PluginResult {
    let track = args.get("track").cloned().unwrap_or(Value::Null);
    let Some(track_id) = track.get("id").and_then(Value::as_str).map(str::to_string) else {
        return Ok(json!({ "results": [] }));
    };
    if track_id.trim().is_empty() {
        return Ok(json!({ "results": [] }));
    }
    // Skip tracks whose audio cannot be read instead of writing a fake value.
    let results = match analyze_track(&track_id, &track) {
        Ok(energy) => vec![json!({
            "trackId": track_id,
            "key": TAG_KEY,
            "value": energy,
        })],
        Err(_) => Vec::new(),
    };
    Ok(json!({ "results": results }))
}

fn analyze_track(track_id: &str, track: &Value) -> Result<f64, String> {
    let duration_ms = track.get("durationMs").and_then(Value::as_u64).unwrap_or(0);
    let start_ms = if duration_ms > READ_WINDOW_MS {
        (duration_ms / 4).min(10_000)
    } else {
        0
    };
    let audio = host::library_audio_read_ex(
        track_id,
        start_ms,
        start_ms + READ_WINDOW_MS,
        Some(MAX_SAMPLES),
    )?;
    let samples = audio
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "audio read returned no samples".to_string())?;
    let mut square_sum = 0.0f64;
    let mut count = 0usize;
    for sample in samples {
        let value = sample.as_f64().unwrap_or(0.0);
        square_sum += value * value;
        count += 1;
    }
    if count == 0 {
        return Err("audio read returned no samples".into());
    }
    let rms = (square_sum / count as f64).sqrt();
    let energy = (rms / RMS_FULL_SCALE).clamp(0.0, 1.0) * 100.0;
    Ok((energy * 10.0).round() / 10.0)
}
