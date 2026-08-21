use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use lofty::{
    file::{AudioFile, TaggedFileExt},
    prelude::Accessor,
    read_from_path,
    tag::ItemKey,
};
use reqwest::blocking::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::bass_bridge::{BassService, BridgeError};
use crate::lyrics::LyricsPayload;

const EVENT_SCAN_PROGRESS: &str = "media/scan-progress";
const EVENT_TRACK_ADDED: &str = "media/track-added";
const EVENT_TRACK_UPDATED: &str = "media/track-updated";
const EVENT_METADATA_UPDATED: &str = "media/metadata-updated";
const EVENT_SCAN_FINISHED: &str = "media/scan-finished";
const EVENT_ERROR: &str = "media/error";
const MAX_URL_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const MAX_COVER_BYTES: u64 = 16 * 1024 * 1024;
const CACHE_SOFT_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaError {
    pub kind: String,
    pub operation: String,
    pub message: String,
    pub debug: String,
    pub recoverable: bool,
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.operation, self.message)
    }
}

impl std::error::Error for MediaError {}

fn media_error(operation: impl Into<String>, message: impl Into<String>) -> MediaError {
    let operation = operation.into();
    let message = message.into();
    MediaError {
        kind: "media".into(),
        operation,
        debug: message.clone(),
        message,
        recoverable: true,
    }
}

fn fatal_media_error(operation: impl Into<String>, message: impl Into<String>) -> MediaError {
    let mut error = media_error(operation, message);
    error.recoverable = false;
    error
}

fn sqlite_error(operation: &str, error: rusqlite::Error) -> MediaError {
    media_error(operation, error.to_string())
}

fn io_error(operation: &str, error: std::io::Error) -> MediaError {
    media_error(operation, error.to_string())
}

