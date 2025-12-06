<template>
  <div class="bidi-parameters-section">
    <h4>⚙️ PARAMÈTRES BIDI OPTIMISÉS</h4>
    <div class="metrics-grid">
      <div class="metric-card">
        <div class="metric-label">
          Meilleur Moment
        </div>
        <div class="metric-value" style="color: #fff;">
          {{ getBestTimeDisplay() }}
        </div>
      </div>
      <MetricTooltip
        title="Winrate"
        direction="top"
      >
        <div class="metric-card">
          <div class="metric-label">
            Winrate
          </div>
          <div class="metric-value" :style="{ color: getWinrateColor(props.winRate?.win_rate_adjusted || 0) }">
            {{ props.winRate?.win_rate_adjusted?.toFixed(1) ?? 'N/A' }}% <span class="unit">(pondéré)</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Taux de gain réaliste ajusté pour tenir compte de la fréquence whipsaw. Formule: Win Rate × (1 - whipsaw_frequency)</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Interprétation</div>
            <div class="interpretation-item"><strong>🟢 Excellent:</strong> ≥50% → Très fiable</div>
            <div class="interpretation-item"><strong>🔵 Bon:</strong> 40-49% → Profitable</div>
            <div class="interpretation-item"><strong>🟡 Acceptable:</strong> 30-39% → Margin serré</div>
            <div class="interpretation-item"><strong>🔴 Faible:</strong> &lt;30% → Risqué</div>
          </div>
        </template>
      </MetricTooltip>
      <MetricTooltip
        title="Stop Loss"
        direction="top"
      >
        <div class="metric-card">
          <div class="metric-label">
            Stop Loss
          </div>
          <div class="metric-value" style="color: #fff;">
            {{ props.offsetOptimal?.sl_adjusted_pips?.toFixed(1) ?? 'N/A' }} <span class="unit">pips</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Distance d'arrêt des pertes, ajustée pour compenser l'impact du whipsaw. Formule: SL × (1 + whipsaw_frequency × 0.3)</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Explication</div>
            <div class="tooltip-section-text">{{ props.offsetOptimal ? `Valeur pondérée pour ${props.whipsawAnalysis?.whipsaw_frequency_percentage?.toFixed(1) || 'N/A'}% de whipsaw` : 'En cours de calcul' }}. Plus le whipsaw est élevé, plus le SL doit être augmenté.</div>
          </div>
        </template>
      </MetricTooltip>
      <MetricTooltip
        title="Trailing Stop"
        direction="top"
      >
        <div class="metric-card">
          <div class="metric-label">
            Trailing Stop
          </div>
          <div class="metric-value" style="color: #fff;">
            {{ props.whipsawAnalysis?.trailing_stop_adjusted?.toFixed(2) ?? props.analysis.tradingPlan.trailingStopCoefficient.toFixed(2) }}x <span class="unit">SL</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Multiplicateur du SL pour stop dynamique, ajusté selon la fréquence whipsaw pour compenser la volatilité.</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Explication</div>
            <div class="tooltip-section-text">{{ props.whipsawAnalysis ? `Valeur pondérée par whipsaw (${props.whipsawAnalysis.whipsaw_frequency_percentage.toFixed(1)}%)` : 'Calculé en fonction de la volatilité du quarter' }}. Formule: 1.59 × (1 - whipsaw / 2)</div>
          </div>
        </template>
      </MetricTooltip>
      <MetricTooltip
        title="Timeout"
        direction="top"
      >
        <div class="metric-card">
          <div class="metric-label">
            Timeout
          </div>
          <div class="metric-value" style="color: #fff;">
            {{ Math.round((props.volatilityDuration?.peak_duration_minutes || 21) * 1.5) }} <span class="unit">min</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Durée maximale du trade, basée sur la décroissance de volatilité du créneau (non pondérée par whipsaw).</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Explication</div>
            <div class="tooltip-section-text">Valeur INITIALE calculée sur la durée de volatilité du quarter. Formule: peak_duration × 1.5. Cette valeur reste STABLE et n'est pas affectée par le whipsaw.</div>
          </div>
        </template>
      </MetricTooltip>
    </div>
  </div>
</template>

<script setup lang="ts">
import MetricTooltip from '../MetricTooltip.vue'
import { useMetricsFormatting } from '../../composables/useMetricsFormatting'
import { getWinrateColor } from './BidiParametersSection.helpers'

interface SliceAnalysis {
  slice: {
    startTime: string
    hour: number
    quarter: number
  }
}

interface EntryWindowAnalysis {
  optimal_offset: number
}

interface WhipsawAnalysis {
  whipsaw_frequency_percentage: number
  trailing_stop_adjusted: number
  optimal_entry_minutes: number
}

interface OffsetOptimal {
  sl_adjusted_pips: number
}

interface WinRate {
  win_rate_adjusted: number
}

interface VolatilityDuration {
  peak_duration_minutes: number
}

interface TradingPlan {
  trailingStopCoefficient: number
}

interface Analysis {
  tradingPlan: TradingPlan
}

const props = defineProps<{
  sliceAnalyses: SliceAnalysis[]
  entryWindowAnalysis: EntryWindowAnalysis
  analysis: Analysis
  volatilityDuration: VolatilityDuration
  whipsawAnalysis?: WhipsawAnalysis
  offsetOptimal?: OffsetOptimal
  winRate?: WinRate
}>()

const { calculateExactTime } = useMetricsFormatting()

const getBestTimeDisplay = () => {
  if (props.sliceAnalyses && props.sliceAnalyses.length > 0) {
    const bestSlice = props.sliceAnalyses[0]
    // Utiliser UNIQUEMENT le meilleur moment d'entrée INITIAL (non pondéré par whipsaw)
    const offset = props.entryWindowAnalysis?.optimal_offset ?? 0
    return calculateExactTime(bestSlice.slice.startTime, offset)
  }
  return props.entryWindowAnalysis?.optimal_offset + ' min'
}
</script>

<style scoped>
.bidi-parameters-section {
  margin-top: 20px;
  padding: 20px;
  background: linear-gradient(135deg, rgba(45, 90, 123, 0.15) 0%, rgba(78, 205, 196, 0.1) 100%);
  border: 1px solid #2d5a7b;
  border-radius: 8px;
}

.bidi-parameters-section h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #e0e7ff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
  margin-top: 15px;
}

.metric-card {
  padding: 12px;
  background: rgba(255,255,255,0.05);
  border-radius: 6px;
}

.metric-label {
  font-size: 11px;
  color: #999;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.metric-value {
  font-size: 13px;
  color: #4ecdc4;
  font-weight: bold;
}

.unit {
  color: #888;
  font-size: 11px;
}

.interpretation-item { margin: 8px 0; font-size: 12px; line-height: 1.4; color: #cbd5e1; padding: 6px; border-left: 2px solid #4ecdc4; }
.interpretation-item strong { color: #e2e8f0; }
.tooltip-section { margin-bottom: 12px; }
.tooltip-section:last-child { margin-bottom: 0; }
.tooltip-section-title { font-weight: 600; color: #58a6ff; margin-bottom: 8px; font-size: 0.9em; text-transform: uppercase; letter-spacing: 0.5px; }
.tooltip-section-text { color: #cbd5e0; font-size: 0.9em; line-height: 1.6; }
</style>
