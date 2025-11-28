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
          <div class="metric-value" :style="{ color: getWinrateColor(entryWindowAnalysis.optimal_win_rate * 100) }">
            {{ (entryWindowAnalysis.optimal_win_rate * 100).toFixed(0) }}% <span class="unit">événement</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Pourcentage de fois où le créneau horaire élu a produit un mouvement gagnant, calculé sur l'historique complet du quarter.</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Interprétation</div>
            <div class="interpretation-item"><strong>🟢 Excellent:</strong> ≥60% → Très fiable</div>
            <div class="interpretation-item"><strong>🔵 Bon:</strong> 55-59% → Profitable</div>
            <div class="interpretation-item"><strong>🟡 Acceptable:</strong> 50-54% → Margin serré</div>
            <div class="interpretation-item"><strong>🔴 Faible:</strong> &lt;50% → Risqué</div>
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
            {{ analysis.tradingPlan.trailingStopCoefficient.toFixed(2) }}x <span class="unit">SL</span>
          </div>
        </div>
        <template #definition>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📖 Définition</div>
            <div class="tooltip-section-text">Multiplicateur du SL pour stop dynamique qui monte avec le prix, protégeant les gains.</div>
          </div>
        </template>
        <template #interpretation>
          <div class="tooltip-section">
            <div class="tooltip-section-title">📊 Explication</div>
            <div class="tooltip-section-text">Calculé en fonction de la volatilité du quarter. 0.9x = 90% du SL, 1.2x = 120% du SL (plus de risque/profit).</div>
          </div>
        </template>
      </MetricTooltip>
      <div class="metric-card">
        <div class="metric-label">
          Timeout Recommandé
        </div>
        <div class="metric-value" style="color: #fff;">
          {{ Math.round((volatilityDuration?.peak_duration_minutes || 21) * 1.5) }} <span class="unit">min</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import MetricTooltip from '../MetricTooltip.vue'
import { useMetricsFormatting } from '../../composables/useMetricsFormatting'
import { getWinrateColor } from './BidiParametersSection.helpers'

const props = defineProps<{
  sliceAnalyses: any[]
  entryWindowAnalysis: any
  analysis: any
  volatilityDuration: any
}>()

const { calculateExactTime } = useMetricsFormatting()

const getBestTimeDisplay = () => {
  if (props.sliceAnalyses && props.sliceAnalyses.length > 0) {
    const bestSlice = props.sliceAnalyses[0]
    return calculateExactTime(bestSlice.slice.startTime, props.entryWindowAnalysis.optimal_offset)
  }
  return props.entryWindowAnalysis.optimal_offset + ' min'
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