fn parse_error(operation: &str, error: impl std::fmt::Debug + std::fmt::Display) -> MediaError {
    MediaError {
        kind: "metadata".into(),
        operation: operation.into(),
        message: error.to_string(),
        debug: format!("{error:?}"),
        recoverable: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaTrack {
    pub id: String,
    pub source: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genres: Vec<String>,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub track_total: Option<i32>,
    pub disc_number: Option<i32>,
    pub disc_total: Option<i32>,
    pub duration_ms: u64,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub codec: Option<String>,
    pub format: Option<String>,
    pub cover_id: Option<String>,
    pub cover_mime_type: Option<String>,
    pub warnings: Vec<String>,
    pub added_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub last_played_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRoot {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub added_at: i64,
    pub last_scan_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanJob {
    pub id: String,
    pub state: String,
    pub root_ids: Vec<i64>,
    pub scanned: u64,
    pub imported: u64,
    pub skipped: u64,
    pub failed: u64,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverPayload {
    pub cover_id: String,
    pub mime_type: String,
    pub data_base64: String,
}

struct MediaRequest {
    operation: String,
    args: Value,
    reply: Sender<Result<Value, MediaError>>,
}

pub struct MediaService {
    sender: Sender<MediaRequest>,
}

impl MediaService {
    pub fn new(app: AppHandle) -> Self {
        let (sender, receiver) = mpsc::channel::<MediaRequest>();
        let worker_sender = sender.clone();
        thread::Builder::new()
            .name("media-db".into())
            .spawn(move || {
                let mut runtime = MediaRuntime::new(app, worker_sender);
                while let Ok(request) = receiver.recv() {
                    let result = runtime.dispatch(&request.operation, request.args);
                    let _ = request.reply.send(result);
                }
            })
            .expect("failed to start media database worker thread");
        Self { sender }
    }

    fn call(&self, operation: &str, args: Value) -> Result<Value, MediaError> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(MediaRequest {
                operation: operation.into(),
                args,
                reply,
            })
            .map_err(|_| fatal_media_error(operation, "media database worker is not running"))?;
        receiver.recv().map_err(|_| {
            fatal_media_error(operation, "media database worker dropped the response")
        })?
    }
}

struct MediaRuntime {
    app: AppHandle,
    sender: Sender<MediaRequest>,
    db_path: PathBuf,
    cache_dir: PathBuf,
    connection: Option<Connection>,
    scan_flags: HashMap<String, Arc<AtomicBool>>,
}

impl MediaRuntime {
    fn new(app: AppHandle, sender: Sender<MediaRequest>) -> Self {
        let app_data = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));
        let cache_dir = app
            .path()
            .app_cache_dir()
            .unwrap_or_else(|_| app_data.join("cache"))
            .join("metadata-cache");
        Self {
            app,
            sender,
            db_path: app_data.join("dropin.sqlite3"),
            cache_dir,
            connection: None,
            scan_flags: HashMap::new(),
        }
    }

    fn connection(&mut self, operation: &str) -> Result<&mut Connection, MediaError> {
        if self.connection.is_none() {
            if let Some(parent) = self.db_path.parent() {
                fs::create_dir_all(parent).map_err(|error| io_error(operation, error))?;
            }
            fs::create_dir_all(&self.cache_dir).map_err(|error| io_error(operation, error))?;
            let connection =
                Connection::open(&self.db_path).map_err(|error| sqlite_error(operation, error))?;
            connection
                .busy_timeout(Duration::from_secs(5))
                .map_err(|error| sqlite_error(operation, error))?;
            connection
                .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
                .map_err(|error| sqlite_error(operation, error))?;
            migrate(&connection).map_err(|error| sqlite_error(operation, error))?;
            self.connection = Some(connection);
        }
        self.connection
            .as_mut()
            .ok_or_else(|| fatal_media_error(operation, "media database connection is unavailable"))
    }

    fn dispatch(&mut self, operation: &str, args: Value) -> Result<Value, MediaError> {
        match operation {
            "media_metadata_read_file" => self.metadata_read_file(args),
            "media_metadata_read_url" => self.metadata_read_url(args),
            "media_lyrics_read" => self.lyrics_read(args),
            "media_library_add_root" => self.add_root(args),
            "media_library_remove_root" => self.remove_root(args),
            "media_library_roots" => self.roots(),
            "media_library_scan" => self.start_scan(args),
            "media_library_cancel_scan" => self.cancel_scan(args),
            "media_library_tracks" => self.tracks(args),
            "media_library_albums" => self.albums(args),
            "media_library_artists" => self.artists(args),
            "media_library_refresh_track" => self.refresh_track(args),
            "media_library_remove_track" => self.remove_track(args),
            "media_cover_get" => self.cover_get(args),
            "media_cover_path" => self.cover_path(args),
            "media_playback_history" => self.playback_history(args),
            "media_playback_record" => self.playback_record(args),
            "media_pick_folder" => self.pick_folder(),
            "media_should_refresh" => self.should_refresh(args),
            "media_mark_seen" => self.mark_seen(args),
            "media_upsert_track" => self.upsert_track_value(args),
            "media_scan_cleanup" => self.scan_cleanup(args),
            "media_url_cache_touch" => self.url_cache_touch(args),
            "media_track_source" => self.track_source(args),
            _ => Err(media_error(operation, "unknown media operation")),
        }
    }

    fn metadata_read_file(&mut self, args: Value) -> Result<Value, MediaError> {
        let path = required_string(&args, "path", "media_metadata_read_file")?;
        let parsed = parse_audio_file(Path::new(&path), &self.cache_dir)?;
        serde_json::to_value(parsed)
            .map_err(|error| media_error("media_metadata_read_file", error.to_string()))
    }

    fn lyrics_read(&mut self, args: Value) -> Result<Value, MediaError> {
        let track_id = required_string(&args, "trackId", "media_lyrics_read")?;
        let source: Option<(String, Option<String>, Option<String>)> = self
            .connection("media_lyrics_read")?
            .query_row(
                "SELECT source, path, url FROM tracks WHERE id = ?1 AND missing = 0",
                params![track_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| sqlite_error("media_lyrics_read", error))?;

        let Some((source_kind, path, url)) = source else {
            return Err(media_error(
                "media_lyrics_read",
                "track is not in the library",
            ));
        };

        let (path, allow_sidecar) = if source_kind == "url" {
            let cached_path: Option<String> = url.as_deref().and_then(|url| {
                self.connection("media_lyrics_read")
                    .ok()
                    .and_then(|connection| {
                        connection
                            .query_row(
                                "SELECT local_path FROM url_cache WHERE url = ?1",
                                params![url],
                                |row| row.get(0),
                            )
                            .optional()
                            .ok()
                            .flatten()
                    })
            });
            (cached_path, false)
        } else {
            (path, true)
        };

        let payload = match path {
            Some(path) if Path::new(&path).is_file() => {
                crate::lyrics::read_for_audio_path(Path::new(&path), allow_sidecar)
                    .map_err(|error| media_error("media_lyrics_read", error))?
            }
            _ => LyricsPayload {
                source: "none".into(),
                warnings: if source_kind == "url" {
                    vec!["cached audio is unavailable; embedded lyrics were not read".into()]
                } else {
                    Vec::new()
                },
                ..Default::default()
            },
        };

        serde_json::to_value(payload)
            .map_err(|error| media_error("media_lyrics_read", error.to_string()))
    }

    fn metadata_read_url(&mut self, args: Value) -> Result<Value, MediaError> {
        let url = required_string(&args, "url", "media_metadata_read_url")?;
        validate_url(&url)?;
        let track = provisional_url_track(&url);
        self.upsert_track(&track, None, None)?;
        let cached_path: Option<String> = self
            .connection("media_metadata_read_url")?
            .query_row(
                "SELECT local_path FROM url_cache WHERE url = ?1",
                params![url],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| sqlite_error("media_metadata_read_url", error))?;
        let job_id = track.id.clone();
        let app = self.app.clone();
        let sender = self.sender.clone();
        let cache_dir = self.cache_dir.clone();
        thread::Builder::new()
            .name("media-url-metadata".into())
            .spawn(move || {
                let result = cached_path
                    .filter(|path| Path::new(path).is_file())
                    .map(|path| {
                        let mut parsed = parse_audio_file(Path::new(&path), &cache_dir)?;
                        parsed.source = "url".into();
                        parsed.path = None;
                        parsed.url = Some(url.clone());
                        let size = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
                        Ok((parsed, PathBuf::from(path), None, None, None, size))
                    })
                    .unwrap_or_else(|| download_and_parse_url(&url, &cache_dir));
                match result {
                    Ok((mut parsed, cache_path, etag, last_modified, content_type, size)) => {
                        parsed.id = job_id.clone();
                        parsed.source = "url".into();
                        parsed.url = Some(url.clone());
                        parsed.path = None;
                        let args = json!({
                            "track": parsed,
                            "urlCache": {
                                "url": url,
                                "localPath": cache_path.to_string_lossy(),
                                "etag": etag,
                                "lastModified": last_modified,
                                "contentType": content_type,
                                "contentLength": size
                            }
                        });
                        let (reply, receiver) = mpsc::channel();
                        let _ = sender.send(MediaRequest {
                            operation: "media_upsert_track".into(),
                            args,
                            reply,
                        });
                        let _ = receiver.recv();
                        let _ = app.emit(
                            EVENT_METADATA_UPDATED,
                            json!({ "trackId": job_id, "source": "url" }),
                        );
                    }
                    Err(error) => {
                        let _ = app.emit(
                            EVENT_ERROR,
                            json!({
                                "operation": "media_metadata_read_url",
                                "trackId": job_id,
                                "url": url,
                                "error": error
                            }),
                        );
                    }
                }
            })
            .map_err(|error| media_error("media_metadata_read_url", error.to_string()))?;
        serde_json::to_value(track)
            .map_err(|error| media_error("media_metadata_read_url", error.to_string()))
    }

    fn add_root(&mut self, args: Value) -> Result<Value, MediaError> {
        let path = canonical_directory(required_string(&args, "path", "media_library_add_root")?)?;
        let now = now_ms();
        let connection = self.connection("media_library_add_root")?;
        connection
            .execute(
                "INSERT INTO library_roots(path, enabled, added_at) VALUES(?1, 1, ?2)
                 ON CONFLICT(path) DO UPDATE SET enabled = 1",
                params![path, now],
            )
            .map_err(|error| sqlite_error("media_library_add_root", error))?;
        let id: i64 = connection
            .query_row(
                "SELECT id FROM library_roots WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .map_err(|error| sqlite_error("media_library_add_root", error))?;
        Ok(
            json!({ "root": LibraryRoot { id, path, enabled: true, added_at: now, last_scan_at: None } }),
        )
    }

    fn remove_root(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_i64(&args, "rootId", "media_library_remove_root")?;
        let connection = self.connection("media_library_remove_root")?;
        let path: Option<String> = connection
            .query_row(
                "SELECT path FROM library_roots WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| sqlite_error("media_library_remove_root", error))?;
        if let Some(path) = path {
            connection
                .execute(
                    "DELETE FROM tracks WHERE source = 'file' AND path LIKE ?1",
                    params![format!("{path}%")],
                )
                .map_err(|error| sqlite_error("media_library_remove_root", error))?;
        }
        let deleted = connection
            .execute("DELETE FROM library_roots WHERE id = ?1", params![id])
            .map_err(|error| sqlite_error("media_library_remove_root", error))?;
        Ok(json!({ "rootId": id, "removed": deleted > 0 }))
    }

    fn roots(&mut self) -> Result<Value, MediaError> {
        let connection = self.connection("media_library_roots")?;
        let mut statement = connection
            .prepare(
                "SELECT id, path, enabled, added_at, last_scan_at FROM library_roots ORDER BY path",
            )
            .map_err(|error| sqlite_error("media_library_roots", error))?;
        let rows = statement
            .query_map([], |row| {
                Ok(LibraryRoot {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    enabled: row.get::<_, i64>(2)? != 0,
                    added_at: row.get(3)?,
                    last_scan_at: row.get(4)?,
                })
            })
            .map_err(|error| sqlite_error("media_library_roots", error))?;
        let roots = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| sqlite_error("media_library_roots", error))?;
        Ok(json!({ "roots": roots }))
    }

    fn start_scan(&mut self, args: Value) -> Result<Value, MediaError> {
        let requested_root_ids = args
            .get("rootIds")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_i64).collect::<Vec<_>>());
        let roots = self.load_roots(requested_root_ids)?;
        let job_id = format!("scan-{}", now_ms());
        let cancel = Arc::new(AtomicBool::new(false));
        self.scan_flags.insert(job_id.clone(), cancel.clone());
        let app = self.app.clone();
        let sender = self.sender.clone();
        let cache_dir = self.cache_dir.clone();
        let root_ids = roots.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let thread_job_id = job_id.clone();
        let thread_root_ids = root_ids.clone();
        thread::Builder::new()
            .name("media-scan".into())
            .spawn(move || {
                run_scan(
                    app,
                    sender,
                    cache_dir,
                    thread_job_id,
                    roots,
                    thread_root_ids,
                    cancel,
                );
            })
            .map_err(|error| media_error("media_library_scan", error.to_string()))?;
        Ok(
            json!({ "job": ScanJob { id: job_id, state: "running".into(), root_ids, scanned: 0, imported: 0, skipped: 0, failed: 0, started_at: now_ms(), finished_at: None, error: None } }),
        )
    }

    fn load_roots(
        &mut self,
        root_ids: Option<Vec<i64>>,
    ) -> Result<Vec<(i64, PathBuf)>, MediaError> {
        let connection = self.connection("media_library_scan")?;
        let mut result = Vec::new();
        if let Some(ids) = root_ids {
            for id in ids {
                if let Some(path) = connection
                    .query_row(
                        "SELECT path FROM library_roots WHERE id = ?1 AND enabled = 1",
                        params![id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(|error| sqlite_error("media_library_scan", error))?
                {
                    result.push((id, PathBuf::from(path)));
                }
            }
        } else {
            let mut statement = connection
                .prepare("SELECT id, path FROM library_roots WHERE enabled = 1 ORDER BY path")
                .map_err(|error| sqlite_error("media_library_scan", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get(0)?, PathBuf::from(row.get::<_, String>(1)?)))
                })
                .map_err(|error| sqlite_error("media_library_scan", error))?;
            result = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| sqlite_error("media_library_scan", error))?;
        }
        Ok(result)
    }

    fn cancel_scan(&mut self, args: Value) -> Result<Value, MediaError> {
        let job_id = required_string(&args, "jobId", "media_library_cancel_scan")?;
        let cancelled = self
            .scan_flags
            .get(&job_id)
            .map(|flag| {
                flag.store(true, Ordering::Release);
                true
            })
            .unwrap_or(false);
        Ok(json!({ "jobId": job_id, "cancelled": cancelled }))
    }

    fn tracks(&mut self, args: Value) -> Result<Value, MediaError> {
        let limit = args
            .get("limit")
            .and_then(Value::as_i64)
            .unwrap_or(500)
            .clamp(1, 2000);
        let offset = args
            .get("offset")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0);
        let search = args
            .get("search")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned();
        let connection = self.connection("media_library_tracks")?;
        let mut statement = connection
            .prepare(
                "SELECT id, source, path, url, title, artist, album, album_artist, composer,
                        genres_json, year, track_number, track_total, disc_number, disc_total,
                        duration_ms, bitrate, sample_rate, channels, codec, format, cover_id,
                        cover_mime_type, warnings_json, added_at, updated_at, last_played_at
                 FROM tracks
                 WHERE missing = 0 AND (?1 = '' OR title LIKE '%' || ?1 || '%' OR artist LIKE '%' || ?1 || '%' OR album LIKE '%' || ?1 || '%')
                 ORDER BY COALESCE(album, ''), disc_number IS NULL, disc_number, track_number IS NULL, track_number, title
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|error| sqlite_error("media_library_tracks", error))?;
        let rows = statement
            .query_map(params![search, limit, offset], row_to_track)
            .map_err(|error| sqlite_error("media_library_tracks", error))?;
        let tracks = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| sqlite_error("media_library_tracks", error))?;
        let total: i64 = connection
            .query_row("SELECT COUNT(*) FROM tracks WHERE missing = 0", [], |row| {
                row.get(0)
            })
            .map_err(|error| sqlite_error("media_library_tracks", error))?;
        Ok(json!({ "tracks": tracks, "total": total, "limit": limit, "offset": offset }))
    }

    fn albums(&mut self, args: Value) -> Result<Value, MediaError> {
        let search = args
            .get("search")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned();
        let connection = self.connection("media_library_albums")?;
        let mut statement = connection
            .prepare(
                "SELECT album, COALESCE(MAX(album_artist), MAX(artist), 'Unknown Artist'),
                        COUNT(*), SUM(duration_ms), MAX(year), MAX(cover_id), MAX(cover_mime_type)
                 FROM tracks WHERE missing = 0 AND album <> ''
                 AND (?1 = '' OR album LIKE '%' || ?1 || '%' OR artist LIKE '%' || ?1 || '%')
                 GROUP BY album, COALESCE(album_artist, artist) ORDER BY album COLLATE NOCASE",
            )
            .map_err(|error| sqlite_error("media_library_albums", error))?;
        let rows = statement
            .query_map(params![search], |row| {
                let title: String = row.get(0)?;
                let artist: String = row.get(1)?;
                let duration: i64 = row.get(3)?;
                Ok(json!({
                    "id": stable_id(&format!("album\n{artist}\n{title}")),
                    "title": title,
                    "artist": artist,
                    "trackCount": row.get::<_, i64>(2)?,
                    "duration": format_duration(duration as u64),
                    "year": row.get::<_, Option<i32>>(4)?,
                    "coverId": row.get::<_, Option<String>>(5)?,
                    "coverMimeType": row.get::<_, Option<String>>(6)?,
                    "genres": Vec::<String>::new(),
                }))
            })
            .map_err(|error| sqlite_error("media_library_albums", error))?;
        let albums = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| sqlite_error("media_library_albums", error))?;
        Ok(json!({ "albums": albums }))
    }

    fn artists(&mut self, args: Value) -> Result<Value, MediaError> {
        let search = args
            .get("search")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned();
        let connection = self.connection("media_library_artists")?;
        let mut statement = connection
            .prepare(
                "SELECT artist, COUNT(DISTINCT NULLIF(album, '')), COUNT(*), MAX(cover_id)
                 FROM tracks WHERE missing = 0 AND artist <> ''
                 AND (?1 = '' OR artist LIKE '%' || ?1 || '%')
                 GROUP BY artist ORDER BY artist COLLATE NOCASE",
            )
            .map_err(|error| sqlite_error("media_library_artists", error))?;
        let rows = statement
            .query_map(params![search], |row| {
                let name: String = row.get(0)?;
                Ok(json!({
                    "id": stable_id(&format!("artist\n{name}")),
                    "name": name,
                    "albumCount": row.get::<_, i64>(1)?,
                    "songCount": row.get::<_, i64>(2)?,
                    "coverId": row.get::<_, Option<String>>(3)?,
                    "genres": Vec::<String>::new(),
                    "followers": 0,
                    "isFollowing": false,
                }))
            })
            .map_err(|error| sqlite_error("media_library_artists", error))?;
        let artists = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| sqlite_error("media_library_artists", error))?;
        Ok(json!({ "artists": artists }))
    }

    fn refresh_track(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_string(&args, "trackId", "media_library_refresh_track")?;
        let path: Option<String> = self
            .connection("media_library_refresh_track")?
            .query_row(
                "SELECT path FROM tracks WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| sqlite_error("media_library_refresh_track", error))?;
        let path = path.ok_or_else(|| {
            media_error("media_library_refresh_track", "track is not a local file")
        })?;
        let track = parse_audio_file(Path::new(&path), &self.cache_dir)?;
        self.upsert_track(&track, None, None)?;
        Ok(json!({ "track": track }))
    }

    fn remove_track(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_string(&args, "trackId", "media_library_remove_track")?;
        let connection = self.connection("media_library_remove_track")?;
        let removed = connection
            .execute("DELETE FROM tracks WHERE id = ?1", params![id])
            .map_err(|error| sqlite_error("media_library_remove_track", error))?;
        Ok(json!({ "trackId": id, "removed": removed > 0 }))
    }

    fn cover_get(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_string(&args, "coverId", "media_cover_get")?;
        let record: Option<(String, String)> = self
            .connection("media_cover_get")?
            .query_row(
                "SELECT path, mime_type FROM covers WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| sqlite_error("media_cover_get", error))?;
        let Some((path, mime_type)) = record else {
            return Err(media_error("media_cover_get", "cover was not found"));
        };
        let metadata = fs::metadata(&path).map_err(|error| io_error("media_cover_get", error))?;
        if metadata.len() > MAX_COVER_BYTES {
            return Err(media_error(
                "media_cover_get",
                "cover is larger than the IPC limit",
            ));
        }
        let bytes = fs::read(&path).map_err(|error| io_error("media_cover_get", error))?;
        self.connection("media_cover_get")?
            .execute(
                "UPDATE covers SET last_accessed_at = ?1 WHERE id = ?2",
                params![now_ms(), id],
            )
            .map_err(|error| sqlite_error("media_cover_get", error))?;
        Ok(serde_json::to_value(CoverPayload {
            cover_id: id,
            mime_type,
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
        })
        .map_err(|error| media_error("media_cover_get", error.to_string()))?)
    }

    fn cover_path(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_string(&args, "coverId", "media_cover_path")?;
        let path: Option<String> = self
            .connection("media_cover_path")?
            .query_row(
                "SELECT path FROM covers WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| sqlite_error("media_cover_path", error))?;
        Ok(json!({
            "coverId": id,
            "path": path.filter(|path| Path::new(path).is_file())
        }))
    }

    fn playback_record(&mut self, args: Value) -> Result<Value, MediaError> {
        let track_id = required_string(&args, "trackId", "media_playback_record")?;
        let position_ms = args
            .get("positionMs")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0);
        let connection = self.connection("media_playback_record")?;
        connection
            .execute(
                "INSERT INTO playback_history(track_id, played_at, position_ms) VALUES(?1, ?2, ?3)",
                params![track_id, now_ms(), position_ms],
            )
            .map_err(|error| sqlite_error("media_playback_record", error))?;
        connection
            .execute(
                "UPDATE tracks SET last_played_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now_ms(), track_id],
            )
            .map_err(|error| sqlite_error("media_playback_record", error))?;
        Ok(json!({ "trackId": track_id, "recorded": true }))
    }

    fn playback_history(&mut self, args: Value) -> Result<Value, MediaError> {
        let limit = args
            .get("limit")
            .and_then(Value::as_i64)
            .unwrap_or(50)
            .clamp(1, 500);
        let connection = self.connection("media_playback_history")?;
        let mut statement = connection
            .prepare(
                "SELECT h.id, h.track_id, h.played_at, h.position_ms
                 FROM playback_history h ORDER BY h.played_at DESC LIMIT ?1",
            )
            .map_err(|error| sqlite_error("media_playback_history", error))?;
        let rows = statement
            .query_map(params![limit], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "trackId": row.get::<_, String>(1)?,
                    "playedAt": row.get::<_, i64>(2)?,
                    "positionMs": row.get::<_, i64>(3)?,
                }))
            })
            .map_err(|error| sqlite_error("media_playback_history", error))?;
        let history = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| sqlite_error("media_playback_history", error))?;
        Ok(json!({ "history": history }))
    }

    fn pick_folder(&self) -> Result<Value, MediaError> {
        let path = rfd::FileDialog::new().pick_folder();
        Ok(json!({ "path": path.map(|path| path.to_string_lossy().into_owned()) }))
    }

    fn should_refresh(&mut self, args: Value) -> Result<Value, MediaError> {
        let path = required_string(&args, "path", "media_should_refresh")?;
        let metadata =
            fs::metadata(&path).map_err(|error| io_error("media_should_refresh", error))?;
        let size = metadata.len() as i64;
        let modified_at = modified_ms(&metadata).unwrap_or(0);
        let existing: Option<(i64, i64)> = self
            .connection("media_should_refresh")?
            .query_row(
                "SELECT file_size, file_modified_at FROM tracks WHERE path = ?1",
                params![path],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| sqlite_error("media_should_refresh", error))?;
        Ok(
            json!({ "refresh": existing.map(|(old_size, old_modified)| old_size != size || old_modified != modified_at).unwrap_or(true) }),
        )
    }

    fn mark_seen(&mut self, args: Value) -> Result<Value, MediaError> {
        let path = required_string(&args, "path", "media_mark_seen")?;
        let scan_token = required_string(&args, "scanToken", "media_mark_seen")?;
        self.connection("media_mark_seen")?
            .execute(
                "UPDATE tracks SET scan_token = ?1, missing = 0, updated_at = ?2 WHERE path = ?3",
                params![scan_token, now_ms(), path],
            )
            .map_err(|error| sqlite_error("media_mark_seen", error))?;
        Ok(json!({ "path": path, "seen": true }))
    }

    fn upsert_track_value(&mut self, args: Value) -> Result<Value, MediaError> {
        let track: MediaTrack = serde_json::from_value(
            args.get("track")
                .cloned()
                .ok_or_else(|| media_error("media_upsert_track", "track is required"))?,
        )
        .map_err(|error| media_error("media_upsert_track", error.to_string()))?;
        let cache = args.get("urlCache").cloned();
        let scan_token = args.get("scanToken").and_then(Value::as_str);
        self.upsert_track(&track, cache.as_ref(), scan_token)?;
        Ok(json!({ "track": track }))
    }

    fn upsert_track(
        &mut self,
        track: &MediaTrack,
        url_cache: Option<&Value>,
        scan_token: Option<&str>,
    ) -> Result<(), MediaError> {
        let cache_dir = self.cache_dir.clone();
        let connection = self.connection("media_upsert_track")?;
        let tx = connection
            .transaction()
            .map_err(|error| sqlite_error("media_upsert_track", error))?;
        let now = now_ms();
        let path = track.path.clone();
        let file_size = path
            .as_deref()
            .and_then(|value| fs::metadata(value).ok())
            .map(|meta| meta.len() as i64)
            .unwrap_or(0);
        let file_modified_at = path
            .as_deref()
            .and_then(|value| fs::metadata(value).ok())
            .and_then(|meta| modified_ms(&meta))
            .unwrap_or(0);
        tx.execute(
            "INSERT INTO tracks(
                id, source, path, url, title, artist, album, album_artist, composer, genres_json,
                year, track_number, track_total, disc_number, disc_total, duration_ms, bitrate,
                sample_rate, channels, codec, format, cover_id, cover_mime_type, warnings_json,
                file_size, file_modified_at, scan_token, missing, added_at, updated_at, last_played_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                      ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, 0, COALESCE((SELECT added_at FROM tracks WHERE id = ?1), ?28), ?28, COALESCE((SELECT last_played_at FROM tracks WHERE id = ?1), NULL))
             ON CONFLICT(id) DO UPDATE SET
                source=excluded.source, path=excluded.path, url=excluded.url, title=excluded.title,
                artist=excluded.artist, album=excluded.album, album_artist=excluded.album_artist,
                composer=excluded.composer, genres_json=excluded.genres_json, year=excluded.year,
                track_number=excluded.track_number, track_total=excluded.track_total,
                disc_number=excluded.disc_number, disc_total=excluded.disc_total,
                duration_ms=excluded.duration_ms, bitrate=excluded.bitrate, sample_rate=excluded.sample_rate,
                channels=excluded.channels, codec=excluded.codec, format=excluded.format,
                cover_id=excluded.cover_id, cover_mime_type=excluded.cover_mime_type,
                warnings_json=excluded.warnings_json, file_size=excluded.file_size,
                file_modified_at=excluded.file_modified_at, scan_token=excluded.scan_token,
                missing=0, updated_at=excluded.updated_at",
            params![
                track.id,
                track.source,
                track.path,
                track.url,
                track.title,
                track.artist,
                track.album,
                track.album_artist,
                track.composer,
                serde_json::to_string(&track.genres).unwrap_or_else(|_| "[]".into()),
                track.year,
                track.track_number,
                track.track_total,
                track.disc_number,
                track.disc_total,
                track.duration_ms as i64,
                track.bitrate,
                track.sample_rate,
                track.channels,
                track.codec,
                track.format,
                track.cover_id,
                track.cover_mime_type,
                serde_json::to_string(&track.warnings).unwrap_or_else(|_| "[]".into()),
                file_size,
                file_modified_at,
                scan_token,
                now,
            ],
        )
        .map_err(|error| sqlite_error("media_upsert_track", error))?;
        if let (Some(cover_id), Some(mime_type)) = (&track.cover_id, &track.cover_mime_type) {
            let cover_path = cache_dir
                .join("covers")
                .join(format!("{cover_id}.{}", cover_extension(mime_type)));
            if cover_path.exists() {
                let byte_length = fs::metadata(&cover_path)
                    .map(|meta| meta.len() as i64)
                    .unwrap_or(0);
                tx.execute(
                    "INSERT INTO covers(id, path, mime_type, byte_length, created_at, last_accessed_at)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?5)
                     ON CONFLICT(id) DO UPDATE SET path=excluded.path, mime_type=excluded.mime_type,
                       byte_length=excluded.byte_length, last_accessed_at=excluded.last_accessed_at",
                    params![cover_id, cover_path.to_string_lossy(), mime_type, byte_length, now],
                )
                .map_err(|error| sqlite_error("media_upsert_track", error))?;
            }
        }
        if let Some(cache) = url_cache {
            let url = cache.get("url").and_then(Value::as_str).unwrap_or_default();
            if !url.is_empty() {
                tx.execute(
                    "INSERT INTO url_cache(url, local_path, etag, last_modified, content_type, content_length, fetched_at, last_accessed_at)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                     ON CONFLICT(url) DO UPDATE SET local_path=excluded.local_path, etag=excluded.etag,
                       last_modified=excluded.last_modified, content_type=excluded.content_type,
                       content_length=excluded.content_length, fetched_at=excluded.fetched_at,
                       last_accessed_at=excluded.last_accessed_at",
                    params![
                        url,
                        cache.get("localPath").and_then(Value::as_str),
                        cache.get("etag").and_then(Value::as_str),
                        cache.get("lastModified").and_then(Value::as_str),
                        cache.get("contentType").and_then(Value::as_str),
                        cache.get("contentLength").and_then(Value::as_u64).map(|value| value as i64),
                        now_ms(),
                    ],
                )
                .map_err(|error| sqlite_error("media_upsert_track", error))?;
            }
        }
        tx.commit()
            .map_err(|error| sqlite_error("media_upsert_track", error))?;
        if scan_token.is_none() {
            let _ = self
                .app
                .emit(EVENT_TRACK_UPDATED, json!({ "trackId": track.id }));
        }
        Ok(())
    }

    fn scan_cleanup(&mut self, args: Value) -> Result<Value, MediaError> {
        let job_id = required_string(&args, "jobId", "media_scan_cleanup")?;
        let root_ids = args
            .get("rootIds")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
            .unwrap_or_default();
        let complete = args
            .get("complete")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let state = args
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("finished")
            .to_owned();
        let connection = self.connection("media_scan_cleanup")?;
        if complete {
            for root_id in root_ids {
                connection
                    .execute("UPDATE tracks SET missing = 1, updated_at = ?1 WHERE source = 'file' AND path LIKE (SELECT path || '%' FROM library_roots WHERE id = ?2) AND COALESCE(scan_token, '') <> ?3", params![now_ms(), root_id, job_id])
                    .map_err(|error| sqlite_error("media_scan_cleanup", error))?;
                connection
                    .execute(
                        "UPDATE library_roots SET last_scan_at = ?1 WHERE id = ?2",
                        params![now_ms(), root_id],
                    )
                    .map_err(|error| sqlite_error("media_scan_cleanup", error))?;
            }
        }
        self.scan_flags.remove(&job_id);
        let _ = self.app.emit(
            EVENT_SCAN_FINISHED,
            json!({ "jobId": job_id, "state": state }),
        );
        Ok(json!({ "jobId": job_id, "finished": complete }))
    }

    fn url_cache_touch(&mut self, args: Value) -> Result<Value, MediaError> {
        let url = required_string(&args, "url", "media_url_cache_touch")?;
        let cache_dir = self.cache_dir.clone();
        let connection = self.connection("media_url_cache_touch")?;
        connection
            .execute(
                "UPDATE url_cache SET last_accessed_at = ?1 WHERE url = ?2",
                params![now_ms(), url],
            )
            .map_err(|error| sqlite_error("media_url_cache_touch", error))?;
        cleanup_cache(&connection, &cache_dir)
            .map_err(|error| sqlite_error("media_url_cache_touch", error))?;
        Ok(json!({ "url": url, "touched": true }))
    }

    fn track_source(&mut self, args: Value) -> Result<Value, MediaError> {
        let id = required_string(&args, "trackId", "media_track_source")?;
        let source: Option<(String, Option<String>, Option<String>)> = self
            .connection("media_track_source")?
            .query_row(
                "SELECT source, path, url FROM tracks WHERE id = ?1 AND missing = 0",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| sqlite_error("media_track_source", error))?;
        let Some((kind, path, url)) = source else {
            return Err(media_error(
                "media_track_source",
                "track is not in the library",
            ));
        };
        Ok(json!({ "trackId": id, "source": kind, "path": path, "url": url }))
    }
}

fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS library_roots(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            enabled INTEGER NOT NULL DEFAULT 1,
            added_at INTEGER NOT NULL,
            last_scan_at INTEGER
        );
        CREATE TABLE IF NOT EXISTS tracks(
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            path TEXT,
            url TEXT,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT NOT NULL,
            album_artist TEXT,
            composer TEXT,
            genres_json TEXT NOT NULL DEFAULT '[]',
            year INTEGER,
            track_number INTEGER,
            track_total INTEGER,
            disc_number INTEGER,
            disc_total INTEGER,
            duration_ms INTEGER NOT NULL DEFAULT 0,
            bitrate INTEGER,
            sample_rate INTEGER,
            channels INTEGER,
            codec TEXT,
            format TEXT,
            cover_id TEXT,
            cover_mime_type TEXT,
            warnings_json TEXT NOT NULL DEFAULT '[]',
            file_size INTEGER NOT NULL DEFAULT 0,
            file_modified_at INTEGER NOT NULL DEFAULT 0,
            scan_token TEXT,
            missing INTEGER NOT NULL DEFAULT 0,
            added_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_played_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
        CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
        CREATE TABLE IF NOT EXISTS track_artists(track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE, artist TEXT NOT NULL, PRIMARY KEY(track_id, artist));
        CREATE TABLE IF NOT EXISTS track_tags(track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE, tag_key TEXT NOT NULL, tag_value TEXT NOT NULL, PRIMARY KEY(track_id, tag_key, tag_value));
        CREATE TABLE IF NOT EXISTS albums(id TEXT PRIMARY KEY, title TEXT NOT NULL, artist TEXT, year INTEGER, cover_id TEXT, updated_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS artists(id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, cover_id TEXT, updated_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS covers(id TEXT PRIMARY KEY, path TEXT NOT NULL, mime_type TEXT NOT NULL, byte_length INTEGER NOT NULL, created_at INTEGER NOT NULL, last_accessed_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS url_cache(url TEXT PRIMARY KEY, local_path TEXT NOT NULL, etag TEXT, last_modified TEXT, content_type TEXT, content_length INTEGER, fetched_at INTEGER NOT NULL, last_accessed_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS scan_jobs(id TEXT PRIMARY KEY, state TEXT NOT NULL, root_ids_json TEXT NOT NULL, scanned INTEGER NOT NULL DEFAULT 0, imported INTEGER NOT NULL DEFAULT 0, skipped INTEGER NOT NULL DEFAULT 0, failed INTEGER NOT NULL DEFAULT 0, started_at INTEGER NOT NULL, finished_at INTEGER, error TEXT);
        CREATE TABLE IF NOT EXISTS playback_history(id INTEGER PRIMARY KEY AUTOINCREMENT, track_id TEXT NOT NULL, played_at INTEGER NOT NULL, position_ms INTEGER NOT NULL DEFAULT 0);
        PRAGMA user_version = 1;",
    )
}

fn row_to_track(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaTrack> {
    let genres_json: String = row.get(9)?;
    let warnings_json: String = row.get(23)?;
    Ok(MediaTrack {
        id: row.get(0)?,
        source: row.get(1)?,
        path: row.get(2)?,
        url: row.get(3)?,
        title: row.get(4)?,
        artist: row.get(5)?,
        album: row.get(6)?,
        album_artist: row.get(7)?,
        composer: row.get(8)?,
        genres: serde_json::from_str(&genres_json).unwrap_or_default(),
        year: row.get(10)?,
        track_number: row.get(11)?,
        track_total: row.get(12)?,
        disc_number: row.get(13)?,
        disc_total: row.get(14)?,
        duration_ms: row.get::<_, i64>(15)?.max(0) as u64,
        bitrate: row.get(16)?,
        sample_rate: row.get(17)?,
        channels: row.get(18)?,
        codec: row.get(19)?,
        format: row.get(20)?,
        cover_id: row.get(21)?,
        cover_mime_type: row.get(22)?,
        warnings: serde_json::from_str(&warnings_json).unwrap_or_default(),
        added_at: row.get(24)?,
        updated_at: row.get(25)?,
        last_played_at: row.get(26)?,
    })
}

fn run_scan(
    app: AppHandle,
    sender: Sender<MediaRequest>,
    cache_dir: PathBuf,
    job_id: String,
    roots: Vec<(i64, PathBuf)>,
    root_ids: Vec<i64>,
    cancel: Arc<AtomicBool>,
) {
    let mut scanned = 0_u64;
    let mut imported = 0_u64;
    let mut skipped = 0_u64;
    let mut failed = 0_u64;
    let _ = app.emit(EVENT_SCAN_PROGRESS, json!({ "jobId": job_id, "state": "running", "scanned": 0, "imported": 0, "skipped": 0, "failed": 0 }));
    for (root_id, root) in roots {
        for path in audio_files(&root) {
            if cancel.load(Ordering::Acquire) {
                let _ = call_worker(
                    &sender,
                    "media_scan_cleanup",
                    json!({ "jobId": job_id, "rootIds": root_ids.clone(), "state": "cancelled", "complete": false }),
                );
                let _ = app.emit(
                    EVENT_SCAN_FINISHED,
                    json!({ "jobId": job_id, "state": "cancelled" }),
                );
                return;
            }
            scanned += 1;
            let should_refresh = call_worker(
                &sender,
                "media_should_refresh",
                json!({ "path": path.to_string_lossy() }),
            )
            .ok()
            .and_then(|value| value.get("refresh").and_then(Value::as_bool))
            .unwrap_or(true);
            if !should_refresh {
                skipped += 1;
                let _ = call_worker(
                    &sender,
                    "media_mark_seen",
                    json!({ "path": path.to_string_lossy(), "scanToken": job_id }),
                );
            } else {
                match parse_audio_file(&path, &cache_dir) {
                    Ok(mut track) => {
                        track.source = "file".into();
                        track.path = Some(path.to_string_lossy().into_owned());
                        let value =
                            json!({ "track": track, "scanToken": job_id, "rootId": root_id });
                        if call_worker(&sender, "media_upsert_track", value).is_ok() {
                            imported += 1;
                            let _ = app.emit(
                                EVENT_TRACK_ADDED,
                                json!({ "path": path, "rootId": root_id }),
                            );
                        } else {
                            failed += 1;
                        }
                    }
                    Err(error) => {
                        failed += 1;
                        let _ = call_worker(
                            &sender,
                            "media_mark_seen",
                            json!({ "path": path.to_string_lossy(), "scanToken": job_id }),
                        );
                        let _ = app.emit(EVENT_ERROR, json!({ "operation": "media_library_scan", "path": path, "error": error }));
                    }
                }
            }
            let _ = app.emit(EVENT_SCAN_PROGRESS, json!({ "jobId": job_id, "state": "running", "scanned": scanned, "imported": imported, "skipped": skipped, "failed": failed }));
        }
    }
    let _ = call_worker(
        &sender,
        "media_scan_cleanup",
        json!({ "jobId": job_id, "rootIds": root_ids }),
    );
    let _ = app.emit(EVENT_SCAN_PROGRESS, json!({ "jobId": job_id, "state": "finished", "scanned": scanned, "imported": imported, "skipped": skipped, "failed": failed }));
}

fn call_worker(
    sender: &Sender<MediaRequest>,
    operation: &str,
    args: Value,
) -> Result<Value, MediaError> {
    let (reply, receiver) = mpsc::channel();
    sender
        .send(MediaRequest {
            operation: operation.into(),
            args,
            reply,
        })
        .map_err(|_| fatal_media_error(operation, "media worker is not running"))?;
    receiver
        .recv()
        .map_err(|_| fatal_media_error(operation, "media worker dropped the response"))?
}

fn parse_audio_file(path: &Path, cache_dir: &Path) -> Result<MediaTrack, MediaError> {
    if !path.is_file() {
        return Err(media_error(
            "media_metadata_read_file",
            "audio path is not a file",
        ));
    }
    let tagged_file =
        read_from_path(path).map_err(|error| parse_error("media_metadata_read_file", error))?;
    let properties = tagged_file.properties();
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let file_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Unknown title");
    let title = tag
        .and_then(|value| value.title().map(|text| text.into_owned()))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| file_name.to_owned());
    let artist = tag
        .and_then(|value| value.artist().map(|text| text.into_owned()))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown Artist".into());
    let album = tag
        .and_then(|value| value.album().map(|text| text.into_owned()))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown Album".into());
    let album_artist = tag
        .and_then(|value| value.get_string(ItemKey::AlbumArtist))
        .map(ToOwned::to_owned);
    let composer = tag
        .and_then(|value| value.get_string(ItemKey::Composer))
        .map(ToOwned::to_owned);
    let genres = tag
        .map(|value| {
            value
                .get_strings(ItemKey::Genre)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let year = tag
        .and_then(|value| value.get_string(ItemKey::Year))
        .and_then(parse_year);
    let (track_number, track_total) = tag
        .and_then(|value| value.get_string(ItemKey::TrackNumber))
        .map(parse_pair)
        .unwrap_or((None, None));
    let (disc_number, disc_total) = tag
        .and_then(|value| value.get_string(ItemKey::DiscNumber))
        .map(parse_pair)
        .unwrap_or((None, None));
    let mut warnings = Vec::new();
    if tag.is_none() {
        warnings.push("file has no embedded tags; filename fallback was used".into());
    }
    let (cover_id, cover_mime_type) = store_first_cover(&tagged_file, cache_dir)?;
    let format = Some(format!("{:?}", tagged_file.file_type()));
    let id = stable_id(&path.to_string_lossy());
    Ok(MediaTrack {
        id,
        source: "file".into(),
        path: Some(path.to_string_lossy().into_owned()),
        url: None,
        title,
        artist,
        album,
        album_artist,
        composer,
        genres,
        year,
        track_number,
        track_total,
        disc_number,
        disc_total,
        duration_ms: properties.duration().as_millis() as u64,
        bitrate: properties
            .audio_bitrate()
            .or_else(|| properties.overall_bitrate()),
        sample_rate: properties.sample_rate(),
        channels: properties.channels(),
        codec: format.clone(),
        format,
        cover_id,
        cover_mime_type,
        warnings,
        added_at: None,
        updated_at: None,
        last_played_at: None,
    })
}

fn store_first_cover(
    file: &lofty::file::TaggedFile,
    cache_dir: &Path,
) -> Result<(Option<String>, Option<String>), MediaError> {
    let Some(picture) = file
        .tags()
        .iter()
        .flat_map(|tag| tag.pictures())
        .find(|picture| !picture.data().is_empty())
    else {
        return Ok((None, None));
    };
    let bytes = picture.data();
    if bytes.len() as u64 > MAX_COVER_BYTES {
        return Ok((None, None));
    }
    let id = stable_bytes_id(bytes);
    let mime_type = picture
        .mime_type()
        .map(ToString::to_string)
        .unwrap_or_else(|| "application/octet-stream".into());
    let extension = picture
        .mime_type()
        .and_then(|mime| mime.ext())
        .unwrap_or("img");
    let directory = cache_dir.join("covers");
    fs::create_dir_all(&directory).map_err(|error| io_error("media_cover_store", error))?;
    let path = directory.join(format!("{id}.{extension}"));
    if !path.exists() {
        fs::write(&path, bytes).map_err(|error| io_error("media_cover_store", error))?;
    }
    Ok((Some(id), Some(mime_type)))
}

fn provisional_url_track(url: &str) -> MediaTrack {
    let title = url
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .split('?')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("Network stream")
        .to_owned();
    MediaTrack {
        id: stable_id(url),
        source: "url".into(),
        path: None,
        url: Some(url.into()),
        title,
        artist: "Unknown Artist".into(),
        album: "Network Stream".into(),
        warnings: vec!["metadata is provisional and may be updated from stream tags".into()],
        ..Default::default()
    }
}

fn download_and_parse_url(
    url: &str,
    cache_dir: &Path,
) -> Result<
    (
        MediaTrack,
        PathBuf,
        Option<String>,
        Option<String>,
        Option<String>,
        u64,
    ),
    MediaError,
> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| media_error("media_metadata_read_url", error.to_string()))?;
    let response = client
        .get(url)
        .header("Icy-MetaData", "1")
        .send()
        .map_err(|error| media_error("media_metadata_read_url", error.to_string()))?;
    if !response.status().is_success() {
        return Err(media_error(
            "media_metadata_read_url",
            format!("HTTP {}", response.status()),
        ));
    }
    let content_length = response.content_length().unwrap_or(0);
    if content_length > MAX_URL_DOWNLOAD_BYTES {
        return Err(media_error(
            "media_metadata_read_url",
            "remote file exceeds the metadata download limit",
        ));
    }
    let etag = header_string(&response, "etag");
    let last_modified = header_string(&response, "last-modified");
    let content_type = header_string(&response, "content-type");
    let extension = url
        .rsplit('.')
        .next()
        .and_then(|value| value.split('?').next())
        .filter(|value| value.len() <= 8)
        .unwrap_or("audio");
    let directory = cache_dir.join("urls");
    fs::create_dir_all(&directory).map_err(|error| io_error("media_metadata_read_url", error))?;
    let id = stable_id(url);
    let path = directory.join(format!("{id}.{extension}"));
    let part = path.with_extension(format!("{extension}.part"));
    let mut file =
        fs::File::create(&part).map_err(|error| io_error("media_metadata_read_url", error))?;
    let copied = std::io::copy(&mut response.take(MAX_URL_DOWNLOAD_BYTES + 1), &mut file)
        .map_err(|error| io_error("media_metadata_read_url", error))?;
    if copied > MAX_URL_DOWNLOAD_BYTES {
        let _ = fs::remove_file(&part);
        return Err(media_error(
            "media_metadata_read_url",
            "remote file exceeds the metadata download limit",
        ));
    }
    drop(file);
    fs::rename(&part, &path).map_err(|error| io_error("media_metadata_read_url", error))?;
    let mut track = parse_audio_file(&path, cache_dir)?;
    track.source = "url".into();
    track.path = None;
    track.url = Some(url.into());
    Ok((track, path, etag, last_modified, content_type, copied))
}

fn header_string(response: &reqwest::blocking::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn cleanup_cache(connection: &Connection, cache_dir: &Path) -> rusqlite::Result<()> {
    let total: i64 = connection.query_row(
        "SELECT COALESCE(SUM(content_length), 0) FROM url_cache",
        [],
        |row| row.get(0),
    )?;
    if total <= CACHE_SOFT_LIMIT_BYTES as i64 {
        return Ok(());
    }
    let mut statement = connection.prepare(
        "SELECT local_path, content_length FROM url_cache ORDER BY last_accessed_at ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut current = total as u64;
    for row in rows {
        let (path, size) = row?;
        if current <= CACHE_SOFT_LIMIT_BYTES {
            break;
        }
        let _ = fs::remove_file(Path::new(&path));
        connection.execute("DELETE FROM url_cache WHERE local_path = ?1", params![path])?;
        current = current.saturating_sub(size.max(0) as u64);
    }
    let _ = cache_dir;
    Ok(())
}

fn audio_files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() && is_supported_audio(&path) {
                result.push(path);
            }
        }
    }
    result.sort();
    result
}

fn is_supported_audio(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some(
            "mp3"
                | "flac"
                | "m4a"
                | "mp4"
                | "aac"
                | "ogg"
                | "oga"
                | "opus"
                | "wav"
                | "aiff"
                | "aif"
                | "ape"
                | "wv"
                | "mpc"
        )
    )
}

fn canonical_directory(value: String) -> Result<String, MediaError> {
    let path =
        fs::canonicalize(&value).map_err(|error| io_error("media_library_add_root", error))?;
    if !path.is_dir() {
        return Err(media_error(
            "media_library_add_root",
            "path is not a directory",
        ));
    }
    Ok(path.to_string_lossy().into_owned())
}

fn validate_url(url: &str) -> Result<(), MediaError> {
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(media_error(
            "media_metadata_read_url",
            "only http and https URLs are supported",
        ));
    }
    Ok(())
}

