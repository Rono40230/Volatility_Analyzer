<template>
  <div
    v-if="props.result"
    class="analysis-panel"
  >
    <!-- Panel Header: All on same line - Title + Candles Info + Date Controls -->
    <div class="panel-header">
      <!-- Title and Symbol Selector -->
      <div class="header-title">
        <h2>🎯 Analyse: {{ props.result.symbol }}</h2>
        <select
          :value="currentSymbol"
          class="symbol-select"
          @change="(e) => onSymbolChange((e.target as HTMLSelectElement).value)"
        >
          <option
            v-for="s in symbols"
            :key="s.symbol"
            :value="s.symbol"
          >
            {{ s.symbol }}
          </option>
        </select>
      </div>
      
      <!-- Candles Info -->
      <div class="candles-info">
        calculée à partir de <span class="candles-value">{{ formatCandlesCount(props.result.global_metrics.total_candles) }}</span> bougies du <span class="date-range">{{ formatDate(props.result.period_start) }}</span> au <span class="date-range">{{ formatDate(props.result.period_end) }}</span>
      </div>
      
      <!-- Date Range Filter Controls (Right Side) -->
      <div v-if="props.dateStart && props.dateEnd" class="date-filter-right">
        <div class="period-display">
          <span class="period-label">Période du calendrier du</span>
          <input
            v-model="localDateStart"
            type="date"
            :min="props.minDateAvailable"
            :max="props.maxDateAvailable"
            class="date-input-inline"
          />
          <span class="period-label">au</span>
          <input
            v-model="localDateEnd"
            type="date"
            :min="props.minDateAvailable"
            :max="props.maxDateAvailable"
            class="date-input-inline"
          />
          <button
            class="btn-recalculate-right"
            @click="handleRecalculate"
          >
            Recalculer
          </button>
        </div>
      </div>
    </div>

    <!-- Sub-components -->
    <MetricsDisplay 
      :global-metrics="props.result.global_metrics" 
      :point-value="props.result.point_value"
      :unit="props.result.unit"
      :symbol="props.result.symbol"
      :recommendation="props.result.recommendation"
      :risk-level="props.result.risk_level"
      :confidence-score="props.result.confidence_score"
    />
  </div>
  <div
    v-else
    class="loading"
  >
    <p>Sélectionnez une paire pour analyser...</p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore, type Stats15Min } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import MetricsDisplay from './analysis/MetricsDisplay.vue'

interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  mean_range: number
  total_candles: number
}

interface HourlyStats {
  hour: number
  candle_count: number
  atr_mean: number
}

interface AnalysisResult {
  symbol: string
  period_start: string
  period_end: string
  timeframe: string
  recommendation: string
  risk_level: string
  confidence_score: number
  global_metrics: GlobalMetrics
  hourly_stats: HourlyStats[]
  best_hours?: number[]
  stats_15min?: Stats15Min[]
  point_value: number
  unit: string
}

const props = defineProps<{
  result: AnalysisResult | null
  symbols: Array<{ symbol: string; file_path: string }>
  dateStart?: string
  dateEnd?: string
  minDateAvailable?: string
  maxDateAvailable?: string
  onRecalculate?: () => Promise<void>
}>()

const emit = defineEmits<{
  (e: 'symbolSelected', symbol: string): void
  (e: 'update:dateStart', value: string): void
  (e: 'update:dateEnd', value: string): void
}>()

const store = useVolatilityStore()
const currentSymbol = computed(() => props.result?.symbol || '')
const symbols = ref<Array<{ symbol: string; file_path: string }>>([])
const localDateStart = ref(props.dateStart || '')
const localDateEnd = ref(props.dateEnd || '')

watch(() => props.dateStart, (newVal) => { if (newVal) localDateStart.value = newVal }, { immediate: true })
watch(() => props.dateEnd, (newVal) => { if (newVal) localDateEnd.value = newVal }, { immediate: true })

watch(localDateStart, (newVal) => { emit('update:dateStart', newVal) })
watch(localDateEnd, (newVal) => { emit('update:dateEnd', newVal) })

async function handleRecalculate() {
  if (props.onRecalculate) {
    await props.onRecalculate()
  }
}
const { onPairDataRefresh } = useDataRefresh()
const unsubscribe = onPairDataRefresh(() => { store.loadSymbols() })

onMounted(async () => {
  try {
    symbols.value = props.symbols || await invoke('load_symbols')
  } catch (err) {}
})

onBeforeUnmount(() => { unsubscribe() })

watch(() => store.symbols, (newSymbols) => {
  symbols.value = newSymbols
}, { deep: true })

