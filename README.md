<div align="right">
  <strong>简体中文</strong> | <a href="./docs/README_EN.md">English</a>
</div>

<div align="center">

# Dropin

声来有理，不期而遇

<span></span>

![Version](https://img.shields.io/badge/🐢-龟速更新-red.svg) ![STARS](https://img.shields.io/github/stars/Vkango/dropin?style=round-square&logo=github&color=yellow) ![FORKS](https://img.shields.io/github/forks/Vkango/dropin?style=round-square)

基于 `Tauri2.0` + `Vue3` + `BASS` 构建的音乐播放器, 适用于桌面端应用, 缓速更新中……

Dropin 以开放的态度开发, 欢迎提交 PR 以及相关探索. 感谢支持.

</div>

> [!warning]
>
> **此软件仅供学习交流使用, 严禁用于商业用途. 出现的任何后果作者概不负责!**

> [!caution]
>
> **应用目前处于早期开发阶段, 存在运行不稳定、功能不可用等问题, 数据格式可能随时变更并不兼容旧版, 仅供测试使用!**

## 🚀 功能

- [x] 本地音乐库: 导入音乐文件夹, 自动扫描曲目 / 专辑 / 艺术家, 支持封面提取
- [x] 播放: 顺序 / 随机 / 单曲循环, 列表循环, 播放进度记忆
- [x] 歌词: 读取本地歌词文件, 全屏歌词显示
- [x] Windows 系统集成: SMTC 媒体控制 (系统媒体键 / 音量面板)
- [x] 主题: 跟随系统深浅色, 专辑封面自动取色
- [x] 多语言支持.
- [ ] 音效设置
- [ ] 插件系统

等等, 一个个来吧!

## 🐛 尝鲜与调试

确保已安装 Rust 工具链与 [Tauri 前置依赖](https://tauri.app/start/prerequisites/).

安装依赖: `pnpm install`

运行 Dev 版: `pnpm tauri dev`

构建发布版: `pnpm tauri build`

## 📷 运行截图

![1](docs/assets/1.png)

![2](docs/assets/2.png)

![3](docs/assets/3.png)

![4](docs/assets/4.png)

## 📚 引用与参考项目

Dropin 的开发参考并使用了以下开源项目与库:

| 项目 / 包 | 用途 |
| --- | --- |
| [Tauri](https://tauri.app/) | 跨平台桌面应用框架 |
| [Vue 3](https://vuejs.org/) | 前端界面框架 |
| [Vite](https://vitejs.dev/) | 前端构建工具 |
| [BASS](https://www.un4seen.com/) ([bass-library](https://crates.io/crates/bass-library)) | 音频播放核心 |
| [lofty](https://crates.io/crates/lofty) | 音频元数据读取与封面提取 |
| [rusqlite](https://crates.io/crates/rusqlite) | 本地 SQLite 数据存储 |
| [smtc-player](https://crates.io/crates/smtc-player) | Windows SMTC 媒体控制集成 |
| [rfd](https://crates.io/crates/rfd) | 跨平台文件/文件夹选择对话框 |
| [lrc](https://crates.io/crates/lrc) | LRC 歌词解析 |
| [motion-v](https://motion.dev/) | Vue 动画库 |
| [Lucide](https://lucide.dev/) (@lucide/vue) | 图标库 |
| [pinyin-pro](https://github.com/hotoo/pinyin-pro) | 中文拼音转换 (字母排序) |