fn parse_pair(value: &str) -> (Option<i32>, Option<i32>) {
    let mut parts = value.split('/');
    (
        parts.next().and_then(|part| part.trim().parse().ok()),
        parts.next().and_then(|part| part.trim().parse().ok()),
    )
}

fn parse_year(value: &str) -> Option<i32> {
    value.get(..4).and_then(|value| value.parse().ok())
}

fn stable_id(value: &str) -> String {
    stable_bytes_id(value.as_bytes())
}

fn stable_bytes_id(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}

fn cover_extension(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/tiff" => "tif",
        "image/bmp" => "bmp",
        "image/gif" => "gif",
        _ => "img",
    }
}

fn modified_ms(metadata: &fs::Metadata) -> Option<i64> {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as i64)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn format_duration(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn required_string(args: &Value, field: &str, operation: &str) -> Result<String, MediaError> {
    args.get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| media_error(operation, format!("field {field} is required")))
}

fn required_i64(args: &Value, field: &str, operation: &str) -> Result<i64, MediaError> {
    args.get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| media_error(operation, format!("field {field} must be an integer")))
}

fn value_result<T: for<'de> Deserialize<'de>>(
    value: Value,
    operation: &str,
) -> Result<T, MediaError> {
    serde_json::from_value(value).map_err(|error| media_error(operation, error.to_string()))
}

