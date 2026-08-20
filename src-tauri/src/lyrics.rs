use std::{
    fs,
    path::{Path, PathBuf},
};

use encoding_rs::{GB18030, UTF_16BE, UTF_16LE};
use lofty::{
    file::{AudioFile, TaggedFileExt},
    read_from_path,
    tag::ItemKey,
};
use lrc::Lyrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub text: String,
    pub secondary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LyricsPayload {
    pub source: String,
    pub lines: Vec<LyricLine>,
    pub plain_lines: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct ParsedLyrics {
    timed: Vec<(u64, String)>,
    plain: Vec<String>,
}

pub fn read_for_audio_path(path: &Path, allow_sidecar: bool) -> Result<LyricsPayload, String> {
    if !path.is_file() {
        return Err("audio path is not a file".into());
    }

    let tagged_file = read_from_path(path).map_err(|error| error.to_string())?;
    let duration_ms = tagged_file.properties().duration().as_millis() as u64;
    let mut warnings = Vec::new();
    for text in embedded_lyrics(&tagged_file) {
        match parse_lrc_text(text) {
            Ok(parsed) if parsed.has_content() => {
                return Ok(payload_from_parsed(
                    "embedded",
                    parsed,
                    duration_ms,
                    warnings,
                ));
            }
            Ok(_) => warnings.push("embedded lyrics are empty".into()),
            Err(error) => warnings.push(format!("embedded lyrics could not be parsed: {error}")),
        }
    }

    if allow_sidecar {
        if let Some(sidecar) = find_sidecar(path) {
            match read_sidecar(&sidecar).and_then(|text| parse_lrc_text(&text)) {
                Ok(parsed) if parsed.has_content() => {
                    return Ok(payload_from_parsed(
                        "sidecar",
                        parsed,
                        duration_ms,
                        warnings,
                    ));
                }
                Ok(_) => warnings.push("sidecar lyrics are empty".into()),
                Err(error) => warnings.push(format!("sidecar lyrics could not be parsed: {error}")),
            }
        }
    }

    Ok(LyricsPayload {
        source: "none".into(),
        warnings,
        ..Default::default()
    })
}

fn embedded_lyrics<'a>(tagged_file: &'a lofty::file::TaggedFile) -> Vec<&'a str> {
    let mut lyrics = Vec::new();
    for tag in [tagged_file.primary_tag(), tagged_file.first_tag()]
        .into_iter()
        .flatten()
    {
        for value in [
            tag.get_string(ItemKey::Lyrics),
            tag.get_string(ItemKey::UnsyncLyrics),
        ]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        {
            if !lyrics.iter().any(|candidate| candidate == &value) {
                lyrics.push(value);
            }
        }
    }

    lyrics
}

fn find_sidecar(audio_path: &Path) -> Option<PathBuf> {
    let parent = audio_path.parent()?;
    let stem = audio_path.file_stem()?.to_str()?;
    let direct = parent.join(format!("{}.lrc", stem));
    if direct.is_file() {
        return Some(direct);
    }

    fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|candidate| {
            candidate.is_file()
                && candidate
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("lrc"))
                && candidate
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|candidate_stem| candidate_stem == stem)
        })
}

fn read_sidecar(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    decode_text(&bytes)
}

fn decode_text(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec()).map_err(|error| error.to_string());
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (text, _, had_errors) = UTF_16LE.decode(&bytes[2..]);
        return (!had_errors)
            .then_some(text.into_owned())
            .ok_or_else(|| "UTF-16LE lyrics contain invalid sequences".into());
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (text, _, had_errors) = UTF_16BE.decode(&bytes[2..]);
        return (!had_errors)
            .then_some(text.into_owned())
            .ok_or_else(|| "UTF-16BE lyrics contain invalid sequences".into());
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok(text.to_owned());
    }

    let (text, _, had_errors) = GB18030.decode(bytes);
    (!had_errors)
        .then_some(text.into_owned())
        .ok_or_else(|| "lyrics use an unsupported text encoding".into())
}

fn parse_lrc_text(text: &str) -> Result<ParsedLyrics, String> {
    let lyrics = text.parse::<Lyrics>().map_err(|error| error.to_string())?;
    let timed = lyrics
        .get_timed_lines()
        .iter()
        .filter_map(|(timestamp, text)| {
            let text = text.trim();
            (!text.is_empty()).then_some((timestamp.get_timestamp().max(0) as u64, text.to_owned()))
        })
        .collect();
    let plain = lyrics
        .get_lines()
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Ok(ParsedLyrics { timed, plain })
}

impl ParsedLyrics {
    fn has_content(&self) -> bool {
        !self.timed.is_empty() || !self.plain.is_empty()
    }
}

fn payload_from_parsed(
    source: &str,
    parsed: ParsedLyrics,
    duration_ms: u64,
    warnings: Vec<String>,
) -> LyricsPayload {
    let mut lines: Vec<LyricLine> = Vec::new();
    for (timestamp, text) in parsed.timed {
        if let Some(line) = lines.last_mut() {
            if line.start_time_ms == timestamp {
                if text != line.text && !line.secondary.iter().any(|value| value == &text) {
                    line.secondary.push(text);
                }
                continue;
            }
        }

        lines.push(LyricLine {
            start_time_ms: timestamp,
            end_time_ms: 0,
            text,
            secondary: Vec::new(),
        });
    }

    for index in 0..lines.len() {
        let next_start = lines.get(index + 1).map(|line| line.start_time_ms);
        lines[index].end_time_ms = next_start.unwrap_or_else(|| {
            if duration_ms > lines[index].start_time_ms {
                duration_ms
            } else {
                lines[index].start_time_ms.saturating_add(5_000)
            }
        });
    }

    LyricsPayload {
        source: source.into(),
        lines,
        plain_lines: parsed.plain,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_equal_timestamps_as_secondary_lines() {
        let parsed = parse_lrc_text(
            "[00:01.00]主歌词\n[00:01.00]副歌词\n[00:01.00]第三行\n[00:03.00]下一行",
        )
        .expect("valid lrc");
        let payload = payload_from_parsed("sidecar", parsed, 5_000, Vec::new());

        assert_eq!(payload.lines.len(), 2);
        assert_eq!(payload.lines[0].text, "主歌词");
        assert_eq!(payload.lines[0].secondary, vec!["副歌词", "第三行"]);
        assert_eq!(payload.lines[0].end_time_ms, 3_000);
        assert_eq!(payload.lines[1].end_time_ms, 5_000);
    }

    #[test]
    fn parses_plain_text_without_timestamps() {
        let parsed = parse_lrc_text("第一行\n第二行").expect("valid plain lyrics");
        let payload = payload_from_parsed("embedded", parsed, 0, Vec::new());

        assert!(payload.lines.is_empty());
        assert_eq!(payload.plain_lines, vec!["第一行", "第二行"]);
    }

    #[test]
    fn accepts_utf8_and_utf16_bom() {
        assert_eq!(decode_text("歌词".as_bytes()).unwrap(), "歌词");

        let mut bytes = vec![0xFF, 0xFE];
        for code_unit in "歌词".encode_utf16() {
            bytes.extend_from_slice(&code_unit.to_le_bytes());
        }
        assert_eq!(decode_text(&bytes).unwrap(), "歌词");

        let (gb18030, _, _) = GB18030.encode("歌词");
        assert_eq!(decode_text(&gb18030).unwrap(), "歌词");
    }

    #[test]
    fn rejects_invalid_lrc() {
        assert!(parse_lrc_text("[00:60.00]line").is_err());
    }
}
