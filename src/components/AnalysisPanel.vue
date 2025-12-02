<template>
  <div
    v-if="props.result"
    class="analysis-panel"
  >
    <!-- Panel Header: Title + Symbol + Badges -->
    <div class="panel-header">
      <div class="header-title">
        <h2>🎯 Analyse: {{ props.result.symbol }}</h2>
        <div class="symbol-control">
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
          <div class="candles-info">
            calculée à partir de <span class="candles-value">{{ formatCandlesCount(props.result.global_metrics.total_candles) }}</span> bougies
          </div>
        </div>
      </div>
      <div class="badges">
        <MetricTooltip title="Qualité du Setup Straddle">
          <span 
            :class="['badge', 'recommendation', recommendationClass]"
          >
            {{ formatRecommendation(props.result.recommendation) }}
          </span>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">
              Évalue la qualité des conditions de marché pour exécuter un <strong>Straddle</strong>.
            </div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Codes Couleurs & Signification</div>
            <div class="tooltip-section-text">
              <strong style="color: #10b981;">✅ SETUP OPTIMAL (Vert)</strong><br>Score 80-100 : Conditions idéales. Offset standard <strong>10-15 pips</strong>.<br><br>
              <strong style="color: #3b82f6;">🟢 SETUP CORRECT (Bleu)</strong><br>Score 65-80 : Bon setup. Élargir légèrement l'offset à <strong>15-20 pips</strong>.<br><br>
              <strong style="color: #f59e0b;">🔵 SETUP ACCEPTABLE (Orange)</strong><br>Score 50-65 : Setup moyen. Offset large recommandé <strong>20-30 pips</strong>.<br><br>
              <strong style="color: #f97316;">🟠 SETUP RISQUÉ (Orange foncé)</strong><br>Score 35-50 : Conditions médiocres. Envisager de passer l'événement.<br><br>
              <strong style="color: #ef4444;">❌ NE PAS TRADER (Rouge)</strong><br>Score &lt;35 : Conditions inadaptées.
            </div>
          </template>
        </MetricTooltip>

        <MetricTooltip title="Qualité du Mouvement">
          <span 
            :class="['badge', 'risk', getRiskClass(props.result.risk_level)]"
          >
            {{ formatRisk(props.result.risk_level) }}
          </span>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">
              Caractérise le <strong>type de mouvement</strong> attendu après l'annonce.
            </div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Codes Couleurs & Signification</div>
            <div class="tooltip-section-text">
              <strong style="color: #22c55e;">🟢 DIRECTIONNEL (Vert)</strong><br>Volatilité 15-30% avec faible bruit. Idéal pour Straddle.<br><br>
              <strong style="color: #f59e0b;">🔵 MODÉRÉ (Orange)</strong><br>Volatilité 5-15% avec bruit acceptable. Straddle possible.<br><br>
              <strong style="color: #ef4444;">🔴 ERRATIQUE (Rouge)</strong><br>Soit trop calme (&lt;5%) soit trop chaotique (&gt;30%).
            </div>
          </template>
        </MetricTooltip>
      </div>
    </div>

    <!-- Sub-components -->
    <ConfidenceSection :confidence-score="props.result.confidence_score" />
    <MetricsDisplay 
      :global-metrics="props.result.global_metrics" 
      :estimated-price="getEstimatedPrice()"
    />
    <ColorLegendSection @analyze="openAnalysisModal" />
  </div>
  <div
    v-else
    class="loading"
  >
    <p>Sélectionnez une paire pour analyser...</p>
  </div>

  <!-- Analysis Modal -->
  <MetricsAnalysisModal
    :is-open="isAnalysisModalOpen"
    :analysis-result="props.result"
    @close="isAnalysisModalOpen = false"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore, type Stats15Min } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import MetricTooltip from './MetricTooltip.vue'
import MetricsAnalysisModal from './MetricsAnalysisModal.vue'
import ConfidenceSection from './analysis/ConfidenceSection.vue'
import MetricsDisplay from './analysis/MetricsDisplay.vue'
import ColorLegendSection from './analysis/ColorLegendSection.vue'

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
  best_hours: number[]
  stats_15min?: Stats15Min[]
}

