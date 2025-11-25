<template>
  <div class="volatility-decay-chart">
    <h3 class="chart-title">📉 Décroissance de la Volatilité</h3>
    <div class="chart-container">
      <DecayChartSVG
        :peak-volatility="peakVolatility"
        :half-life-minutes="halfLifeMinutes"
        :recommended-duration="recommendedDuration"
        :start-hour="startHour"
        :start-minute="startMinute"
      />
    </div>
    <DecayChartLegend
      :peak-volatility="peakVolatility"
      :half-life-minutes="halfLifeMinutes"
      :recommended-duration="recommendedDuration"
    />
  </div>
</template>

<script setup lang="ts">
import DecayChartSVG from './volatility/DecayChartSVG.vue'
import DecayChartLegend from './volatility/DecayChartLegend.vue'

interface Props {
  peakVolatility: number
  halfLifeMinutes: number
  recommendedDuration: number
  maxTime?: number
  startHour?: number
  startMinute?: number
}

withDefaults(defineProps<Props>(), {
  maxTime: 240,
  startHour: 13,
  startMinute: 0
})
</script>

<style scoped>
.volatility-decay-chart {
  background: linear-gradient(135deg, #0f172a 0%, #1a202c 100%);
  border: 1px solid #667eea;
  border-radius: 8px;
  padding: 20px;
  margin: 20px 0;
}

.chart-title {
  color: #e2e8f0;
  font-size: 1.2em;
  font-weight: 600;
  margin-bottom: 15px;
  text-align: center;
}

.chart-container {
  display: flex;
  justify-content: center;
  margin-bottom: 20px;
  overflow-x: auto;
}

@media (max-width: 768px) {
  .volatility-decay-chart {
    padding: 15px;
  }

  .chart-container {
    min-width: 100%;
    overflow-x: auto;
  }
}
</style>
