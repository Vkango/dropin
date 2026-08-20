# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## BASS audio engine

The Tauri backend uses the dynamically loaded `bass-rs` crate. The current
development setup expects the crate at `../../../Rust/bass-rs` relative to
`src-tauri/Cargo.toml`; update that path if the crate is stored elsewhere.

For Windows x64, put the official runtime files here:

```text
src-tauri/resources/bass/x64/bass.dll
src-tauri/resources/bass/x64/bass_fx.dll
```

The files are also included as Tauri bundle resources. Optional BASS add-ons
can be placed in the same directory and loaded from the BASS test page. The
application exposes the bridge through the `bass_call` command and emits
`bass/download`, `bass/sync`, `bass/dsp`, and `bass/channel-state` events.

The BASS DLLs are third-party proprietary components. Keep the original files
unchanged and review the official licensing terms before adding donations,
advertising, or paid distribution: https://www.un4seen.com/bass.html

## Real music library

Local metadata is parsed in Rust with `lofty`. The Tauri WebView does not
depend on a Node.js runtime for file access. Library roots, normalized track
metadata, playback history, URL metadata cache records, and cover references
are stored in SQLite under the application data directory. Imported folders
are scanned recursively from the library page; use the folder controls to
import a directory and start a scan.

Remote URLs are playable immediately through BASS. Stream information is
treated as provisional; when the URL is a complete downloadable audio file,
the Rust metadata worker downloads it into the application cache and updates
the SQLite record after parsing it with `lofty`. Live streams keep their
stream tags when a complete file cannot be downloaded.