const props = defineProps<{
  result: AnalysisResult | null
  symbols: Array<{ symbol: string; file_path: string }>
}>()

const emit = defineEmits<{
  symbolSelected: [symbol: string]
}>()

const store = useVolatilityStore()
const currentSymbol = computed(() => props.result?.symbol || '')
const symbols = ref<Array<{ symbol: string; file_path: string }>>([])
const isAnalysisModalOpen = ref(false)
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

function formatRecommendation(rec: string): string {
  const map: { [key: string]: string } = {
    'StraddleOptimal': '✅ SETUP OPTIMAL',
    'StraddleGood': '🟢 SETUP CORRECT',
    'StraddleCautious': '🔵 SETUP ACCEPTABLE',
    'StraddleRisky': '🟠 SETUP RISQUÉ',
    'NoTrade': '❌ NE PAS TRADER'
  }
  return map[rec] || rec
}

function formatRisk(risk: string): string {
  const map: { [key: string]: string } = {
    'Low': '🟢 DIRECTIONNEL',
    'Medium': '🔵 MODÉRÉ',
    'High': '🔴 ERRATIQUE'
  }
  return map[risk] || risk
}

function formatCandlesCount(count: number): string {
  return count.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ' ')
}

function getRiskClass(risk: string): string {
  const map: { [key: string]: string } = { 'Low': 'low', 'Medium': 'medium', 'High': 'high' }
  return map[risk] || ''
}

function getEstimatedPrice(): number {
  if (!props.result?.hourly_stats || props.result.hourly_stats.length === 0) {
    return 100000
  }
  const atr = props.result.global_metrics.mean_atr
  if (atr > 1000) return 100000
  if (atr > 10) return 10000
  return 1.0
}

function openAnalysisModal() {
  isAnalysisModalOpen.value = true
}

const recommendationClass = computed(() => {
  const rec = props.result?.recommendation
  if (rec === 'StraddleOptimal') return 'optimal'
  if (rec === 'StraddleGood') return 'good'
  if (rec === 'StraddleCautious') return 'cautious'
  if (rec === 'StraddleRisky') return 'risky'
  if (rec === 'NoTrade') return 'notrade'
  return 'hold'
})
</script>

<style scoped>
.analysis-panel { background: #161b22; padding: 30px; border-radius: 12px; border: 1px solid #30363d; }
.panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; }
.header-title { display: flex; align-items: center; gap: 15px; }
.header-title h2 { margin: 0; }
.symbol-control { display: flex; align-items: center; gap: 20px; }
.symbol-select { padding: 8px 12px; border: 2px solid #30363d; background: #1a202c; color: #000000; border-radius: 6px; cursor: pointer; font-weight: 600; }
.symbol-select option { background: #ffffff; color: #000000; }
.candles-info { display: flex; align-items: center; gap: 6px; padding: 10px 14px; background: rgba(59, 130, 246, 0.1); border-left: 3px solid #3b82f6; border-radius: 6px; font-size: 0.95em; color: #cbd5e0; white-space: nowrap; }
.candles-value { font-size: 1.5em; font-weight: bold; color: #3b82f6; }
.badges { display: flex; gap: 10px; }
.badge { padding: 8px 16px; border-radius: 6px; font-weight: 600; font-size: 0.9em; color: white; cursor: help; transition: all 0.2s; border: 2px solid transparent; }
.badge:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
.recommendation.optimal { background: linear-gradient(135deg, #10b981 0%, #059669 100%); border-color: #047857; }
.recommendation.good { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); border-color: #1d4ed8; }
.recommendation.cautious { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); border-color: #b45309; }
.recommendation.risky { background: linear-gradient(135deg, #f97316 0%, #ea580c 100%); border-color: #c2410c; }
.recommendation.notrade, .badge.risk.high { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); border-color: #b91c1c; }
.loading { text-align: center; padding: 40px; color: #a0aec0; }
</style>