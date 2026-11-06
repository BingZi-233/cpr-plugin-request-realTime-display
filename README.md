# Codex Proxy 实时请求日志插件

这是一个面向 [codex-proxy-rs](https://github.com/zyycn/codex-proxy-rs) 的可信进程插件。它订阅宿主的请求完成事件，在插件管理页实时展示请求日志，并支持按**密钥名称**和**模型**过滤。

## 在 CPR 中安装

不要直接上传 GitHub 源码目录。打开 CPR 的「插件管理 → 安装 → GitHub」，填写 `BingZi-233/cpr-plugin-request-realTime-display`，选择 `v1.0.0`，再确认安装对应平台的 Release 安装包。当前 Release 提供 macOS Apple Silicon 包。

## 特性

- 通过 SDK `observer` 能力接收请求完成事件，不读取密钥原文或请求正文
- 管理页每秒刷新，展示时间、密钥名称、模型、请求 ID、状态和事件
- 筛选在浏览器端即时生效，后端保留最近 64 条事件
- 页面使用宿主桥接 API，安装后无需额外 HTTP 服务

## 开发

环境要求：Rust 1.99.1、Node.js 24、pnpm 12.6。

```bash
pnpm --dir examples/workbench/frontend install --frozen-lockfile
pnpm --dir examples/workbench/frontend dev
RUST_MIN_STACK=16777216 cargo test --manifest-path examples/workbench/backend/Cargo.toml --locked
```

构建安装包：

```bash
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package
```

插件清单位于 `examples/workbench/plugin.json`，后端入口为 `examples/workbench/backend/src/app.rs`，前端入口为 `examples/workbench/frontend/src/App.vue`。
