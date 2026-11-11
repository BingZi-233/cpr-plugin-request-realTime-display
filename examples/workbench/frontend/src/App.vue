<script setup lang="ts">
import type { RequestLogEntry } from './api/types/workbench'
import { ZCard, ZConfigProvider, ZEmpty, ZTag } from '@codex-proxy/ui'
import { zhHans } from '@codex-proxy/ui/locale'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getSnapshot } from './api'

defineProps<{ preview: boolean }>()
const entries = ref<RequestLogEntry[]>([])
const loading = ref(true)
const error = ref('')
let timer: ReturnType<typeof setInterval> | undefined

async function refresh() {
  try {
    const snapshot = await getSnapshot()
    entries.value = snapshot.requests
    error.value = ''
  }
  catch (cause) {
    error.value = cause instanceof Error ? cause.message : '读取请求日志失败'
  }
  finally {
    loading.value = false
  }
}

const rows = computed(() => entries.value.slice().reverse())
const counts = computed(() => ({
  running: entries.value.filter(item => item.status === 'running').length,
  completed: entries.value.filter(item => item.status === 'completed').length,
  failed: entries.value.filter(item => item.status === 'error').length,
}))
const rates = computed(() => {
  let cached = 0
  let cacheInput = 0
  let written = 0
  let writeInput = 0
  let output = 0
  let durationMs = 0
  for (const item of entries.value) {
    if (item.status === 'running')
      continue
    if (item.inputTokens && item.cachedTokens != null) {
      cacheInput += item.inputTokens
      cached += item.cachedTokens
    }
    if (item.inputTokens && item.cacheWriteTokens != null) {
      writeInput += item.inputTokens
      written += item.cacheWriteTokens
    }
    const ms = speedMs(item)
    if (item.outputTokens && ms) {
      output += item.outputTokens
      durationMs += ms
    }
  }
  return {
    inputCache: cacheInput > 0 ? cached / cacheInput : null,
    writeCache: writeInput > 0 ? written / writeInput : null,
    tokensPerSecond: durationMs > 0 ? (output * 1000) / durationMs : null,
  }
})

function speedMs(item: RequestLogEntry) {
  if (item.upstreamResponseMs && item.upstreamResponseMs > 0)
    return item.upstreamResponseMs
  if (item.latencyMs && item.latencyMs > 0)
    return item.latencyMs
  return null
}

function tokensPerSecond(item: RequestLogEntry) {
  const ms = speedMs(item)
  if (!item.outputTokens || !ms)
    return null
  return (item.outputTokens * 1000) / ms
}

function cacheRate(part: number | null, whole: number | null) {
  if (part == null || !whole)
    return null
  return part / whole
}

function formatSpeed(value: number | null) {
  if (value == null)
    return '—'
  return value >= 100 ? value.toFixed(0) : value.toFixed(1)
}

function formatPercent(value: number | null) {
  if (value == null)
    return '—'
  return `${(value * 100).toFixed(1)}%`
}

function formatTokens(value: number | null) {
  if (value == null)
    return '—'
  return new Intl.NumberFormat('zh-CN').format(value)
}

function formatTime(ms: number) {
  return new Date(ms).toLocaleTimeString('zh-CN', { hour12: false })
}

function statusText(item: RequestLogEntry) {
  if (item.status === 'running')
    return '进行中'
  switch (item.outcome) {
    case 'succeeded': return '完成'
    case 'failed': return '失败'
    case 'rejected': return '拒绝'
    case 'cancelled': return '取消'
    case 'incomplete': return '未完成'
    default: return '错误'
  }
}

function statusType(item: RequestLogEntry): 'warning' | 'success' | 'danger' {
  if (item.status === 'running')
    return 'warning'
  if (item.status === 'completed')
    return 'success'
  return 'danger'
}

function keyName(item: RequestLogEntry) {
  return item.clientKeyName || item.clientKeyId || '—'
}