#[tauri::command]
pub fn media_metadata_read_file(
    service: State<'_, MediaService>,
    path: String,
) -> Result<MediaTrack, MediaError> {
    value_result(
        service.call("media_metadata_read_file", json!({ "path": path }))?,
        "media_metadata_read_file",
    )
}

#[tauri::command]
pub fn media_metadata_read_url(
    service: State<'_, MediaService>,
    url: String,
) -> Result<MediaTrack, MediaError> {
    value_result(
        service.call("media_metadata_read_url", json!({ "url": url }))?,
        "media_metadata_read_url",
    )
}

#[tauri::command]
pub fn media_lyrics_read(
    service: State<'_, MediaService>,
    track_id: String,
) -> Result<LyricsPayload, MediaError> {
    value_result(
        service.call("media_lyrics_read", json!({ "trackId": track_id }))?,
        "media_lyrics_read",
    )
}

#[tauri::command]
pub fn media_library_add_root(
    service: State<'_, MediaService>,
    path: String,
) -> Result<Value, MediaError> {
    service.call("media_library_add_root", json!({ "path": path }))
}

#[tauri::command]
pub fn media_library_remove_root(
    service: State<'_, MediaService>,
    root_id: i64,
) -> Result<Value, MediaError> {
    service.call("media_library_remove_root", json!({ "rootId": root_id }))
}

