<script setup lang="ts">
import { computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'

const { dynamicAdvice, eventStatistics, globalStats } = useArchiveStatistics()

interface AdviceItem {
  type: 'success' | 'warning' | 'info' | 'strategy'
  emoji: string
  title: string
  description: string
}

const formattedAdvice = computed<AdviceItem[]>(() => {
  const items: AdviceItem[] = []

  if (!dynamicAdvice.value || dynamicAdvice.value.length === 0) {
    return items
  }

  // Parse advice strings
  dynamicAdvice.value.forEach((advice) => {
    if (advice.includes('optimal') || advice.includes('OPTIMAL')) {
      items.push({
        type: 'success',
        emoji: '✅',
        title: 'Configuration Optimale Détectée',
        description: advice,
      })
    } else if (advice.includes('volatilité') || advice.includes('Volatilité')) {
      items.push({
        type: 'info',
        emoji: '📊',
        title: 'Analyse de Volatilité',
        description: advice,
      })
    } else if (advice.includes('cautious') || advice.includes('Risqué')) {
      items.push({
        type: 'warning',
        emoji: '⚠️',
        title: 'Attention Requise',
        description: advice,
      })
    } else {
      items.push({
        type: 'strategy',
        emoji: '🎯',
        title: 'Recommandation Stratégique',
        description: advice,
      })
    }
  })

  return items
})

const optimalEventCount = computed(() => {
  if (!eventStatistics.value) return 0
  return Object.values(eventStatistics.value).filter((stats) => stats.tradabilityScore >= 80).length
})

const riskLevel = computed(() => {
  if (!globalStats.value) return 'Moyen'
  const avgConfidence = globalStats.value.avgConfidence
  if (avgConfidence >= 85) return 'Très Faible'
  if (avgConfidence >= 70) return 'Faible'
  if (avgConfidence >= 50) return 'Moyen'
  return 'Élevé'
})

const winRateEstimate = computed(() => {
  if (!globalStats.value) return 55
  return Math.min(75, Math.max(45, globalStats.value.estimatedWinRate))
})
</script>

<template>
  <div class="advice-block">
    <!-- Header -->
    <div class="header-section">
      <div class="header-content">
        <h3>Recommandations Stratégiques</h3>
        <p class="header-subtitle">
          {{ optimalEventCount }} événements OPTIMAL • Risque: {{ riskLevel }} • Win Rate estimé: {{ winRateEstimate }}%
        </p>
      </div>
      <div class="header-icon">🎯</div>
    </div>

    <!-- Risk & Win Rate Summary -->
    <div class="summary-grid">
      <div class="summary-card">
        <div class="summary-label">Niveau de Risque</div>
        <div class="summary-value" :data-risk="riskLevel.toLowerCase()">
          <span v-if="riskLevel === 'Très Faible'">🟢</span>
          <span v-else-if="riskLevel === 'Faible'">🟢</span>
          <span v-else-if="riskLevel === 'Moyen'">🟡</span>
          <span v-else>🔴</span>
          {{ riskLevel }}
        </div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Win Rate Estimé</div>
        <div class="summary-value">📈 {{ winRateEstimate }}%</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Configuration</div>
        <div class="summary-value">{{ optimalEventCount > 3 ? '⭐ Excellente' : '🔶 Bonne' }}</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Événements Optimaux</div>
        <div class="summary-value">{{ optimalEventCount }}</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Archives Chargées</div>
        <div class="summary-value">{{ globalStats?.totalArchives || 0 }}/25</div>
      </div>
    </div>

    <!-- Advice Items (Horizontal) -->
    <div v-if="formattedAdvice.length > 0" class="advice-items-row">
      <div
        v-for="(item, idx) in formattedAdvice"
        :key="idx"
        class="advice-card-compact"
        :data-type="item.type"
      >
        <span class="advice-emoji-compact">{{ item.emoji }}</span>
        <div class="advice-text-compact">
          <div class="advice-title-compact">{{ item.title }}</div>
          <div class="advice-description-compact">{{ item.description }}</div>
        </div>
      </div>
    </div>

    <!-- General Recommendations (Fallback) -->
    <div v-if="formattedAdvice.length === 0" class="advice-card-compact" data-type="info">
      <span class="advice-emoji-compact">💡</span>
      <div class="advice-text-compact">
        <div class="advice-title-compact">Données d'Archives Chargées</div>
        <div class="advice-description-compact">
          {{ globalStats?.totalArchives || 0 }} archives détectées. Analysez les événements pour obtenir des recommandations personnalisées.
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.advice-block {
  animation: slideIn 0.3s ease-out 0.3s both;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  border-bottom: 1px solid rgba(168, 85, 247, 0.3);
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

.summary-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
  margin-bottom: 12px;
}

.summary-card {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 10px;
}

.summary-label {
  font-size: 10px;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  margin-bottom: 6px;
}

.summary-value {
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
}

.summary-value[data-risk="très faible"],
.summary-value[data-risk="faible"] {
  color: #10b981;
}

.summary-value[data-risk="moyen"] {
  color: #fb923c;
}

.summary-value[data-risk="élevé"] {
  color: #ef4444;
}

.advice-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.advice-card[data-type="success"] {
  background: rgba(16, 185, 129, 0.15);
  border-left-color: #10b981;
}

.advice-card[data-type="info"] {
  background: rgba(59, 130, 246, 0.15);
  border-left-color: #3b82f6;
}

.advice-card[data-type="warning"] {
  background: rgba(239, 68, 68, 0.15);
  border-left-color: #ef4444;
}

.advice-card[data-type="strategy"] {
  background: rgba(168, 85, 247, 0.15);
  border-left-color: #a855f7;
}

.advice-content {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.advice-emoji {
  font-size: 20px;
  margin-top: 2px;
  flex-shrink: 0;
}

.advice-emoji-compact {
  font-size: 18px;
  flex-shrink: 0;
}

.advice-items-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.advice-card-compact {
  border-radius: 8px;
  padding: 12px;
  border-left: 4px solid;
  display: flex;
  gap: 10px;
  align-items: flex-start;
  background: rgba(0, 0, 0, 0.3);
}

.advice-card-compact[data-type="success"] {
  background: rgba(16, 185, 129, 0.15);
  border-left-color: #10b981;
}

.advice-card-compact[data-type="info"] {
  background: rgba(59, 130, 246, 0.15);
  border-left-color: #3b82f6;
}

.advice-card-compact[data-type="warning"] {
  background: rgba(239, 68, 68, 0.15);
  border-left-color: #ef4444;
}

.advice-card-compact[data-type="strategy"] {
  background: rgba(168, 85, 247, 0.15);
  border-left-color: #a855f7;
}

.advice-text-compact {
  flex: 1;
}

.advice-title-compact {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
}

.advice-description-compact {
  margin: 4px 0 0 0;
  font-size: 11px;
  color: #d1d5db;
  line-height: 1.4;
}

.advice-text {
  flex: 1;
}

.advice-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
}

.advice-description {
  margin: 4px 0 0 0;
  font-size: 12px;
  color: #d1d5db;
  line-height: 1.5;
}

.action-section {
  display: flex;
  justify-content: center;
}

.action-button {
  background: linear-gradient(135deg, #a855f7, #7c3aed);
  border: none;
  color: white;
  padding: 10px 24px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-button:hover {
  background: linear-gradient(135deg, #9333ea, #6d28d9);
  transform: translateY(-2px);
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
