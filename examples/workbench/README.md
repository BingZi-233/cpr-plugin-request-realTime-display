# 实时请求日志插件

此目录包含插件的 Rust 后端和 Vue 管理页面。

- `backend/src/request/log.rs` 只保存 `generate` 对话请求。模型列表等 `query` 不进入。进行中的行在完成或失败后改写。
- `backend/src/management/workbench.rs` 提供 `GET api/snapshot`，返回最近请求和当前 Key 名称。
- `frontend/src/App.vue` 每秒刷新，铺满可用宽度，展示进行中、完成、错误以及 tok/s 和缓存率。

插件不保存请求正文和密钥原文，请求日志保留最近 200 条。页面通过 `window.codexProxyPlugin` 宿主桥接访问管理接口。
