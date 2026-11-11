//! 按请求 ID 就地更新的内存日志。进行中的行会在终态到达后改写，而不是再追加一条。
//!
//! ponytail: 只留最近 200 条。需要更长历史时再落到宿主用量记录，而不是在插件里做分页存储。

use std::{
    collections::VecDeque,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use gateway_plugin_sdk::call::observation::{RequestCompleted, RequestOutcome};
use serde::Serialize;

const MAXIMUM_REQUESTS: usize = 200;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RequestPhase {
    Running,
    Completed,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestRecord {
    pub id: String,
    pub started_at_ms: u64,
    pub updated_at_ms: u64,
    pub status: RequestPhase,
    pub outcome: Option<String>,
    pub client_key_id: Option<String>,
    pub client_key_name: Option<String>,
    pub model: Option<String>,
    pub error_code: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub upstream_response_ms: Option<u64>,
    pub latency_ms: Option<u64>,
}

pub(crate) struct RequestSeen<'a> {
    pub request_id: &'a str,
    pub client_key_id: Option<&'a str>,
    pub model: Option<&'a str>,
}

#[derive(Default)]
pub(crate) struct RequestLog {
    records: Mutex<VecDeque<RequestRecord>>,
}

impl RequestLog {
    pub fn seen(&self, seen: RequestSeen<'_>) {
        if seen.request_id.is_empty() {
            return;
        }
        let mut records = self.lock();
        if let Some(record) = find_mut(&mut records, seen.request_id) {
            if record.status == RequestPhase::Running {
                assign(&mut record.client_key_id, seen.client_key_id);
                assign(&mut record.model, seen.model);
                record.updated_at_ms = now_ms();
            }
            return;
        }
        let now = now_ms();
        records.push_back(RequestRecord {
            id: seen.request_id.to_owned(),
            started_at_ms: now,
            updated_at_ms: now,
            status: RequestPhase::Running,
            outcome: None,
            client_key_id: None,
            client_key_name: None,
            model: None,
            error_code: None,
            input_tokens: None,
            output_tokens: None,
            cached_tokens: None,
            cache_write_tokens: None,
            total_tokens: None,
            upstream_response_ms: None,
            latency_ms: None,
        });
        let record = records.back_mut().expect("刚写入的请求");
        assign(&mut record.client_key_id, seen.client_key_id);
        assign(&mut record.model, seen.model);
        trim(&mut records);
    }

    pub fn fail(&self, request_id: &str) {
        if request_id.is_empty() {
            return;
        }
        let mut records = self.lock();
        if let Some(record) = find_mut(&mut records, request_id) {
            if record.status == RequestPhase::Running {
                record.status = RequestPhase::Error;
                record.outcome = Some("failed".to_owned());
                record.updated_at_ms = now_ms();
            }
            return;
        }
        let now = now_ms();
        records.push_back(RequestRecord {
            id: request_id.to_owned(),
            started_at_ms: now,
            updated_at_ms: now,
            status: RequestPhase::Error,
            outcome: Some("failed".to_owned()),
            client_key_id: None,
            client_key_name: None,
            model: None,
            error_code: None,
            input_tokens: None,
            output_tokens: None,
            cached_tokens: None,
            cache_write_tokens: None,
            total_tokens: None,
            upstream_response_ms: None,
            latency_ms: None,
        });
        trim(&mut records);
    }

    pub fn finish(&self, request: &RequestCompleted) {
        if request.request_id.is_empty() {
            return;
        }
        let mut records = self.lock();
        let status = match request.terminal.outcome {
            RequestOutcome::Succeeded => RequestPhase::Completed,
            RequestOutcome::Failed
            | RequestOutcome::Rejected
            | RequestOutcome::Cancelled
            | RequestOutcome::Incomplete => RequestPhase::Error,
        };
        let latency_ms = request
            .usage
            .timings
            .as_ref()
            .and_then(|timings| timings.latency_ms);
        let upstream_response_ms = request
            .usage
            .timings
            .as_ref()
            .and_then(|timings| timings.upstream_response_ms);
        if let Some(record) = find_mut(&mut records, &request.request_id) {
            record.status = status;
            record.outcome = Some(outcome_name(request.terminal.outcome).to_owned());
            record.updated_at_ms = request.completed_at_ms.max(record.updated_at_ms);
            record.error_code = request.terminal.error_code.clone();
            apply_usage(record, request, upstream_response_ms, latency_ms);
            return;
        }
        let started_at_ms = request
            .completed_at_ms
            .saturating_sub(latency_ms.unwrap_or(0));
        records.push_back(RequestRecord {
            id: request.request_id.clone(),
            started_at_ms,
            updated_at_ms: request.completed_at_ms,
            status,
            outcome: Some(outcome_name(request.terminal.outcome).to_owned()),
            client_key_id: None,
            client_key_name: None,
            model: None,
            error_code: request.terminal.error_code.clone(),
            input_tokens: None,
            output_tokens: None,
            cached_tokens: None,
            cache_write_tokens: None,
            total_tokens: None,
            upstream_response_ms,
            latency_ms,
        });
        let record = records.back_mut().expect("刚写入的完成请求");
        apply_usage(record, request, upstream_response_ms, latency_ms);
        trim(&mut records);
    }

    pub fn snapshot(&self) -> Vec<RequestRecord> {
        self.lock().iter().cloned().collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<RequestRecord>> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn apply_usage(
    record: &mut RequestRecord,
    request: &RequestCompleted,
    upstream_response_ms: Option<u64>,
    latency_ms: Option<u64>,
) {
    assign(&mut record.client_key_id, request.client_key_id.as_deref());
    assign(
        &mut record.model,
        request
            .upstream_model
            .as_deref()
            .or(request.requested_model.as_deref()),
    );
    let usage = &request.usage;
    record.input_tokens = usage.input_tokens;
    record.output_tokens = usage.output_tokens;
    record.cached_tokens = usage.cached_tokens;
    record.cache_write_tokens = usage.cache_write_tokens;
    record.total_tokens = usage.total_tokens;
    record.upstream_response_ms = upstream_response_ms;
    record.latency_ms = latency_ms;
}

fn find_mut<'a>(
    records: &'a mut VecDeque<RequestRecord>,
    request_id: &str,
) -> Option<&'a mut RequestRecord> {
    records.iter_mut().find(|record| record.id == request_id)
}

fn assign(slot: &mut Option<String>, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        *slot = Some(value.to_owned());
    }
}

fn trim(records: &mut VecDeque<RequestRecord>) {
    while records.len() > MAXIMUM_REQUESTS {
        records.pop_front();
    }
}

fn outcome_name(outcome: RequestOutcome) -> &'static str {
    match outcome {
        RequestOutcome::Succeeded => "succeeded",
        RequestOutcome::Failed => "failed",
        RequestOutcome::Rejected => "rejected",
        RequestOutcome::Cancelled => "cancelled",
        RequestOutcome::Incomplete => "incomplete",
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}
