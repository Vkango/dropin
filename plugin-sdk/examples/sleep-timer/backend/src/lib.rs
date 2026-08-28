use dropin_wasm_sdk::{dropin_plugin, host, PluginResult, Request};
use serde_json::{json, Value};
use std::sync::Mutex;

const MIN_DURATION_MS: u64 = 1_000;
const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Clone, Copy)]
struct TimerState {
    active: bool,
    started_at_ms: u64,
    deadline_ms: u64,
    duration_ms: u64,
}

impl TimerState {
    const fn empty() -> Self {
        Self {
            active: false,
            started_at_ms: 0,
            deadline_ms: 0,
            duration_ms: 0,
        }
    }
}

static TIMER: Mutex<TimerState> = Mutex::new(TimerState::empty());

dropin_plugin!(handle);

fn handle(request: Request) -> PluginResult {
    match request
        .method
        .strip_prefix("backend.")
        .unwrap_or(&request.method)
    {
        "start" => start_timer(request.args),
        "cancel" => cancel_timer(),
        "state" => read_state(request.args),
        "tick" => tick(request.args),
        method => Err(format!("unknown sleep timer method: {method}")),
    }
}

fn start_timer(args: Value) -> PluginResult {
    let now_ms = number_arg(&args, "nowMs")?;
    let duration_ms = duration_arg(&args)?;
    let state = TimerState {
        active: true,
        started_at_ms: now_ms,
        deadline_ms: now_ms.saturating_add(duration_ms),
        duration_ms,
    };
    *TIMER
        .lock()
        .map_err(|_| "timer state poisoned".to_string())? = state;
    Ok(state_value(state, now_ms, false))
}

fn cancel_timer() -> PluginResult {
    let mut timer = TIMER
        .lock()
        .map_err(|_| "timer state poisoned".to_string())?;
    *timer = TimerState::empty();
    Ok(state_value(*timer, 0, false))
}

fn read_state(args: Value) -> PluginResult {
    let now_ms = number_arg(&args, "nowMs").unwrap_or(0);
    let state = *TIMER
        .lock()
        .map_err(|_| "timer state poisoned".to_string())?;
    Ok(state_value(state, now_ms, false))
}

fn tick(args: Value) -> PluginResult {
    let now_ms = number_arg(&args, "nowMs")?;
    {
        let mut timer = TIMER
            .lock()
            .map_err(|_| "timer state poisoned".to_string())?;
        if !timer.active || now_ms < timer.deadline_ms {
            return Ok(state_value(*timer, now_ms, false));
        }
        *timer = TimerState::empty();
    }

    host::player_pause()?;
    host::notification_show(
        "Sleep timer finished",
        "Playback has been paused automatically.",
        8000,
    )?;

    Ok(state_value(TimerState::empty(), now_ms, true))
}

fn state_value(state: TimerState, now_ms: u64, paused: bool) -> Value {
    let remaining_ms = if state.active {
        state.deadline_ms.saturating_sub(now_ms)
    } else {
        0
    };
    json!({
        "active": state.active,
        "startedAtMs": state.started_at_ms,
        "deadlineMs": state.deadline_ms,
        "durationMs": state.duration_ms,
        "remainingMs": remaining_ms,
        "paused": paused
    })
}

fn duration_arg(args: &Value) -> Result<u64, String> {
    let duration_ms = args
        .get("durationMs")
        .and_then(Value::as_u64)
        .or_else(|| {
            args.get("seconds")
                .and_then(Value::as_u64)
                .map(|value| value.saturating_mul(1_000))
        })
        .or_else(|| {
            args.get("minutes")
                .and_then(Value::as_u64)
                .map(|value| value.saturating_mul(60_000))
        })
        .ok_or_else(|| "durationMs is required".to_string())?;
    Ok(duration_ms.clamp(MIN_DURATION_MS, MAX_DURATION_MS))
}

fn number_arg(args: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{key} is required"))
}