#[tauri::command]
pub fn media_library_roots(service: State<'_, MediaService>) -> Result<Value, MediaError> {
    service.call("media_library_roots", json!({}))
}

#[tauri::command]
pub fn media_library_scan(
    service: State<'_, MediaService>,
    root_ids: Option<Vec<i64>>,
) -> Result<Value, MediaError> {
    service.call("media_library_scan", json!({ "rootIds": root_ids }))
}

#[tauri::command]
pub fn media_library_cancel_scan(
    service: State<'_, MediaService>,
    job_id: String,
) -> Result<Value, MediaError> {
    service.call("media_library_cancel_scan", json!({ "jobId": job_id }))
}

#[tauri::command]
pub fn media_library_tracks(
    service: State<'_, MediaService>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Value, MediaError> {
    service.call(
        "media_library_tracks",
        json!({ "search": search.unwrap_or_default(), "limit": limit, "offset": offset }),
    )
}

#[tauri::command]
pub fn media_library_albums(
    service: State<'_, MediaService>,
    search: Option<String>,
) -> Result<Value, MediaError> {
    service.call(
        "media_library_albums",
        json!({ "search": search.unwrap_or_default() }),
    )
}

#[tauri::command]
pub fn media_library_artists(
    service: State<'_, MediaService>,
    search: Option<String>,
) -> Result<Value, MediaError> {
    service.call(
        "media_library_artists",
        json!({ "search": search.unwrap_or_default() }),
    )
}

