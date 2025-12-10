<template>
  <div class="graph-header-metrics">
    <div class="stat-group">
      <div class="stat-item">
        <span class="stat-label">Noise Avant</span>
        <span class="stat-value" :class="noiseQualityBefore">{{ noiseRatioBefore.toFixed(2) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Noise Pendant</span>
        <span class="stat-value" :class="noiseQualityDuring">{{ noiseRatioDuring.toFixed(2) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Noise Après</span>
        <span class="stat-value" :class="noiseQualityAfter">{{ noiseRatioAfter.toFixed(2) }}</span>
      </div>
    </div>

    <div class="stat-divider"></div>

    <div class="stat-group">
      <div class="stat-item impact-item">
        <span class="stat-label">Impact Volatilité</span>
        <span class="stat-value impact-value">+{{ volatilityIncreasePercent.toFixed(1) }}%</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Occurrences</span>
        <span class="stat-value">{{ eventCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  noiseRatioBefore: number
  noiseRatioDuring: number
  noiseRatioAfter: number
  volatilityIncreasePercent: number
  eventCount: number
}>()

const noiseQualityBefore = computed(() => props.noiseRatioBefore < 1.5 ? 'clean' : props.noiseRatioBefore < 2.5 ? 'mixed' : 'choppy')
const noiseQualityDuring = computed(() => props.noiseRatioDuring < 1.5 ? 'clean' : props.noiseRatioDuring < 2.5 ? 'mixed' : 'choppy')
const noiseQualityAfter = computed(() => props.noiseRatioAfter < 1.5 ? 'clean' : props.noiseRatioAfter < 2.5 ? 'mixed' : 'choppy')
</script>

<style scoped>
.graph-header-metrics {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 0 10px;
}

.stat-group {
  display: flex;
  gap: 20px;
  align-items: center;
}

.stat-divider {
  width: 1px;
  height: 30px;
  background: #30363d;
  margin: 0 20px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
}

.stat-label {
  font-size: 0.7em;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}

.stat-value {
  font-size: 1.1em;
  font-weight: 700;
  color: #e6edf3;
}

.stat-value.clean { color: #3fb950; }
.stat-value.mixed { color: #d29922; }
.stat-value.choppy { color: #f85149; }
.impact-value { color: #a371f7; }
</style>
