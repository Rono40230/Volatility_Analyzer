<script setup lang="ts">
import { computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'

const { pairStatistics } = useArchiveStatistics()

interface PairDisplay {
  pair: string
  stats: ReturnType<typeof useArchiveStatistics>['pairStatistics']['value'][string]
  performanceColor: string
  performanceIcon: string
}

const sortedPairs = computed<PairDisplay[]>(() => {
  if (!pairStatistics.value) return []

  return Object.entries(pairStatistics.value)
    .map(([pair, stats]) => {
      let performanceColor = 'text-red-500'
      let performanceIcon = '🔴'

      if (stats.performanceRating === '🟢 TRÈS BON') {
        performanceColor = 'text-green-500'
        performanceIcon = '🟢'
      } else if (stats.performanceRating === '🟡 BON') {
        performanceColor = 'text-yellow-500'
        performanceIcon = '🟡'
      } else if (stats.performanceRating === '🟠 MOYEN') {
        performanceColor = 'text-orange-500'
        performanceIcon = '🟠'
      }

      return {
        pair,
        stats,
        performanceColor,
        performanceIcon,
      }
    })
    .sort((a, b) => b.stats.avgConfidence - a.stats.avgConfidence)
})

const totalPairs = computed(() => sortedPairs.value.length)
const strongPairs = computed(() => sortedPairs.value.filter(p => p.stats.performanceRating.includes('TRÈS BON')).length)
const avgATR = computed(() => {
  if (sortedPairs.value.length === 0) return 0
  const sum = sortedPairs.value.reduce((acc, p) => acc + p.stats.avgATR, 0)
  return Math.round((sum / sortedPairs.value.length) * 10) / 10
})

function getTopSensitiveEvent(eventSensitivity: Record<string, number>): { event: string; sensitivity: number } | null {
  const entries = Object.entries(eventSensitivity)
  if (entries.length === 0) return null
  return entries.reduce((best, current) => (current[1] > best[1] ? { event: current[0], sensitivity: current[1] } : best), {
    event: entries[0][0],
    sensitivity: entries[0][1],
  })
}
</script>

<template>
  <div class="pair-analysis-block">
    <!-- Header -->
    <div class="header-section">
      <div class="header-content">
      </div>
    </div>

    <!-- Pairs Grid -->
    <div v-if="sortedPairs.length > 0" class="pairs-grid">
      <div v-for="pairItem in sortedPairs" :key="pairItem.pair" class="pair-card">
        <!-- Pair Header -->
        <div class="pair-header">
          <h4 class="pair-name">{{ pairItem.pair }}</h4>
          <span class="performance-icon">{{ pairItem.performanceIcon }}</span>
        </div>

        <!-- Metrics -->
        <div class="metrics-list">
          <div class="metric-row">
            <span class="metric-label">Confiance moyenne</span>
            <span class="metric-value">{{ Math.round(pairItem.stats.avgConfidence) }}%</span>
          </div>
          <div class="metric-row">
            <span class="metric-label">Volatilité ATR</span>
            <span class="metric-value">{{ Math.round(pairItem.stats.avgATR * 10) / 10 }}p</span>
          </div>
          <div class="metric-row">
            <span class="metric-label">Performance</span>
            <span class="metric-value">{{ pairItem.stats.performanceRating }}</span>
          </div>
        </div>

        <!-- Top Sensitive Event -->
        <div v-if="getTopSensitiveEvent(pairItem.stats.eventSensitivity)" class="sensitive-event">
          <div class="sensitive-label">Événement le plus sensible</div>
          <div class="sensitive-content">
            <span class="sensitive-event-name">{{ getTopSensitiveEvent(pairItem.stats.eventSensitivity)?.event }}</span>
            <span class="sensitive-value">+{{ Math.round((getTopSensitiveEvent(pairItem.stats.eventSensitivity)?.sensitivity ?? 0) * 100) }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <p>Aucune paire analysée</p>
    </div>
  </div>
</template>

<style scoped>
.pair-analysis-block {
  animation: slideIn 0.3s ease-out 0.1s both;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  border-bottom: 1px solid rgba(16, 185, 129, 0.3);
  padding-bottom: 16px;
  margin-bottom: 20px;
}

.header-content h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
}

.header-subtitle {
  margin: 6px 0 0 0;
  font-size: 12px;
  color: #a0aec0;
}

.header-icon {
  font-size: 32px;
  opacity: 0.7;
}

.pairs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.pair-card {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  padding: 14px;
  transition: all 0.3s ease;
}

.pair-card:hover {
  background: rgba(0, 0, 0, 0.5);
  border-color: rgba(255, 255, 255, 0.1);
}

.pair-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.pair-name {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.performance-icon {
  font-size: 20px;
}

.metrics-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  padding: 6px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.metric-row:last-of-type {
  border-bottom: none;
}

.metric-label {
  color: #a0aec0;
}

.metric-value {
  font-weight: 600;
  color: #ffffff;
}

.sensitive-event {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 6px;
  padding: 8px;
}

.sensitive-label {
  font-size: 10px;
  color: #a0aec0;
  margin-bottom: 4px;
}

.sensitive-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sensitive-event-name {
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
}

.sensitive-value {
  font-size: 12px;
  color: #60a5fa;
  font-weight: 600;
}

.empty-state {
  border: 1px dashed rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  color: #a0aec0;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
