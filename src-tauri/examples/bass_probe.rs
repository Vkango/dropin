use bass_library::{BassEngine, BassEngineOptions, InitOptions, SourceOptions};
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dll_dir = PathBuf::from("src-tauri/resources/bass/x64");
    let engine = BassEngine::load_from_directory_with_options(&dll_dir, BassEngineOptions::default())?;
    engine.initialize(InitOptions::default())?;

    let root = PathBuf::from("E:\\Music");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && matches!(
                    path.extension().and_then(|value| value.to_str()),
                    Some("mp3" | "flac" | "ogg" | "m4a" | "wav" | "aac" | "wma")
                )
        })
        .collect();
    files.sort();
    files.truncate(8);

    for path in &files {
        let options = SourceOptions {
            float: true,
            decode_only: true,
            ..SourceOptions::default()
        };
        let channel = match engine.load_file(path, options) {
            Ok(channel) => channel,
            Err(error) => {
                println!("LOAD-FAIL {}: {error}", path.display());
                continue;
            }
        };
        let info = match channel.info() {
            Ok(info) => info,
            Err(error) => {
                println!("INFO-FAIL {}: {error}", path.display());
                continue;
            }
        };
        let length_seconds = channel
            .length()?
            .map(|duration| duration.as_secs_f64())
            .unwrap_or(0.0);
        let start_seconds = (length_seconds / 4.0).min(10.0);
        if let Err(error) = channel.seek(Duration::from_secs_f64(start_seconds)) {
            println!("SEEK-FAIL {}: {error}", path.display());
            continue;
        }
        match channel.read_float_data(8000, 0) {
            Ok(samples) => {
                let square_sum: f64 = samples.iter().map(|value| (*value as f64) * (*value as f64)).sum();
                let rms = if samples.is_empty() {
                    0.0
                } else {
                    (square_sum / samples.len() as f64).sqrt()
                };
                let head: Vec<String> = samples.iter().take(6).map(|value| format!("{value:.4}")).collect();
                println!(
                    "OK {} | freq={} chans={} len={:.1}s start={:.1}s n={} rms={:.5} head=[{}]",
                    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
                    info.frequency,
                    info.channels,
                    length_seconds,
                    start_seconds,
                    samples.len(),
                    rms,
                    head.join(", ")
                );
            }
            Err(error) => println!("READ-FAIL {}: {error}", path.display()),
        }
    }
    Ok(())
}
