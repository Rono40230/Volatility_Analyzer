<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useBacktestStore } from '../stores/backtest'
import { computed, ref } from 'vue'
import ArchiveModal from './ArchiveModal.vue'
import BacktestAnalysisModal from './BacktestAnalysisModal.vue'
import BacktestSummary from './backtest/BacktestSummary.vue'
import BacktestTradesTable from './backtest/BacktestTradesTable.vue'
import type { BacktestResult, BacktestConfig } from '../stores/backtest'

const props = defineProps<{
  archivedData?: {
    result: BacktestResult,
    config: BacktestConfig
  }
}>()

const store = useBacktestStore()
const { result: storeResult, loading, error, config: storeConfig } = storeToRefs(store)

const result = computed(() => props.archivedData?.result || storeResult.value)
const config = computed(() => props.archivedData?.config || storeConfig.value)
const mode = computed(() => 'Simultane')

// --- Archive Logic ---
const showArchiveModal = ref(false)
const archiveData = ref({
  type: 'Backtest',
  periodStart: '',
  periodEnd: '',
  symbol: '',
  eventName: '',
  dataJson: '',
  defaultTitle: ''
})

function openArchiveModal() {
  if (!result.value) return
  
  // Calculate period from trades
  let start = new Date().toISOString()
  let end = new Date().toISOString()
  
  if (result.value.trades.length > 0) {
    const dates = result.value.trades.map(t => new Date(t.event_date).getTime())
    start = new Date(Math.min(...dates)).toISOString()
    end = new Date(Math.max(...dates)).toISOString()
  }

  // Format: Paire-Evenement-mode de stratégie-TP/R/ATR/Timeout/Spread
  const pair = result.value.symbol
  const event = result.value.event_name
  const strategyMode = mode.value
  const tpR = config.value?.tp_rr ?? 0
  const sl = config.value?.stop_loss_pips ?? 0
  const atrPeriod = config.value?.atr_period ?? 0
  const tsCoef = config.value?.trailing_atr_coef ?? 0
  const timeout = config.value?.timeout_minutes ?? 0
  const spread = config.value?.spread_pips ?? 0

  const defaultTitle = `${pair}-${event}-${strategyMode}-TP${tpR}/R${sl}/ATR${atrPeriod}x${tsCoef}/${timeout}/${spread}`

  archiveData.value = {
    type: 'Backtest',
    periodStart: start,
    periodEnd: end,
    symbol: result.value.symbol,
    eventName: result.value.event_name,
    dataJson: JSON.stringify({
      result: result.value,
      config: config.value,
      mode: mode.value
    }),
    defaultTitle
  }
  
  showArchiveModal.value = true
}

// --- Analysis Logic ---
const showAnalysisModal = ref(false)

function applyRecommendedParams(params: { sl: number; tpRR: number; trailing: number; atrPeriod: number; timeout: number }) {
  storeConfig.value.stop_loss_pips = params.sl
  storeConfig.value.tp_rr = params.tpRR
  storeConfig.value.trailing_atr_coef = params.trailing
  storeConfig.value.atr_period = params.atrPeriod
  storeConfig.value.timeout_minutes = params.timeout
}
</script>

<template>
  <div class="results-panel">
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>Simulation tick-par-tick en cours...</p>
    </div>

    <div v-else-if="error" class="error-state">
      <h3>Erreur</h3>
      <p>{{ error }}</p>
    </div>

    <div v-else-if="result" class="results-content">
      <div class="results-header">
        <h2>Résultats</h2>
        <div class="actions">
          <button class="btn-icon" @click="showAnalysisModal = true" title="Analyser">🧠 Analyse</button>
          <button class="btn-icon" @click="openArchiveModal" title="Archiver">💾 Archiver</button>
        </div>
      </div>
      <div class="results-meta">
        <span>Paire: {{ result.symbol }}</span>
        <span class="meta-sep">|</span>
        <span>Evenement: {{ result.event_name }}</span>
        <span class="meta-sep">|</span>
        <span>Spread: {{ config?.spread_pips ?? 0 }} pips</span>
        <span class="meta-sep">|</span>
        <span>Slippage: {{ config?.slippage_pips ?? 0 }} pips</span>
      </div>

      <BacktestSummary :result="result" />
      <BacktestTradesTable :trades="result.trades" :unit="result.unit" :symbol="result.symbol" />
    </div>

    <div v-else class="empty-state">
      <p>Configurez et lancez un backtest pour voir les résultats.</p>
    </div>

    <ArchiveModal 
      v-if="showArchiveModal"
      :show="showArchiveModal"
      :archive-type="archiveData.type"
      :period-start="archiveData.periodStart"
      :period-end="archiveData.periodEnd"
      :symbol="archiveData.symbol"
      :event-name="archiveData.eventName"
      :data-json="archiveData.dataJson"
      :default-title="archiveData.defaultTitle"
      @close="showArchiveModal = false"
    />

    <BacktestAnalysisModal
      v-if="result && config"
      :is-open="showAnalysisModal"
      :result="result"
      :config="config"
      @close="showAnalysisModal = false"
      @apply-params="applyRecommendedParams"
    />
  </div>
</template>

<style scoped>
.results-panel {
  background: #1a202c;
  border-radius: 8px;
  border: 1px solid #2d3748;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.results-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  overflow: hidden;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.results-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #a0aec0;
  font-size: 0.9rem;
  margin-bottom: 1.5rem;
  flex-wrap: wrap;
}

.meta-sep {
  color: #4a5568;
}

.results-header h2 {
  margin: 0;
  color: #e2e8f0;
}

.actions {
  display: flex;
  gap: 0.5rem;
}

.btn-icon {
  background: #2d3748;
  border: 1px solid #4a5568;
  color: #e2e8f0;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-icon:hover {
  background: #4a5568;
}

.loading-state, .error-state, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #a0aec0;
  padding: 2rem;
}

.error-state {
  color: #f56565;
}

.spinner {
  border: 4px solid rgba(255, 255, 255, 0.1);
  border-left-color: #4299e1;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
