use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};

const EVENT_SMTC: &str = "smtc/event";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtcError {
    pub kind: String,
    pub operation: String,
    pub message: String,
    pub debug: String,
    pub recoverable: bool,
}

impl std::fmt::Display for SmtcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.operation, self.message)
    }
}

impl std::error::Error for SmtcError {}

fn smtc_error(operation: impl Into<String>, message: impl Into<String>) -> SmtcError {
    let operation = operation.into();
    let message = message.into();
    SmtcError {
        kind: "smtc".into(),
        operation,
        debug: message.clone(),
        message,
        recoverable: true,
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{json, smtc_error, AppHandle, Emitter, SmtcError, Value, EVENT_SMTC};
    use smtc_player::{SmtcEvent, SmtcPlayer};
    use std::{sync::mpsc, thread, time::Duration};

    type Reply = mpsc::Sender<Result<Value, SmtcError>>;

    struct Request {
        operation: String,
        args: Value,
        reply: Reply,
    }

    #[derive(Clone)]
    pub struct SmtcService {
        sender: mpsc::Sender<Request>,
    }

    impl SmtcService {
        pub fn new(app: AppHandle) -> Self {
            let (sender, receiver) = mpsc::channel::<Request>();
            thread::Builder::new()
                .name("smtc-player".into())
                .spawn(move || {
                    let mut player = SmtcPlayer::new().ok();
                    let mut initialization_error = None;

                    if player.is_none() {
                        initialization_error = Some("failed to initialize Windows SMTC".into());
                    } else if let Some(current_player) = player.as_ref() {
                        let _ = current_player.set_media_info("Dropin", "Player", "", None);
                        let _ = current_player.set_playback_status(false);
                        let _ = current_player.set_timeline(Duration::ZERO, Duration::ZERO);
                        let event_app = app.clone();
                        if let Err(error) = current_player.on_event(move |event| {
                            let _ = event_app.emit(EVENT_SMTC, event_name(event));
                        }) {
                            initialization_error = Some(format!("{error:?}"));
                        }
                    }

                    while let Ok(request) = receiver.recv() {
                        let should_close = request.operation == "smtc_close";
                        let result = if let Some(error) = initialization_error.as_ref() {
                            Err(smtc_error(&request.operation, error.clone()))
                        } else if let Some(current_player) = player.as_ref() {
                            dispatch(current_player, &request.operation, request.args)
                        } else {
                            Err(smtc_error(&request.operation, "SMTC player is unavailable"))
                        };
                        let _ = request.reply.send(result);
                        if should_close {
                            break;
                        }
                    }

                    if let Some(current_player) = player.take() {
                        let _ = current_player.close();
                    }
                })
                .expect("failed to start SMTC worker thread");
            Self { sender }
        }

        pub(crate) fn call_operation(
            &self,
            operation: &str,
            args: Value,
        ) -> Result<Value, SmtcError> {
            let (reply, receiver) = mpsc::channel();
            self.sender
                .send(Request {
                    operation: operation.into(),
                    args,
                    reply,
                })
                .map_err(|_| smtc_error(operation, "SMTC worker thread is not running"))?;
            receiver
                .recv()
                .map_err(|_| smtc_error(operation, "SMTC worker dropped the response"))?
        }
    }

    fn dispatch(player: &SmtcPlayer, operation: &str, args: Value) -> Result<Value, SmtcError> {
        match operation {
            "smtc_set_media_info" => {
                let title = args.get("title").and_then(Value::as_str).unwrap_or("");
                let artist = args.get("artist").and_then(Value::as_str).unwrap_or("");
                let album = args.get("album").and_then(Value::as_str).unwrap_or("");
                let thumbnail_path = args.get("thumbnailPath").and_then(Value::as_str);
                player
                    .set_media_info(title, artist, album, thumbnail_path)
                    .map_err(|error| smtc_error(operation, format!("{error:?}")))?;
                Ok(json!({ "updated": true }))
            }
            "smtc_set_playback_status" => {
                let playing = args
                    .get("playing")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                player
                    .set_playback_status(playing)
                    .map_err(|error| smtc_error(operation, format!("{error:?}")))?;
                Ok(json!({ "playing": playing }))
            }
            "smtc_set_timeline" => {
                let position_ms = args.get("positionMs").and_then(Value::as_u64).unwrap_or(0);
                let duration_ms = args.get("durationMs").and_then(Value::as_u64).unwrap_or(0);
                player
                    .set_timeline(
                        Duration::from_millis(position_ms),
                        Duration::from_millis(duration_ms),
                    )
                    .map_err(|error| smtc_error(operation, format!("{error:?}")))?;
                Ok(json!({ "positionMs": position_ms, "durationMs": duration_ms }))
            }
            "smtc_close" => Ok(json!({ "closed": true })),
            _ => Err(smtc_error(operation, "unknown SMTC operation")),
        }
    }

    fn event_name(event: SmtcEvent) -> &'static str {
        match event {
            SmtcEvent::Play => "play",
            SmtcEvent::Pause => "pause",
            SmtcEvent::Previous => "previous",
            SmtcEvent::Next => "next",
            SmtcEvent::Stop => "stop",
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::SmtcService;

#[cfg(not(target_os = "windows"))]
#[derive(Clone)]
pub struct SmtcService;

#[cfg(not(target_os = "windows"))]
impl SmtcService {
    pub fn new(_: AppHandle) -> Self {
        Self
    }

    pub(crate) fn call_operation(&self, operation: &str, _: Value) -> Result<Value, SmtcError> {
        Err(smtc_error(operation, "SMTC is only available on Windows"))
    }
}

#[tauri::command]
pub fn smtc_call(
    service: State<'_, SmtcService>,
    operation: String,
    args: Value,
) -> Result<Value, SmtcError> {
    service.call_operation(&operation, args)
}
