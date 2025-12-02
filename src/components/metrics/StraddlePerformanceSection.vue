<template>
  <div class="straddle-performance-section">
    <h4>📊 Performance Straddle Simulée</h4>
    <div class="performance-grid">
      <!-- Win Rate Ajusté -->
      <div class="performance-metric">
        <div class="metric-label">
          Taux de Gain (ajusté)
        </div>
        <div
          v-if="winRate"
          class="metric-display"
        >
          <span
            class="metric-value"
            :style="{ color: winRateColor }"
          >{{ winRate.win_rate_adjusted.toFixed(1) }}%</span>
          <span class="metric-subtext">(pondéré par whipsaw)</span>
        </div>
        <div
          v-else
          class="metric-loading"
        >
          <span>⏳ Calcul...</span>
        </div>
      </div>

      <!-- Stop Loss Ajusté -->
      <div class="performance-metric">
        <div class="metric-label">
          Stop Loss (ajusté)
        </div>
        <div
          v-if="offsetOptimal"
          class="metric-display"
        >
          <span class="metric-value">{{ offsetOptimal.sl_adjusted_pips.toFixed(1) }} pips</span>
          <span class="metric-subtext">(pondéré par whipsaw)</span>
        </div>
        <div
          v-else
          class="metric-loading"
        >
          <span>⏳ Calcul...</span>
        </div>
      </div>

      <!-- Fréquence Whipsaw -->
      <div class="performance-metric">
        <div class="metric-label">
          Fréquence Whipsaw
        </div>
        <div
          v-if="whipsawAnalysis"
          class="metric-display"
        >
          <span
            class="metric-value"
            :style="{ color: whipsawAnalysis.risk_color }"
          >{{ whipsawAnalysis.whipsaw_frequency_percentage.toFixed(1) }}%</span>
          <span class="metric-subtext">({{ whipsawAnalysis.risk_level }})</span>
        </div>
        <div
          v-else
          class="metric-loading"
        >
          <span>⏳ Calcul...</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface WinRate {
  win_rate_adjusted: number
}

interface WhipsawAnalysis {
  whipsaw_frequency_percentage: number
  risk_color: string
  risk_level: string
}

interface OffsetOptimal {
  sl_adjusted_pips: number
}

defineProps<{
  winRate: WinRate
  whipsawAnalysis: WhipsawAnalysis
  offsetOptimal: OffsetOptimal
  winRateColor: string
}>()
</script>

<style scoped>
.straddle-performance-section {
  background: linear-gradient(135deg, rgba(45, 90, 123, 0.15) 0%, rgba(78, 205, 196, 0.1) 100%);
  border: 1px solid #2d5a7b;
  padding: 20px;
  border-radius: 8px;
  margin-top: 12px;
}

.straddle-performance-section h4 {
  color: #e0e7ff;
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 10px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.performance-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
}

.performance-metric {
  background: rgba(30, 30, 45, 0.6);
  border: 1px solid rgba(168, 85, 247, 0.2);
  border-radius: 6px;
  padding: 12px;
  text-align: center;
}

.metric-label {
  font-size: 11px;
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  margin-bottom: 8px;
  font-weight: 600;
}

.metric-display {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-value {
  font-size: 18px;
  font-weight: bold;
  line-height: 1;
}

.metric-subtext {
  font-size: 10px;
  color: #94a3b8;
}

.metric-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 40px;
  color: #64748b;
  font-size: 12px;
  font-style: italic;
}
</style>