#[tauri::command]
pub fn media_library_refresh_track(
    service: State<'_, MediaService>,
    track_id: String,
) -> Result<Value, MediaError> {
    service.call(
        "media_library_refresh_track",
        json!({ "trackId": track_id }),
    )
}

#[tauri::command]
pub fn media_library_remove_track(
    service: State<'_, MediaService>,
    track_id: String,
) -> Result<Value, MediaError> {
    service.call("media_library_remove_track", json!({ "trackId": track_id }))
}

#[tauri::command]
pub fn media_cover_get(
    service: State<'_, MediaService>,
    cover_id: String,
) -> Result<CoverPayload, MediaError> {
    value_result(
        service.call("media_cover_get", json!({ "coverId": cover_id }))?,
        "media_cover_get",
    )
}

#[tauri::command]
pub fn media_cover_path(
    service: State<'_, MediaService>,
    cover_id: String,
) -> Result<Value, MediaError> {
    service.call("media_cover_path", json!({ "coverId": cover_id }))
}

#[tauri::command]
pub fn media_playback_history(
    service: State<'_, MediaService>,
    limit: Option<i64>,
) -> Result<Value, MediaError> {
    service.call("media_playback_history", json!({ "limit": limit }))
}

#[tauri::command]
pub fn media_playback_record(
    service: State<'_, MediaService>,
    track_id: String,
    position_ms: Option<i64>,
) -> Result<Value, MediaError> {
    service.call(
        "media_playback_record",
        json!({ "trackId": track_id, "positionMs": position_ms }),
    )
}

