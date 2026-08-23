<div align="right">
  <a href="../README.md">简体中文</a> | <strong>English</strong>
</div>

<div align="center">

# Dropin

A voice with reason, an encounter unlooked for

<span></span>

![Version](https://img.shields.io/badge/🐢-Slow_updates-red.svg) ![STARS](https://img.shields.io/github/stars/Vkango/dropin?style=round-square&logo=github&color=yellow) ![FORKS](https://img.shields.io/github/forks/Vkango/dropin?style=round-square)

A music player built with `Tauri2.0` + `Vue3` + `BASS`, for desktop applications. Updates are slow but steady……

Dropin is developed in an open manner. PRs and related exploration are welcome. Thanks for your support.

</div>

> [!warning]
>
> **This software is for learning and exchange purposes only. Commercial use is strictly prohibited. The author is not responsible for any consequences that may arise!**

> [!caution]
>
> **The application is currently in early development. It may run unstably, and some features may be unavailable. The data format is subject to change at any time and may not be backward compatible. For testing purposes only!**

## 🚀 Features

- [x] Local music library: import music folders, automatically scan tracks / albums / artists, supports cover extraction
- [x] Playback: sequential / shuffle / repeat one, list repeat, playback progress memory
- [x] Lyrics: read local lyrics files, full-screen lyrics display
- [x] Windows system integration: SMTC media control (system media keys / volume panel)
- [x] Themes: follow system light/dark mode, automatic color extraction from album covers
- [x] Multi-language support.
- [ ] Audio effects settings
- [ ] Plugin system

And so on, one by one!

## 🐛 Try it out & Debugging

Make sure the Rust toolchain and [Tauri prerequisites](https://tauri.app/start/prerequisites/) are installed.

Install dependencies: `pnpm install`

Run Dev version: `pnpm tauri dev`

Build release version: `pnpm tauri build`

## 📷 Screenshots

![1](assets/1.png)

![2](assets/2.png)

![3](assets/3.png)

![4](assets/4.png)

## 📚 Credits & References

Dropin's development was inspired by and built upon the following open-source projects and libraries:

| Project / Package | Usage |
| --- | --- |
| [Tauri](https://tauri.app/) | Cross-platform desktop application framework |
| [Vue 3](https://vuejs.org/) | Frontend UI framework |
| [Vite](https://vitejs.dev/) | Frontend build tool |
| [BASS](https://www.un4seen.com/) ([bass-library](https://crates.io/crates/bass-library)) | Audio playback core |
| [lofty](https://crates.io/crates/lofty) | Audio metadata reading & cover extraction |
| [rusqlite](https://crates.io/crates/rusqlite) | Local SQLite data storage |
| [smtc-player](https://crates.io/crates/smtc-player) | Windows SMTC media control integration |
| [rfd](https://crates.io/crates/rfd) | Cross-platform file/folder picker dialogs |
| [lrc](https://crates.io/crates/lrc) | LRC lyrics parsing |
| [motion-v](https://motion.dev/) | Vue animation library |
| [Lucide](https://lucide.dev/) (@lucide/vue) | Icon library |
| [pinyin-pro](https://github.com/hotoo/pinyin-pro) | Chinese pinyin conversion (alphabetical sorting) |