onMounted(() => {
  refresh()
  timer = setInterval(refresh, 1000)
})
onUnmounted(() => {
  if (timer)
    clearInterval(timer)
})
</script>

<template>
  <ZConfigProvider :locale="zhHans">
    <main class="log-page">
      <div class="toolbar">
        <span class="hint">只记录对话请求，完成或出错时更新同一条</span>
        <span class="count">{{ rows.length }} 条</span>
        <span class="live"><i /> LIVE · 1s</span>
      </div>
      <section class="summary-grid" aria-label="请求统计">
        <ZCard as="article" padding="compact" class="summary-card active">
          <span class="summary-label">进行中</span>
          <strong>{{ counts.running }}</strong>
          <small>尚未结束</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">完成</span>
          <strong>{{ counts.completed }}</strong>
          <small>成功终态</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card danger">
          <span class="summary-label">错误</span>
          <strong>{{ counts.failed }}</strong>
          <small>失败、拒绝、取消</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">tok/s</span>
          <strong>{{ formatSpeed(rates.tokensPerSecond) }}</strong>
          <small>输出 / 上游耗时</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">输入缓存率</span>
          <strong>{{ formatPercent(rates.inputCache) }}</strong>
          <small>读取 / 输入</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">写入缓存率</span>
          <strong>{{ formatPercent(rates.writeCache) }}</strong>
          <small>写入 / 输入</small>
        </ZCard>
      </section>
      <ZCard as="section" padding="none" class="log-panel" aria-label="请求日志">
        <p v-if="loading" class="empty-state">
          正在连接实时日志…
        </p>
        <p v-else-if="error" class="empty-state error">
          {{ error }}
        </p>
        <div v-else-if="rows.length" class="table-wrap">
          <table>
            <colgroup>
              <col class="col-time">
              <col class="col-status">
              <col class="col-key">
              <col class="col-model">
              <col class="col-num">
              <col class="col-num">
              <col class="col-num">
              <col class="col-num">
              <col class="col-num">
              <col class="col-id">
            </colgroup>
            <thead>
              <tr>
                <th>时间</th>
                <th>状态</th>
                <th>密钥</th>
                <th>模型</th>
                <th class="num">
                  输入
                </th>
                <th class="num">
                  输出
                </th>
                <th class="num">
                  tok/s
                </th>
                <th class="num">
                  输入缓存
                </th>
                <th class="num">
                  写入缓存
                </th>
                <th>请求 ID</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in rows" :key="item.id" :class="{ running: item.status === 'running' }">
                <td data-label="时间" class="muted" :title="formatTime(item.startedAtMs)">
                  {{ formatTime(item.startedAtMs) }}
                </td>
                <td data-label="状态" :title="item.errorCode || statusText(item)">
                  <ZTag class="status" :type="statusType(item)" size="small" round>
                    {{ statusText(item) }}
                  </ZTag>
                </td>
                <td data-label="密钥" class="strong" :title="keyName(item)">
                  {{ keyName(item) }}
                </td>
                <td data-label="模型" :title="item.model || '—'">
                  {{ item.model || '—' }}
                </td>
                <td data-label="输入" class="num">
                  {{ formatTokens(item.inputTokens) }}
                </td>
                <td data-label="输出" class="num">
                  {{ formatTokens(item.outputTokens) }}
                </td>
                <td data-label="tok/s" class="num">
                  {{ formatSpeed(tokensPerSecond(item)) }}
                </td>
                <td data-label="输入缓存" class="num">
                  {{ formatPercent(cacheRate(item.cachedTokens, item.inputTokens)) }}
                </td>
                <td data-label="写入缓存" class="num">
                  {{ formatPercent(cacheRate(item.cacheWriteTokens, item.inputTokens)) }}
                </td>
                <td data-label="请求 ID" class="mono" :title="item.id">
                  {{ item.id }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <ZEmpty v-else title="暂无请求" description="请求开始后会出现在这里，并在完成或出错时更新。" surface="none" />
      </ZCard>
    </main>
  </ZConfigProvider>
</template>
