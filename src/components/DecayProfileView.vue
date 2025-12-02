<template>
  <div class="container">
    <div class="controls">
      <label>Paire: <select v-model="selected" @change="load" class="input"><option value="">-- Choisir --</option><option v-for="p in pairs" :key="p" :value="p">{{ p }}</option></select></label>
    </div>
    <div v-if="decayLoading" class="spinner">⏳</div>
    <div v-else-if="decayError" class="error">{{ decayError }}</div>
    <div v-else-if="!decayResults" class="empty">📭 Chargez une paire</div>
    <div v-else class="result">
      <div class="row"><span>ATR Peak:</span><strong>{{ decayResults.peak_atr.toFixed(4) }}</strong></div>
      <div class="row"><span>Decay Rate:</span><strong>{{ decayResults.decay_rate_pips_per_minute.toFixed(2) }} pips/min</strong></div>
      <div class="row"><span>Vitesse:</span><strong>{{ decayResults.decay_speed }}</strong></div>
      <div class="row"><span>Timeout Recommandé:</span><strong>{{ decayResults.recommended_timeout_minutes }} min</strong></div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

interface Symbol { symbol: string; file_path?: string }
interface Candle { open: number; high: number; low: number; close: number; volume: number }

const { decayLoading, decayError, decayResults, analyzeDecayProfile } = useRetrospectiveAnalysis()
const pairs = ref<string[]>([]), selected = ref('')

onMounted(async () => {
  try { const s = await invoke<Symbol[]>('load_symbols'); pairs.value = s.map((x: Symbol) => x.symbol) } catch (e) { pairs.value = ['EURUSD'] }
})

async function load() {
  if (!selected.value) return
  try { const c = await invoke<Candle[]>('load_pair_candles', { symbol: selected.value }); await analyzeDecayProfile(c) } catch (e) { }
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { margin-bottom: 15px; }
.input { padding: 8px; background: #161b22; border: 1px solid #30363d; color: #e2e8f0; border-radius: 4px; }
.spinner, .empty { text-align: center; color: #8b949e; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
.result { display: flex; flex-direction: column; gap: 10px; }
.row { display: flex; justify-content: space-between; padding: 10px; background: #161b22; border-radius: 4px; }
strong { color: #1f6feb; font-weight: 700; }
</style>
