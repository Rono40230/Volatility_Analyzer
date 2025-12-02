<template>
  <div class="container">
    <div class="controls">
      <label for="pair-select">Paire:</label>
      <select 
        id="pair-select"
        v-model="selected" 
        @change="load"
        class="pair-select"
      >
        <option value="">-- Choisir --</option>
        <option v-for="p in pairs" :key="p" :value="p">{{ p }}</option>
      </select>
      <label for="event-type-select">Type d'événement:</label>
      <select 
        id="event-type-select"
        v-model="selectedEventType" 
        @change="load"
        class="pair-select"
      >
        <option value="">-- Choisir --</option>
        <option v-for="et in eventTypes" :key="et" :value="et">{{ et }}</option>
      </select>
    </div>
    <div v-if="decayLoading" class="spinner">⏳</div>
    <div v-else-if="decayError" class="error">{{ decayError }}</div>
    <div v-else-if="!decayResults" class="empty">📭 Chargez une paire et sélectionnez un événement</div>
    <div v-else class="result">
      <div class="row"><span>ATR Peak:</span><strong>{{ decayResults.peak_atr.toFixed(4) }}</strong></div>
      <div class="row"><span>Decay Rate:</span><strong>{{ decayResults.decay_rate_pips_per_minute.toFixed(2) }} pips/min</strong></div>
      <div class="row"><span>Vitesse:</span><strong>{{ decayResults.decay_speed }}</strong></div>
      <div class="row"><span>Timeout Recommandé:</span><strong>{{ decayResults.recommended_timeout_minutes }} min</strong></div>
      <div class="row"><span>Basé sur:</span><strong>{{ decayResults.event_count }} événements {{ decayResults.event_type }}</strong></div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

interface Symbol { symbol: string; file_path?: string }
interface Candle { open: number; high: number; low: number; close: number; volume: number }

const { decayLoading, decayError, decayResults, analyzeDecayProfile, eventTypes, loadEventTypes } = useRetrospectiveAnalysis()
const pairs = ref<string[]>([]), selected = ref(''), selectedEventType = ref('')

onMounted(async () => {
  try { const s = await invoke<Symbol[]>('load_symbols'); pairs.value = s.map((x: Symbol) => x.symbol) } catch (e) { pairs.value = ['EURUSD'] }
  await loadEventTypes()
})

async function load() {
  if (!selected.value || !selectedEventType.value) return
  try { const c = await invoke<Candle[]>('get_pair_candles', { symbol: selected.value }); await analyzeDecayProfile(c, selectedEventType.value) } catch (e) { decayError.value = String(e) }
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { margin-bottom: 15px; display: flex; gap: 20px; align-items: flex-end; flex-wrap: wrap; }
label { display: block; color: #e2e8f0; font-weight: 600; margin-bottom: 8px; }
.pair-select { width: 200px; padding: 12px 16px; font-size: 1.1em; border: 2px solid #4a5568; border-radius: 8px; background: #2d3748; color: #000000 !important; cursor: pointer; transition: all 0.3s; }
.pair-select:hover { border-color: #667eea; background: #374151; }
.pair-select:focus { outline: none; border-color: #667eea; box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2); }
.pair-select option { background: #ffffff; color: #000000 !important; }
.spinner, .empty { text-align: center; color: #8b949e; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
.result { display: flex; flex-direction: column; gap: 10px; }
.row { display: flex; justify-content: space-between; padding: 10px; background: #161b22; border-radius: 4px; }
strong { color: #1f6feb; font-weight: 700; }
</style>
