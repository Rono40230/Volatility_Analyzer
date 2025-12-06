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
@import './AdviceBlock.css';
</style>