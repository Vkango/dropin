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

## Tag Provider 契约

在清单中声明 `tagProvider` (并授予 `tag.provider`、`library.read`、`library.audio.read` 权限) 后, Dropin 会**逐首**调用插件的 `backend.tag.analyze` 方法 (每次一个 wasm 调用, 独立 fuel 预算):

请求 args:

```json
{
    "providerKey": "energy",
    "trackIndex": 0,
    "track": { "id": "track-id", "title": "...", "durationMs": 213000, "...": "..." }
}
```

插件应返回 (读取失败的曲目直接跳过, 返回空 results 即可):

```json
{
    "results": [
        { "trackId": "track-id", "key": "energy", "value": 72.5 }
    ]
}
```

- `key` 是本次写入的标签键名, `value` 的类型必须匹配清单中的 `valueType`.
- 当 `supportsSegments` 为 true 时, 每条结果可携带 `startMs`/`endMs`, 或用 `segments: [{ startMs, endMs, value }]` 写入分段值.
- 音频读取使用 `host::library_audio_read_ex(&track_id, start_ms, end_ms, Some(max_samples))`; 请保持较小的读取窗口, host 响应上限为 1 MiB.

## 插件 Wiki

任何插件都可以实现 `backend.wiki`，Tag Provider 还可以实现 `backend.tag.wiki`。Wiki 读取不需要额外权限，禁用的插件也可以展示说明。Dropin 会在无 Host API 访问权的独立 WASM 实例中调用这些方法，不影响正在运行的插件。请将 Wiki 保存在插件自己的 `i18n` 文件中，例如用 `include_str!` 编入后端，并按传入的显示语言选择内容。

普通插件收到 `{ "locale": "zh-CN" }`，Tag Provider 收到 `{ "providerKey": "energy", "locale": "zh-CN" }`。支持的内置显示语言为 `en-US`、`zh-CN`、`zh-CLASSICAL`；未知语言可回退到英文。返回 Markdown:

```json
{ "wiki": "## What is Energy?\n\n**Energy** scores ..." }
```

Wiki 内容随插件分发，不要硬编码在主程序里。用户在「我的标签」创建标签、选择关联 Provider 时，Dropin 自动读取该语言的 Wiki 填入编辑框；用户可以修改，也可以清空。创建后，保存的 Wiki 归属于标签，插件升级、停用或显示语言切换不会替换它。人工标签可随时编辑并保存 Wiki；关联插件的标签展示 `Original | Plugin Provided`，其中 Original 是可编辑的标签副本，Plugin Provided 是只读的插件当前语言说明，两者互不覆盖。

普通插件的 Wiki 显示于 Plugins 项目的 `Overview | Wiki | Permissions` 卡片中。未实现 Wiki 方法或返回空字符串时，界面显示暂无 Wiki。
