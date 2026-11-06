<script setup lang="ts">
import type { Evidence } from './api/types/workbench'
import { ZButton, ZCard, ZConfigProvider, ZEmpty, ZInput, ZTag } from '@codex-proxy/ui'
import { zhHans } from '@codex-proxy/ui/locale'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getSnapshot } from './api'

defineProps<{ preview: boolean }>()
const entries = ref<Evidence[]>([])
const keyFilter = ref('')
const modelFilter = ref('')
const loading = ref(true)
const error = ref('')
let timer: ReturnType<typeof setInterval> | undefined
async function refresh() {
  try {
    const snapshot = await getSnapshot()
    entries.value = snapshot.facts.entries.filter(item => item.capability === 'observer' && item.event === 'request_completed')
    error.value = ''
  }
  catch (cause) {
    error.value = cause instanceof Error ? cause.message : '读取请求日志失败'
  }
  finally {
    loading.value = false
  }
}
const filtered = computed(() => entries.value.filter((item) => {
  const key = String(item.details.clientKeyName ?? item.details.clientKeyId ?? '')
  const model = item.model ?? ''
  return (!keyFilter.value || key.toLowerCase().includes(keyFilter.value.toLowerCase()))
    && (!modelFilter.value || model.toLowerCase().includes(modelFilter.value.toLowerCase()))
}))
const stats = computed(() => ({
  total: entries.value.length,
  passed: entries.value.filter(item => item.outcome === 'passed').length,
  failed: entries.value.filter(item => item.outcome === 'failed').length,
  models: new Set(entries.value.map(item => item.model).filter(Boolean)).size,
}))
function formatTime(ms: number) {
  return new Date(ms).toLocaleTimeString()
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
        <span class="hint">自动刷新 · 按密钥名称和模型筛选</span>
        <span class="live"><i /> LIVE · 1s</span>
      </div>
      <section class="summary-grid" aria-label="请求摘要">
        <ZCard as="article" padding="compact" class="summary-card active">
          <span class="summary-label">全部请求</span><strong>{{ stats.total }}</strong><small>最近 64 条</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">成功</span><strong>{{ stats.passed }}</strong><small>已完成请求</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card danger">
          <span class="summary-label">失败</span><strong>{{ stats.failed }}</strong><small>需要关注</small>
        </ZCard>
        <ZCard as="article" padding="compact" class="summary-card">
          <span class="summary-label">模型</span><strong>{{ stats.models }}</strong><small>已观测模型</small>
        </ZCard>
      </section>
      <ZCard as="section" padding="none" class="log-panel" aria-label="请求日志">
        <div class="filters">
          <ZInput v-model="keyFilter" class="filter-input" placeholder="搜索密钥名称" aria-label="按密钥名称筛选" />
          <ZInput v-model="modelFilter" class="filter-input" placeholder="搜索模型" aria-label="按模型筛选" />
          <ZButton class="filter-clear" variant="plain" size="small" @click="keyFilter = ''; modelFilter = ''">
            清除筛选
          </ZButton>
          <span class="count">匹配 {{ filtered.length }} 条</span>
        </div>
        <p v-if="loading" class="empty-state">
          正在连接实时日志…
        </p><p v-else-if="error" class="empty-state error">
          {{ error }}
        </p>
        <div v-else-if="filtered.length" class="table-wrap">
          <table>
            <thead><tr><th>时间</th><th>密钥名称</th><th>模型</th><th>请求 ID</th><th>状态</th></tr></thead><tbody>
              <tr v-for="item in filtered.slice().reverse()" :key="item.id">
                <td data-label="时间" class="muted">
                  {{ formatTime(item.occurredAtMs) }}
                </td><td data-label="密钥名称" class="strong">
                  {{ item.details.clientKeyName || item.details.clientKeyId || '—' }}
                </td><td data-label="模型">
                  {{ item.model || '—' }}
                </td><td data-label="请求 ID" class="mono">
                  {{ item.requestId || '—' }}
                </td><td data-label="状态">
                  <ZTag class="status" :type="item.outcome === 'passed' ? 'success' : 'danger'" size="small" round>
                    {{ item.outcome === 'passed' ? '成功' : '失败' }}
                  </ZTag>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <ZEmpty v-else title="暂无匹配请求" description="发起请求后日志会实时出现在这里。" surface="none" />
      </ZCard>
    </main>
  </ZConfigProvider>
</template>
