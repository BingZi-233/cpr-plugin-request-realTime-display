import type { Evidence, RequestLogEntry, WorkbenchExample, WorkbenchKey, WorkbenchSnapshot } from '../types/workbench'
import { exampleStatuses } from '@/constants/workbench'
import { nonNegativeInteger, nullableString, record, string, stringArray } from '../validation'

function parseKey(value: unknown): WorkbenchKey {
  const item = record(value, 'Key')
  if (typeof item.enabled !== 'boolean')
    throw new Error('Key 状态格式无效')
  return { id: string(item.id, 'Key ID'), name: string(item.name, 'Key 名称'), enabled: item.enabled }
}

function parseEvidence(value: unknown): Evidence {
  const item = record(value, '执行事实')
  if (item.outcome !== 'passed' && item.outcome !== 'failed')
    throw new Error('执行事实结果格式无效')
  return {
    id: string(item.id, '事实 ID'),
    capability: string(item.capability, '能力'),
    event: string(item.event, '事件'),
    outcome: item.outcome,
    occurredAtMs: nonNegativeInteger(item.occurredAtMs, '事件时间'),
    requestId: nullableString(item.requestId, '请求 ID'),
    provider: nullableString(item.provider, 'Provider'),
    accountId: nullableString(item.accountId, '账号 ID'),
    model: nullableString(item.model, '模型'),
    details: record(item.details, '事实详情'),
  }
}

function nullableNumber(value: unknown, label: string): number | null {
  if (value == null)
    return null
  return nonNegativeInteger(value, label)
}

function parseRequest(value: unknown): RequestLogEntry {
  const item = record(value, '请求日志')
  if (item.status !== 'running' && item.status !== 'completed' && item.status !== 'error')
    throw new Error('请求状态格式无效')
  return {
    id: string(item.id, '请求 ID'),
    startedAtMs: nonNegativeInteger(item.startedAtMs, '开始时间'),
    updatedAtMs: nonNegativeInteger(item.updatedAtMs, '更新时间'),
    status: item.status,
    outcome: nullableString(item.outcome, '终态'),
    clientKeyId: nullableString(item.clientKeyId, '密钥 ID'),
    clientKeyName: nullableString(item.clientKeyName, '密钥名称'),
    model: nullableString(item.model, '模型'),
    errorCode: nullableString(item.errorCode, '错误码'),
    inputTokens: nullableNumber(item.inputTokens, '输入 Token'),
    outputTokens: nullableNumber(item.outputTokens, '输出 Token'),
    cachedTokens: nullableNumber(item.cachedTokens, '缓存读取 Token'),
    cacheWriteTokens: nullableNumber(item.cacheWriteTokens, '缓存写入 Token'),
    totalTokens: nullableNumber(item.totalTokens, '总 Token'),
    upstreamResponseMs: nullableNumber(item.upstreamResponseMs, '上游耗时'),
    latencyMs: nullableNumber(item.latencyMs, '总耗时'),
  }
}

function parseExample(value: unknown): WorkbenchExample {
  const item = record(value, '基础示例')
  if (typeof item.status !== 'string' || !exampleStatuses.has(item.status))
    throw new Error('基础示例状态格式无效')
  return {
    id: string(item.id, '示例 ID'),
    title: string(item.title, '示例标题'),
    capabilities: stringArray(item.capabilities, '能力列表'),
    status: item.status as WorkbenchExample['status'],
    runCount: nonNegativeInteger(item.runCount, '运行次数'),
    lastEvidenceId: nullableString(item.lastEvidenceId, '事实 ID'),
    trigger: string(item.trigger, '触发方式'),
  }
}

export function parseSnapshot(input: unknown): WorkbenchSnapshot {
  const value = record(input, '工作台快照')
  const facts = record(value.facts, '执行事实')
  const latest = record(facts.latest, '最新执行事实')
  if (value.contractVersion !== 1)
    throw new Error('工作台合同版本不受支持')
  if (!Array.isArray(value.keys) || !Array.isArray(value.examples) || !Array.isArray(facts.entries) || !Array.isArray(value.requests))
    throw new Error('工作台快照格式无效')
  return {
    contractVersion: 1,
    keys: value.keys.map(parseKey),
    keysNextCursor: nullableString(value.keysNextCursor, 'Key 游标'),
    examples: value.examples.map(parseExample),
    facts: {
      latest: Object.fromEntries(Object.entries(latest).map(([key, item]) => [key, parseEvidence(item)])),
      entries: facts.entries.map(parseEvidence),
    },
    requests: value.requests.map(parseRequest),
  }
}
