# 实时请求日志插件

此目录包含插件的 Rust 后端和 Vue 管理页面。

- `backend/src/request/observer.rs` 接收宿主的 `RequestCompleted` 事件，记录密钥 ID、密钥名称、模型、请求 ID、终态和用量信息。
- `backend/src/management/workbench.rs` 提供 `GET api/snapshot`，把最近事件和当前 Key 名称返回给管理页。
- `frontend/src/App.vue` 每秒刷新快照，提供密钥名称和模型的即时筛选。

插件不保存请求正文和密钥原文，日志仅保留最近 64 条事件。页面通过 `window.codexProxyPlugin` 宿主桥接访问管理接口。