#[tauri::command]
pub fn media_pick_folder(service: State<'_, MediaService>) -> Result<Value, MediaError> {
    service.call("media_pick_folder", json!({}))
}

#[tauri::command]
pub fn media_playback_open(
    media: State<'_, MediaService>,
    bass: State<'_, BassService>,
    track_id: String,
) -> Result<Value, MediaError> {
    let source = media.call("media_track_source", json!({ "trackId": track_id }))?;
    let source_kind = source
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("file");
    let status = bass
        .call_operation("bass_status", json!({}))
        .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    if !status
        .get("loaded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        bass.call_operation("bass_load", json!({ "requireFx": false }))
            .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    }
    let status = bass
        .call_operation("bass_status", json!({}))
        .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    if !status
        .get("initialized")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        bass.call_operation("bass_initialize", json!({}))
            .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    }
    let status = bass
        .call_operation("bass_status", json!({}))
        .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    if !status
        .get("started")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        bass.call_operation("bass_start", json!({}))
            .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    }
    let bass_result = match source_kind {
        "url" => bass
            .call_operation(
                "bass_load_url",
                json!({ "url": source.get("url").and_then(Value::as_str).unwrap_or_default() }),
            )
            .map_err(|error| bridge_to_media_error("media_playback_open", error))?,
        _ => bass
            .call_operation(
                "bass_load_file",
                json!({ "path": source.get("path").and_then(Value::as_str).unwrap_or_default() }),
            )
            .map_err(|error| bridge_to_media_error("media_playback_open", error))?,
    };
    let channel_id = bass_result
        .get("channelId")
        .and_then(Value::as_u64)
        .ok_or_else(|| media_error("media_playback_open", "BASS did not return a channel id"))?;
    let _ = bass
        .call_operation(
            "bass_channel_play",
            json!({ "channelId": channel_id, "restart": true }),
        )
        .map_err(|error| bridge_to_media_error("media_playback_open", error))?;
    let _ = media.call(
        "media_playback_record",
        json!({ "trackId": track_id, "positionMs": 0 }),
    );
    Ok(json!({ "trackId": track_id, "channel": bass_result }))
}

fn bridge_to_media_error(operation: &str, error: BridgeError) -> MediaError {
    MediaError {
        kind: error.kind,
        operation: operation.into(),
        message: error.message,
        debug: error.debug,
        recoverable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_ids_are_deterministic() {
        assert_eq!(stable_id("same"), stable_id("same"));
        assert_ne!(stable_id("same"), stable_id("different"));
    }

    #[test]
    fn parses_track_and_disc_pairs() {
        assert_eq!(parse_pair("2/11"), (Some(2), Some(11)));
        assert_eq!(parse_pair("4"), (Some(4), None));
        assert_eq!(parse_pair("bad"), (None, None));
    }

    #[test]
    fn formats_duration() {
        assert_eq!(format_duration(65_000), "01:05");
        assert_eq!(format_duration(3_661_000), "1:01:01");
    }

    #[test]
    fn validates_only_http_urls() {
        assert!(validate_url("https://example.com/song.mp3").is_ok());
        assert!(validate_url("file:///song.mp3").is_err());
    }

    #[test]
    fn sqlite_migration_is_idempotent() {
        let connection = Connection::open_in_memory().expect("sqlite");
        migrate(&connection).expect("first migration");
        migrate(&connection).expect("second migration");
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'tracks'",
                [],
                |row| row.get(0),
            )
            .expect("table count");
        assert_eq!(count, 1);
    }
}
