<template>
  <div
    class="slice-card"
    :class="getRankClass(analysis.rank)"
  >
    <!-- Rang + Heure + Recommandation -->
    <div
      class="slice-header"
      style="display: flex; justify-content: space-between; align-items: flex-start; gap: 20px;"
    >
      <div style="display: flex; gap: 12px; align-items: flex-start;">
        <div
          class="rank-badge"
          :class="`rank-${analysis.rank}`"
        >
          <span class="rank-medal">⭐</span>
        </div>
        <div class="slice-time">
          <div class="time">
            {{ analysis.slice.startTime }}
          </div>
          <div
            class="score"
            :class="`score-${getScoreSeverity(analysis.slice.straddleScore)}`"
          >
            Score: {{ analysis.slice.straddleScore.toFixed(0) }}/100
          </div>
        </div>
      </div>

      <!-- Qualité du Mouvement -->
      <div style="flex: 0 0 auto; padding: 12px 16px; background: rgba(45, 90, 123, 0.15); border: 1px solid #2d5a7b; border-radius: 6px; font-size: 12px; min-width: 180px;">
        <div style="color: #64a5d8; margin-bottom: 6px; font-weight: bold;">
          💫 QUALITÉ
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <div style="font-size: 20px; font-weight: bold; color: #e0e7ff;">
            {{ (movementQualities[getMovementQualityKey(analysis)]?.score || 0).toFixed(0) }}
          </div>
          <span :class="['quality-badge', getQualityStatus(movementQualities[getMovementQualityKey(analysis)]?.score || 0)]">
            {{ getQualityStatusText(movementQualities[getMovementQualityKey(analysis)]?.score || 0) }}
          </span>
        </div>
      </div>

      <!-- Recommandation inline -->
      <div style="flex: 1; padding: 12px 16px; background: rgba(78, 205, 196, 0.1); border: 1px solid rgba(78, 205, 196, 0.3); border-radius: 6px; font-size: 12px;">
        <div style="color: #4ecdc4; margin-bottom: 6px; font-weight: bold;">
          🎯 RECOMMANDATION
        </div>
        <div style="color: #e0e0e0; line-height: 1.5;">
          <span v-if="analysis.slice.straddleScore >= 75 && (!volatilityDuration || volatilityDuration.confidence_score >= 50)">
            ✅ <strong>TRADER</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Straddle optimal. Risque: <strong>1% par jambe</strong>.
          </span>
          <span v-else-if="analysis.slice.straddleScore >= 60 && (!volatilityDuration || volatilityDuration.confidence_score >= 30)">
            ⚠️ <strong>TRADER</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Setup viable. Risque: <strong>1% par jambe</strong>.
          </span>
          <span v-else>
            ❌ <strong>ATTENDRE</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Setup insuffisant. Ne pas trader.
          </span>
        </div>
      </div>
    </div>

    <!-- Slot pour le contenu (MetricsGrid, etc.) -->
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { useMetricsFormatting } from '../../composables/useMetricsFormatting'

defineProps<{
  analysis: any
  volatilityDuration: any
  movementQualities?: Record<string, any>
}>()

const { getRankClass, getScoreSeverity } = useMetricsFormatting()

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
.slice-card {
  background: rgba(30, 30, 45, 0.6);
  border: 1px solid #2d3748;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.slice-card.rank-1 {
  border: 2px solid #ffd700;
  background: rgba(255, 215, 0, 0.05);
  box-shadow: 0 0 20px rgba(255, 215, 0, 0.1);
}

.rank-badge {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  background: #2d3748;
  border: 2px solid #4a5568;
}

.rank-badge.rank-1 {
  background: linear-gradient(135deg, #ffd700 0%, #ffa500 100%);
  border-color: #ffd700;
  color: #000;
  box-shadow: 0 0 10px rgba(255, 215, 0, 0.3);
}

.slice-time .time {
  font-size: 24px;
  font-weight: 800;
  color: #fff;
  line-height: 1;
  margin-bottom: 4px;
}

.slice-time .score {
  font-size: 13px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-block;
}

.score-excellent {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.score-good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}

.score-acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.score-poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Quality Badges */
.quality-badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 11px;
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
