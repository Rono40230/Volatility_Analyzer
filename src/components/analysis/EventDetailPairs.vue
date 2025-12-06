<script setup lang="ts">
interface Pair {
  pair: string
  impact: number
  confidence: number
}

interface Props {
  pairs: Pair[]
}

defineProps<Props>()

function getImpactColor(impact: number): string {
  if (impact >= 80) return '#10b981'
  if (impact >= 60) return '#fbbf24'
  if (impact >= 40) return '#f97316'
  return '#ef4444'
}

function getImpactIcon(impact: number): string {
  if (impact >= 80) return '🟢'
  if (impact >= 60) return '🟡'
  if (impact >= 40) return '🟠'
  return '🔴'
}
</script>

<template>
  <div class="pairs-section">
    <h3 class="section-title">Corrélation par Paire</h3>
    <div class="pairs-grid">
      <div v-for="pair in pairs" :key="pair.pair" class="pair-card" :style="{ borderLeftColor: getImpactColor(pair.impact) }">
        <div class="pair-header">
          <span class="pair-icon">{{ getImpactIcon(pair.impact) }}</span>
          <span class="pair-name">{{ pair.pair }}</span>
        </div>
        <div class="pair-stats">
          <div class="pair-stat">
            <span class="label">Impact Heatmap</span>
            <span class="value">{{ Math.round(pair.impact) }}%</span>
          </div>
          <div class="pair-stat">
            <span class="label">Confiance</span>
            <span class="value">{{ Math.round(pair.confidence) }}%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pairs-section {
  padding: 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #4ecdc4;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.pairs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.pair-card {
  background: rgba(0, 0, 0, 0.3);
  border-left: 4px solid #3b82f6;
  border-radius: 8px;
  padding: 12px;
}

.pair-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.pair-icon {
  font-size: 16px;
}

.pair-name {
  font-weight: 600;
  color: #ffffff;
  font-size: 14px;
}

.pair-stats {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pair-stat {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.pair-stat .label {
  color: #a0aec0;
}

.pair-stat .value {
  color: #ffffff;
  font-weight: 600;
}
</style>
