<template>
  <div class="movement-quality-section">
    <h4>💫 Qualité du Mouvement - Détails</h4>
    
    <!-- Pas de données -->
    <div
      v-if="!getMovementQualityKey(analysis)"
      style="color: #999;"
    >
      ⚠️ Pas de données de qualité
    </div>
    
    <!-- Données chargées -->
    <div
      v-else-if="movementQualities[getMovementQualityKey(analysis)]"
      style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;"
    >
      <div style="font-size: 14px; color: #e0e7ff; font-weight: bold; margin-bottom: 8px;">
        Score: {{ (movementQualities[getMovementQualityKey(analysis)]?.score || 0).toFixed(0) }}/100
      </div>
      <div style="font-size: 13px; color: #b0bec5; margin-bottom: 8px;">
        Label: {{ movementQualities[getMovementQualityKey(analysis)]?.label || 'N/A' }}
      </div>
      <div style="display: flex; gap: 8px;">
        <span :class="['quality-badge', getQualityStatus(movementQualities[getMovementQualityKey(analysis)]?.score || 0)]">
          {{ getQualityStatusText(movementQualities[getMovementQualityKey(analysis)]?.score || 0) }}
        </span>
      </div>
    </div>

    <!-- Chargement en cours -->
    <div
      v-else
      class="quality-loading"
    >
      ⏳ Analyse du mouvement en cours...
    </div>
  </div>
</template>

<script setup lang="ts">
import MetricTooltip from '../MetricTooltip.vue'

const props = defineProps<{
  analysis: any
  analysisData: any
  movementQualities: Record<string, any>
}>()

/**
 * Helper: construit la clé pour accéder une qualité de mouvement
 */
const getMovementQualityKey = (analysis: any): string => {
  if (!analysis?.slice) return ''
  return `${analysis.slice.hour}-${analysis.slice.quarter}`
}

/**
 * Détermine le statut d'appréciation basé sur le score (échelle 0-100)
 */
const getQualityStatus = (score: number): string => {
  if (score > 80) return 'excellent'
  if (score > 60) return 'good'
  if (score > 40) return 'acceptable'
  return 'poor'
}

/**
 * Retourne le texte d'appréciation avec emoji
 */
const getQualityStatusText = (score: number): string => {
  if (score > 80) return '🟢 Excellent'
  if (score > 60) return '🔵 Bon'
  if (score > 40) return '🟡 Acceptable'
  return '🔴 Faible'
}
</script>

<style scoped>
/* Movement Quality Section */
.movement-quality-section {
  padding: 20px;
  background: linear-gradient(135deg, rgba(45, 90, 123, 0.15) 0%, rgba(78, 205, 196, 0.1) 100%);
  border: 1px solid #2d5a7b;
  border-radius: 8px;
}

.movement-quality-section h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #e0e7ff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.quality-loading {
  text-align: center;
  padding: 12px;
  color: #64748b;
  font-size: 12px;
  font-style: italic;
}

/* Quality Badges */
.quality-badge {
  padding: 8px 16px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 14px;
  white-space: nowrap;
}

.quality-badge.excellent {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.quality-badge.good {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

.quality-badge.acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #eab308;
}

.quality-badge.poor {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}
</style>
