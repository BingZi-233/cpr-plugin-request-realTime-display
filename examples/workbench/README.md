# 实时请求日志插件

此目录包含插件的 Rust 后端和 Vue 管理页面。

- `backend/src/request/log.rs` 按请求 ID 保存日志。中间件开始时写入进行中，下游直接失败时标为错误，`RequestCompleted` 到达后改写同一条并填入用量。
- `backend/src/management/workbench.rs` 提供 `GET api/snapshot`，返回最近请求和当前 Key 名称。
- `frontend/src/App.vue` 每秒刷新，铺满可用宽度，展示进行中、完成、错误以及 tok/s 和缓存率。

插件不保存请求正文和密钥原文，请求日志保留最近 200 条。页面通过 `window.codexProxyPlugin` 宿主桥接访问管理接口。
