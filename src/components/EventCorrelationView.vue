<template>
  <div class="main-container">
    <div class="view-mode-selector">
      <button class="mode-button" :class="{ active: viewMode === 'by-event' }" @click="viewMode = 'by-event'">📅 Par Événement</button>
      <button class="mode-button" :class="{ active: viewMode === 'by-pair' }" @click="viewMode = 'by-pair'">💱 Par Paire</button>
      <button class="mode-button" :class="{ active: viewMode === 'heatmap' }" @click="viewMode = 'heatmap'">🔥 Heatmap</button>
    </div>
    <div class="content-area">
      <EventCorrelationByEvent v-if="viewMode === 'by-event'" :pastEventsHigh="pastEventsHigh" :pastEventsMedium="pastEventsMedium" />
      <EventCorrelationByPair v-if="viewMode === 'by-pair'" :availablePairs="availablePairs" />
      <EventCorrelationHeatmap v-if="viewMode === 'heatmap'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDataRefresh } from '../composables/useDataRefresh'
import EventCorrelationByEvent from './EventCorrelationByEvent.vue'
import EventCorrelationByPair from './EventCorrelationByPair.vue'
import EventCorrelationHeatmap from './EventCorrelationHeatmap.vue'

interface PastEvent {
  name: string
  count: number
  impact: string
}

const viewMode = ref<'by-event' | 'by-pair' | 'heatmap'>('by-event')
const pastEventsHigh = ref<PastEvent[]>([])
const pastEventsMedium = ref<PastEvent[]>([])
const availablePairs = ref<string[]>([])

const { onPairDataRefresh } = useDataRefresh()
const unsubscribe = onPairDataRefresh(loadAvailablePairs)
onBeforeUnmount(() => unsubscribe())

onMounted(async () => {
  await loadPastEvents()
  await loadAvailablePairs()
})

async function loadPastEvents() {
  try {
    const result = await invoke<{ high: PastEvent[], medium: PastEvent[] }>('get_past_events', { monthsBack: 6 })
    pastEventsHigh.value = result.high
    pastEventsMedium.value = result.medium
  } catch (error) {
    console.error('Erreur:', error)
  }
}

async function loadAvailablePairs() {
  try {
    const symbolData = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
    availablePairs.value = symbolData.map(item => item.symbol)
  } catch (error) {
    console.error('Erreur:', error)
    availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
  }
}
</script>

<style scoped>
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  color: #e2e8f0;
}

.view-mode-selector {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 2px solid #30363d;
}

.mode-button {
  flex: 1;
  padding: 15px 20px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1.1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.mode-button:hover {
  background: #1c2128;
  border-color: #58a6ff;
  color: #58a6ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.3);
}

.mode-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 4px 12px rgba(88, 166, 255, 0.4);
}

.content-area {
  padding: 30px;
  min-height: 400px;
}
</style>