function onSymbolChange(newSymbol: string) {
  if (newSymbol && newSymbol !== props.result?.symbol) emit('symbolSelected', newSymbol)
}

function formatCandlesCount(count: number): string {
  return count.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ' ')
}

function formatDate(dateString: string): string {
  if (!dateString || typeof dateString !== 'string') return ''
  
  // Mapping des mois français
  const frenchMonths: { [key: string]: number } = {
    'janvier': 1, 'février': 2, 'mars': 3, 'avril': 4,
    'mai': 5, 'juin': 6, 'juillet': 7, 'août': 8,
    'septembre': 9, 'octobre': 10, 'novembre': 11, 'décembre': 12
  }
  
  // Format français (exemple: "31 décembre 2025")
  const frenchMatch = dateString.match(/^(\d{1,2})\s+(\w+)\s+(\d{4})$/)
  if (frenchMatch) {
    const day = String(frenchMatch[1]).padStart(2, '0')
    const monthName = frenchMatch[2].toLowerCase()
    const month = frenchMonths[monthName]
    if (month) {
      const monthStr = String(month).padStart(2, '0')
      return `${day}/${monthStr}/${frenchMatch[3]}`
    }
  }
  
  // Format ISO 8601 (exemple: "2025-12-31T00:00:00Z")
  if (dateString.includes('T')) {
    const date = new Date(dateString)
    if (!isNaN(date.getTime())) {
      const day = String(date.getUTCDate()).padStart(2, '0')
      const month = String(date.getUTCMonth() + 1).padStart(2, '0')
      const year = date.getUTCFullYear()
      return `${day}/${month}/${year}`
    }
  }
  
  // Format YYYY-MM-DD (exemple: "2025-12-31")
  if (dateString.match(/^\d{4}-\d{2}-\d{2}$/)) {
    const date = new Date(dateString + 'T00:00:00Z')
    if (!isNaN(date.getTime())) {
      const day = String(date.getUTCDate()).padStart(2, '0')
      const month = String(date.getUTCMonth() + 1).padStart(2, '0')
      const year = date.getUTCFullYear()
      return `${day}/${month}/${year}`
    }
  }
  
  // Format DD/MM/YYYY (exemple: "31/12/2025") - déjà au bon format
  if (dateString.match(/^\d{2}\/\d{2}\/\d{4}$/)) {
    return dateString
  }
  
  return ''
}
</script>

<style scoped>
.analysis-panel { background: #161b22; padding: 30px; border-radius: 12px; border: 1px solid #30363d; }
.panel-header { display: flex; align-items: center; justify-content: space-between; gap: 30px; margin-bottom: 30px; flex-wrap: wrap; }
.header-title { display: flex; align-items: center; gap: 12px; }
.header-title h2 { margin: 0; white-space: nowrap; }
.symbol-select { padding: 8px 12px; border: 2px solid #30363d; background: #1a202c; color: #000000; border-radius: 6px; cursor: pointer; font-weight: 600; }
.symbol-select option { background: #ffffff; color: #000000; }

.candles-info { display: flex; align-items: center; gap: 6px; padding: 10px 14px; background: rgba(59, 130, 246, 0.1); border-left: 3px solid #3b82f6; border-radius: 6px; font-size: 0.95em; color: #cbd5e0; white-space: nowrap; flex: 1; min-width: 400px; }
.candles-value { font-size: 1.5em; font-weight: bold; color: #3b82f6; }
.date-range { color: #60a5fa; font-weight: 600; }

/* Date filter controls (right side) */
.date-filter-right { display: flex; align-items: center; }
.period-display { display: flex; align-items: center; gap: 10px; padding: 12px 16px; background: rgba(100, 116, 139, 0.1); border-left: 3px solid #64748b; border-radius: 6px; white-space: nowrap; }
.period-label { color: #cbd5e0; font-size: 0.9em; font-weight: 500; }
.date-input-inline { padding: 6px 10px; border: 2px solid #30363d; background: #1a202c; color: #e2e8f0; border-radius: 4px; font-size: 0.9em; cursor: pointer; font-weight: 600; min-width: 100px; }
.date-input-inline:hover { border-color: #404854; }
.date-input-inline:focus { outline: none; border-color: #3b82f6; }
.btn-recalculate-right { padding: 6px 14px; background: #3b82f6; color: #ffffff; border: none; border-radius: 4px; cursor: pointer; font-weight: 600; font-size: 0.9em; transition: background-color 0.2s ease; margin-left: 8px; white-space: nowrap; }
.btn-recalculate-right:hover { background: #2563eb; }
.btn-recalculate-right:active { background: #1d4ed8; }

.loading { text-align: center; padding: 40px; color: #a0aec0; }
</style>