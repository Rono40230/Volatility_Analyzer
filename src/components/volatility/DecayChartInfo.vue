<template>
  <div>
    <div class="chart-legend">
      <div class="legend-item">
        <span class="legend-color profit" />
        <span>Zone de Profit (0-{{ recommendedDuration }}m)</span>
      </div>
      <div class="legend-item">
        <span class="legend-color caution" />
        <span>Zone de Prudence ({{ recommendedDuration }}-{{ halfLifeMinutes }}m)</span>
      </div>
      <div class="legend-item">
        <span class="legend-color risk" />
        <span>Zone de Risque (après {{ halfLifeMinutes }}m)</span>
      </div>
    </div>
    <div class="chart-info">
      <div class="info-item">
        <strong>Pic de volatilité:</strong> {{ (peakVolatility * 100).toFixed(2) }}%
      </div>
      <div class="info-item">
        <strong>Demi-vie:</strong> {{ halfLifeMinutes }} minutes (50% du pic)
      </div>
      <div class="info-item">
        <strong>Durée recommandée:</strong> {{ recommendedDuration }} minutes ({{ (getVolatilityAt(recommendedDuration) * 100).toFixed(1) }}% du pic)
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  peakVolatility: number
  halfLifeMinutes: number
  recommendedDuration: number
}

const props = defineProps<Props>()

const lambda = Math.log(2) / props.halfLifeMinutes
const getVolatilityAt = (t: number) => Math.exp(-lambda * t)
</script>

<style scoped>
.chart-legend {
  display: flex;
  gap: 20px;
  justify-content: center;
  margin: 20px 0;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  color: #cbd5e0;
}

.legend-color {
  width: 20px;
  height: 20px;
  border-radius: 3px;
  border: 1px solid #667eea;
}

.legend-color.profit {
  background: rgba(16, 185, 129, 0.3);
  border-color: #10b981;
}

.legend-color.caution {
  background: rgba(245, 158, 11, 0.3);
  border-color: #f59e0b;
}

.legend-color.risk {
  background: rgba(239, 68, 68, 0.3);
  border-color: #ef4444;
}

.chart-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid #667eea;
  border-radius: 6px;
  padding: 15px;
}

.info-item {
  color: #e2e8f0;
  font-size: 0.9em;
  line-height: 1.6;
}

.info-item strong {
  color: #60a5fa;
  font-weight: 600;
}
</style>
