# Energy Tag Provider 示例插件

无 UI 的 headless Tag Provider 示例：对每首歌曲在全曲均匀取多个位置读取音频，汇总计算 RMS 能量并归一化为 0-100 的数值标签（`energy`）。

插件清单声明 `tagProvider`，权限为 `tag.provider`、`library.read`、`library.audio.read`。安装并授权、启用后，点击「我的标签」加号，填写名称、选择 Energy 来源，插件会按主程序显示语言填入 Wiki；修改后创建标签，点击刷新即可开始分析。选择「亲手选集」时，使用「管理歌曲」人工添加和移除歌曲。

## 构建 backend.wasm

```powershell
$targetDir = Join-Path $env:TEMP 'dropin-tag-provider-target'
cargo build --manifest-path plugin-sdk/examples/tag-provider/backend/Cargo.toml --release --target wasm32-unknown-unknown --target-dir $targetDir
Copy-Item -LiteralPath (Join-Path $targetDir 'wasm32-unknown-unknown\release\dropin_tag_energy_backend.wasm') -Destination plugin-sdk\examples\tag-provider\backend.wasm -Force
```

## 打包

```powershell
Push-Location plugin-sdk\examples\tag-provider
Compress-Archive -LiteralPath @('plugin.json', 'backend.wasm', 'icon.svg', 'i18n') -DestinationPath ..\tag-provider.zip -Force
Pop-Location
Move-Item -LiteralPath plugin-sdk\examples\tag-provider.zip -Destination plugin-sdk\examples\tag-provider.dropin -Force
```

## 说明

- Dropin 逐首调用 `backend.tag.analyze`，返回 `{ results: [{ trackId, key, value }] }`，详见 `plugin-sdk/wasm/rust/README.md` 的契约说明。
- 整曲均匀取 8 个位置，通过 `library_audio_read_ranges` 一次流式调用完成（host 只打开一次文件）：每处请求 5 秒窗口、最多读取 5,000 个采样点（总计 40,000，在 host 64,000 上限内），汇总全曲采样计算 RMS。
- 读取失败或无采样点的曲目会被跳过（不写入结果），不会用假数据（如 0）污染标签。
- `backend.tag.wiki` 与 `backend.wiki` 从插件的 `i18n/en-US.json`、`zh-CN.json`、`zh-CLASSICAL.json` 返回 Markdown。说明包含分数区间、计算方式及局限，未知语言回退到英文。
