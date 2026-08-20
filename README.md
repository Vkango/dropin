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
