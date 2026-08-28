# Dropin WASM Rust SDK

此 SDK 提供 Rust 插件后端.

## Sample

```rust
use dropin_wasm_sdk::{dropin_plugin, PluginResult, Request};
use serde_json::json;

dropin_plugin!(handle);

fn handle(request: Request) -> PluginResult {
    match request.method.strip_prefix("backend.").unwrap_or(&request.method) {
        "hello" => Ok(json!({ "message": "Hello from WASM" })),
        method => Err(format!("unknown backend method: {method}")),
    }
}
```

构建命令:

```bash
cargo build --manifest-path plugin-sdk/examples/your-plugin/backend/Cargo.toml --release --target wasm32-unknown-unknown
```

将生成的 wasm 文件复制到插件清单的 backend 路径.

## 从 WASM 调用 Dropin

SDK 暴露了与 JavaScript SDK 相同的 Host API 命名空间:

```rust
use dropin_wasm_sdk::host;

host::player_pause()?;
host::notification_show("Paused", "Sleep timer finished.", 8000)?;
```

这些调用使用插件声明并获得的权限. 例如: `player_pause` 需要 `player.control` 权限, `notification_show` 需要 `notification.show` 权限.

## 后台定时

如果清单声明了:

```json
{
    "background": { "tickIntervalMs": 1000 }
}
```

当插件启用时, 即使插件 UI iframe 未打开, Dropin 也会调用 `backend.tick`. tick 接收一个以毫秒为单位的 nowMs 时间戳.