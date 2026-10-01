use dropin_wasm_sdk::{dropin_plugin, host, AudioRange, PluginResult, Request};
use serde_json::{json, Value};

const TAG_KEY: &str = "energy";
/// 整曲均匀取样的位置数：把全曲均分为多段，一次流式调用内逐段读取汇总，避免只测前奏造成不公平
const WINDOW_COUNT: u32 = 8;
const READ_WINDOW_MS: u64 = 5_000;
const SAMPLES_PER_WINDOW: u32 = 5_000;
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
    // 整曲公平取样：在全曲均匀分布的多个位置各读一小段，汇总所有采样计算整体 RMS。
    let mut starts: Vec<u64> = if duration_ms > READ_WINDOW_MS {
        (0..WINDOW_COUNT)
            .map(|index| {
                let raw = duration_ms * u64::from(index) / u64::from(WINDOW_COUNT);
                // 保证窗口完整落在曲长之内
                raw.min(duration_ms - READ_WINDOW_MS)
            })
            .collect()
    } else {
        vec![0]
    };
    starts.sort_unstable();
    starts.dedup();
    // 一次流式调用读取全部区间：host 只打开一次文件
    let ranges: Vec<AudioRange> = starts
        .iter()
        .map(|&start_ms| AudioRange {
            start_ms,
            end_ms: start_ms + READ_WINDOW_MS,
            max_samples: SAMPLES_PER_WINDOW,
        })
        .collect();
    let audio = host::library_audio_read_ranges(track_id, &ranges)?;
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
