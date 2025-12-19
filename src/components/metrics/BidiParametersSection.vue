<template>
  <div class="bidi-parameters-section">
    <!-- 1. Metrics Grid (Top) -->
    <MetricsGrid 
      v-if="analysis && analysisData"
      :analysis="analysis" 
      :analysis-data="analysisData" 
    />

    <!-- 2. Cockpit Grid (3 Columns) -->
    <div class="cockpit-grid">
      <!-- Left: Directional -->
      <div class="col-left">
        <StraddleDirectionalCard
          :meilleur-moment="meilleurMoment"
          :offset="offset"
          :stop-loss="stopLoss"
          :trailing-stop="trailingStop"
          :timeout="timeout"
          :pair="symbol || 'EURUSD'"
          :point-value="pointValue"
          :placement-time="placementTime"
        />
      </div>

      <!-- Center: Duration & Info -->
      <BidiVolatilityGraph
        :volatility-profile="volatilityProfile"
        :meilleur-moment="meilleurMoment"
        :duration="volatilityDuration?.peak_duration_minutes"
        :placement-time="placementTime"
        :hour="analysis?.slice?.hour"
        :quarter="analysis?.slice?.quarter"
        :events="recurringEvents"
      />

      <!-- Right: Simultaneous -->
      <div class="col-right">
        <StraddleSimultaneousCard
          :meilleur-moment="meilleurMoment"
          :offset="offset"
          :stop-loss-recovery="stopLossRecovery"
          :trailing-stop="trailingStop"
          :timeout="timeout"
          :pair="symbol || 'EURUSD'"
          :point-value="pointValue"
          :placement-time="placementTime"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MetricsGrid from './MetricsGrid.vue'
import StraddleDirectionalCard from '../trading/StraddleDirectionalCard.vue'
import StraddleSimultaneousCard from '../trading/StraddleSimultaneousCard.vue'
import BidiVolatilityGraph from './BidiVolatilityGraph.vue'
import type { SliceAnalysis } from '../../utils/straddleAnalysis'
import { getPointsPerPip } from '../../utils/pipConverter'

interface EntryWindowAnalysis { optimal_offset: number; optimal_entry_minutes: number }
interface WhipsawAnalysis { whipsaw_frequency_percentage: number; trailing_stop_adjusted: number; optimal_entry_minutes: number }
interface OffsetOptimal { sl_adjusted_points: number; optimal_offset: number }
// Use any for VolatilityDuration to match Record<string, unknown> expected by child component
type VolatilityDuration = any 

const props = defineProps<{
  sliceAnalyses: SliceAnalysis[]
  entryWindowAnalysis: EntryWindowAnalysis
  analysis: SliceAnalysis
  analysisData: any
  volatilityDuration: VolatilityDuration
  tradingPlan?: Record<string, unknown>
  whipsawAnalysis?: WhipsawAnalysis | null
  offsetOptimal?: OffsetOptimal | null
  symbol?: string
  pointValue?: number
  recurringEvents?: Array<{
    time: string
    name: string
    impact: string
    currency: string
    frequency: number
  }>
}>()

const pointsPerPip = computed(() => getPointsPerPip(props.symbol || 'EURUSD'))

// Computed values for the cards
// Priorité à l'analyse fine (entryWindowAnalysis) pour le temps, sinon fallback sur stats
const meilleurMoment = computed(() => {
  if (props.entryWindowAnalysis?.optimal_entry_minutes !== undefined) {
    return props.entryWindowAnalysis.optimal_entry_minutes
  }
  if (props.analysis?.slice?.stats?.optimal_entry_minute !== undefined) {
    return props.analysis.slice.stats.optimal_entry_minute
  }
  return 0
})

// Offset: Priorité aux paramètres Straddle calculés (points), sinon fallback
const offset = computed(() => {
  if (props.analysis?.slice?.stats?.straddle_parameters?.offset_pips) {
    return props.analysis.slice.stats.straddle_parameters.offset_pips
  }
  return props.entryWindowAnalysis?.optimal_offset ?? 0
})

// Conversion Points -> Pips pour affichage correct via UnitDisplay
const stopLoss = computed(() => {
  if (props.analysis?.slice?.stats?.straddle_parameters?.stop_loss_pips) {
    return props.analysis.slice.stats.straddle_parameters.stop_loss_pips
  }
  return (props.offsetOptimal?.sl_adjusted_points ?? 0) / pointsPerPip.value
})

const trailingStop = computed(() => {
  if (props.analysis?.slice?.stats?.straddle_parameters?.trailing_stop_pips) {
    return props.analysis.slice.stats.straddle_parameters.trailing_stop_pips
  }
  return (props.whipsawAnalysis?.trailing_stop_adjusted ?? 0) / pointsPerPip.value
})

const timeout = computed(() => {
  if (props.analysis?.slice?.stats?.straddle_parameters?.timeout_minutes) {
    return props.analysis.slice.stats.straddle_parameters.timeout_minutes
  }
  return Math.round((props.volatilityDuration?.peak_duration_minutes || 21) * 1.5)
})

const stopLossRecovery = computed(() => {
  if (props.analysis?.slice?.stats?.straddle_parameters?.sl_recovery_pips) {
    return props.analysis.slice.stats.straddle_parameters.sl_recovery_pips
  }
  return stopLoss.value * 1.5
})

const volatilityProfile = computed(() => props.analysis?.slice?.stats?.volatility_profile ?? [])

const placementTime = computed(() => {
  if (!props.analysis?.slice) return undefined
  const h = props.analysis.slice.hour
  const q = props.analysis.slice.quarter
  const entryMin = Math.round(meilleurMoment.value)
  
  // q=0 -> 00, q=1 -> 15, q=2 -> 30, q=3 -> 45
  const startMin = q * 15
  
  let totalMin = startMin + entryMin
  let finalH = h
  
  while (totalMin >= 60) {
    totalMin -= 60
    finalH = (finalH + 1) % 24
  }
  
  const mm = totalMin.toString().padStart(2, '0')
  const hh = finalH.toString().padStart(2, '0')
  return `${hh}:${mm}`
})

</script>

<style scoped>
.bidi-parameters-section {
  margin-top: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.cockpit-grid {
  display: grid;
  grid-template-columns: 300px 1fr 300px;
  gap: 8px;
  margin-top: 8px;
  align-items: stretch; /* Stretch to fill height */
  flex: 1;
  min-height: 0;
}

/* Responsive adjustments */
@media (max-width: 1200px) {
  .cockpit-grid {
    grid-template-columns: 1fr;
  }
  
  .col-left, .col-right {
    max-width: 400px;
    margin: 0 auto;
    width: 100%;
  }
}
</style>
