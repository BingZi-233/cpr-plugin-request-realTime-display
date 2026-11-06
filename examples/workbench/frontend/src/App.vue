<script setup lang="ts">
import type { Evidence } from './api/types/workbench'
import { ZConfigProvider } from '@codex-proxy/ui'
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
    entries.value = snapshot.facts.entries.filter(item => item.capability === 'observer')
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
      <header class="hero">
        <div>
          <p class="eyebrow">
            CODEX PROXY PLUGIN
          </p><h1>实时请求日志</h1><p class="subtitle">
            请求到达后自动刷新，支持按密钥名称和模型筛选。
          </p>
        </div><span class="live"><i /> LIVE · 1s</span>
      </header>
      <section class="filters" aria-label="日志筛选">
        <input v-model="keyFilter" placeholder="密钥名称" aria-label="按密钥名称筛选"><input v-model="modelFilter" placeholder="模型" aria-label="按模型筛选"><button type="button" @click="keyFilter = ''; modelFilter = ''">
          清除筛选
        </button><span class="count">{{ filtered.length }} 条</span>
      </section>
      <p v-if="loading" class="empty">
        正在连接实时日志…
      </p><p v-else-if="error" class="empty error">
        {{ error }}
      </p>
      <div v-else-if="filtered.length" class="table-wrap">
        <table>
          <thead><tr><th>时间</th><th>密钥名称</th><th>模型</th><th>请求 ID</th><th>状态</th><th>事件</th></tr></thead><tbody>
            <tr v-for="item in filtered.slice().reverse()" :key="item.id">
              <td class="muted">
                {{ formatTime(item.occurredAtMs) }}
              </td><td class="strong">
                {{ item.details.clientKeyName || item.details.clientKeyId || '—' }}
              </td><td>{{ item.model || '—' }}</td><td class="mono">
                {{ item.requestId || '—' }}
              </td><td><span class="status" :class="item.outcome">{{ item.outcome === 'passed' ? '成功' : '失败' }}</span></td><td>{{ item.event }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-else class="empty">
        暂无匹配请求，发起请求后日志会实时出现在这里。
      </p>
    </main>
  </ZConfigProvider>
</template>
