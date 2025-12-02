<template>
  <div class="container">
    <div class="controls">
      <label>Paire: <select v-model="selected" @change="load" class="input"><option value="">-- Choisir --</option><option v-for="p in pairs" :key="p" :value="p">{{ p }}</option></select></label>
    </div>
    <div v-if="peakDelayLoading" class="spinner">⏳</div>
    <div v-else-if="peakDelayError" class="error">{{ peakDelayError }}</div>
    <div v-else-if="peakDelayResults.length === 0" class="empty">📭 Aucune donnée</div>
    <table v-else>
      <thead><tr><th>Événement</th><th>Délai</th><th>N</th><th>%</th></tr></thead>
      <tbody><tr v-for="r in peakDelayResults" :key="r.event_type"><td>{{ r.event_type }}</td><td>{{ r.peak_delay_minutes }}</td><td>{{ r.sample_count }}</td><td>{{ r.consistency_percent.toFixed(1) }}</td></tr></tbody>
    </table>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

interface Symbol { symbol: string; file_path?: string }
interface Candle { open: number; high: number; low: number; close: number; volume: number }

const { peakDelayLoading, peakDelayError, peakDelayResults, analyzePeakDelay } = useRetrospectiveAnalysis()
const pairs = ref<string[]>([]), selected = ref('')

onMounted(async () => {
  try { const s = await invoke<Symbol[]>('load_symbols'); pairs.value = s.map((x: Symbol) => x.symbol) } catch (e) { pairs.value = ['EURUSD'] }
})

async function load() {
  if (!selected.value) return
  try { const c = await invoke<Candle[]>('load_pair_candles', { symbol: selected.value }); await analyzePeakDelay(c) } catch (e) { }
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { margin-bottom: 15px; }
.input { padding: 8px; background: #161b22; border: 1px solid #30363d; color: #e2e8f0; border-radius: 4px; }
.spinner { text-align: center; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
.empty { text-align: center; color: #8b949e; padding: 30px; }
table { width: 100%; border-collapse: collapse; background: #161b22; }
th, td { padding: 10px; text-align: left; border-bottom: 1px solid #30363d; }
th { color: #1f6feb; font-weight: 700; }
</style>
