//! 请求日志在中间件开始时记为进行中，终态观察到达后改写同一条。

use gateway_plugin_sdk::{Message, Stage};
use serde_json::json;

use crate::support::{Peer, result};

#[tokio::test]
async fn request_log_updates_the_same_row_from_running_to_completed() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call_with(
            "middleware.handle",
            Stage::Request,
            json!({
                "request_id": "test-request",
                "settings_sources": null,
                "client_key_id": "test-key",
                "account_group_ids": [],
                "mount": "request",
                "operation": "generate",
                "protocol": "openai",
                "endpoint": "responses",
                "transport": "http_sse",
                "model": "gpt-test",
                "headers": []
            }),
            br#"{"input":"hi"}"#.to_vec(),
            |method, _, _| {
                assert_eq!(method, "host.middleware.next");
                Ok((
                    json!({"response":"response-1","protocol":"openai","status":200,"headers":[]}),
                    Vec::new(),
                ))
            },
        )
        .await;
    result(&frame);
    assert!(matches!(peer.receive().await.message, Message::End { .. }));

    let running = peer.snapshot().await;
    let row = &running["requests"][0];
    assert_eq!(row["id"], "test-request");
    assert_eq!(row["status"], "running");
    assert_eq!(row["model"], "gpt-test");
    assert_eq!(row["clientKeyId"], "test-key");
    assert!(row["outputTokens"].is_null());

    let frame = peer
        .call(
            "observer.observe",
            Stage::Observation,
            json!({
                "event": "request_completed",
                "data": {
                    "event_id": "done",
                    "request_id": "test-request",
                    "config_revision": 1,
                    "operation": "generate",
                    "client_key_id": "test-key",
                    "upstream_model": "gpt-test",
                    "completed_at_ms": 5000,
                    "terminal": {"outcome": "succeeded", "send_state": "sent", "attempt_count": 1},
                    "usage": {
                        "input_tokens": 100,
                        "output_tokens": 40,
                        "cached_tokens": 25,
                        "cache_write_tokens": 10,
                        "total_tokens": 140,
                        "timings": {"upstream_response_ms": 2000, "latency_ms": 2200}
                    }
                }
            }),
            Vec::new(),
        )
        .await;
    result(&frame);

    let done = peer.snapshot().await;
    assert_eq!(done["requests"].as_array().unwrap().len(), 1);
    let row = &done["requests"][0];
    assert_eq!(row["status"], "completed");
    assert_eq!(row["outcome"], "succeeded");
    assert_eq!(row["inputTokens"], 100);
    assert_eq!(row["outputTokens"], 40);
    assert_eq!(row["cachedTokens"], 25);
    assert_eq!(row["cacheWriteTokens"], 10);
    assert_eq!(row["upstreamResponseMs"], 2000);
    assert_eq!(row["latencyMs"], 2200);
}

#[tokio::test]
async fn request_log_keeps_an_error_row_when_the_downstream_fails() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call_with(
            "middleware.handle",
            Stage::Request,
            json!({
                "request_id": "test-request",
                "settings_sources": null,
                "client_key_id": "test-key",
                "account_group_ids": [],
                "mount": "request",
                "operation": "generate",
                "protocol": "openai",
                "endpoint": "responses",
                "transport": "http_sse",
                "model": "gpt-test",
                "headers": []
            }),
            br#"{"input":"hi"}"#.to_vec(),
            |method, _, _| {
                assert_eq!(method, "host.middleware.next");
                Ok((
                    json!({"response":"response-1","protocol":"openai","status":503,"headers":[]}),
                    Vec::new(),
                ))
            },
        )
        .await;
    result(&frame);
    assert!(matches!(peer.receive().await.message, Message::End { .. }));
    assert_eq!(peer.snapshot().await["requests"][0]["status"], "error");

    let frame = peer
        .call(
            "observer.observe",
            Stage::Observation,
            json!({
                "event": "request_completed",
                "data": {
                    "event_id": "failed",
                    "request_id": "test-request",
                    "config_revision": 1,
                    "operation": "generate",
                    "completed_at_ms": 900,
                    "terminal": {
                        "outcome": "failed",
                        "send_state": "sent",
                        "attempt_count": 1,
                        "error_code": "upstream_unavailable"
                    },
                    "usage": {"output_tokens": 0, "input_tokens": 12}
                }
            }),
            Vec::new(),
        )
        .await;
    result(&frame);
    let snapshot = peer.snapshot().await;
    let requests = snapshot["requests"].as_array().unwrap();
    assert_eq!(requests.len(), 1);
    let row = &requests[0];
    assert_eq!(row["status"], "error");
    assert_eq!(row["outcome"], "failed");
    assert_eq!(row["errorCode"], "upstream_unavailable");
    assert_eq!(row["inputTokens"], 12);
}

#[tokio::test]
async fn request_log_ignores_model_catalog_queries() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call_with(
            "middleware.handle",
            Stage::Request,
            json!({
                "request_id": "test-request",
                "settings_sources": null,
                "client_key_id": "test-key",
                "account_group_ids": [],
                "mount": "request",
                "operation": "query",
                "protocol": "openai",
                "endpoint": "/v1/models",
                "transport": "http_json",
                "headers": []
            }),
            Vec::new(),
            |method, _, _| {
                assert_eq!(method, "host.middleware.next");
                Ok((
                    json!({"response":"response-1","protocol":"openai","status":503,"headers":[]}),
                    Vec::new(),
                ))
            },
        )
        .await;
    result(&frame);
    assert!(matches!(peer.receive().await.message, Message::End { .. }));
    assert!(
        peer.snapshot().await["requests"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let frame = peer
        .call(
            "observer.observe",
            Stage::Observation,
            json!({
                "event": "request_completed",
                "data": {
                    "event_id": "models",
                    "request_id": "test-request",
                    "config_revision": 1,
                    "operation": "query",
                    "completed_at_ms": 1,
                    "terminal": {"outcome": "succeeded", "send_state": "sent", "attempt_count": 1},
                    "usage": {}
                }
            }),
            Vec::new(),
        )
        .await;
    result(&frame);
    assert!(
        peer.snapshot().await["requests"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
