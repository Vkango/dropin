# 睡眠定时器插件

## 构建 backend.wasm

```powershell
$targetDir = Join-Path $env:TEMP 'dropin-sleep-timer-target'
cargo build --manifest-path plugin-sdk/examples/sleep-timer/backend/Cargo.toml --release --target wasm32-unknown-unknown --target-dir $targetDir
Copy-Item -LiteralPath (Join-Path $targetDir 'wasm32-unknown-unknown\release\dropin_sleep_timer_backend.wasm') -Destination plugin-sdk\examples\sleep-timer\backend.wasm -Force
```

## 打包
```powershell
Push-Location plugin-sdk\examples\sleep-timer
Compress-Archive -LiteralPath @('plugin.json', 'backend.wasm', 'icon.svg', 'ui', 'i18n') -DestinationPath ..\sleep-timer.zip -Force
Pop-Location
Move-Item -LiteralPath plugin-sdk\examples\sleep-timer.zip -Destination plugin-sdk\examples\sleep-timer.dropin -Force
```

插件实现了 `backend.wiki`，按 Dropin 传入的 `locale` 从自身的 `i18n` 文件选择 Markdown，在 Plugins 的 Wiki Tab 中展示。读取说明不需要启用插件或授予权限。
