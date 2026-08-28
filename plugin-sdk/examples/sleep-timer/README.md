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
Compress-Archive -LiteralPath @('plugin.json', 'backend.wasm', 'icon.svg', 'ui') -DestinationPath ..\sleep-timer.dropin -Force
Pop-Location
```