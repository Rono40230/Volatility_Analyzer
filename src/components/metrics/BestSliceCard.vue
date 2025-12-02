<template>
  <div
    class="slice-card"
    :class="getRankClass(analysis.rank)"
  >
    <!-- Rang + Heure + Recommandation -->
    <div
      class="slice-header"
      style="display: flex; justify-content: space-between; align-items: center; gap: 20px;"
    >
      <div style="display: flex; gap: 12px; align-items: center;">
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
        </div>
      </div>

      <!-- Score & Whipsaw Blocs (extracted component) -->
      <ScoreWhipsawBadges
        :score="adjustedScore"
        :whipsaw-frequency="props.whipsawAnalysis?.whipsaw_frequency_percentage || 0"
      />

      <!-- Recommandation inline -->
      <div style="flex: 1; padding: 12px 16px; background: rgba(78, 205, 196, 0.1); border: 1px solid rgba(78, 205, 196, 0.3); border-radius: 6px; font-size: 12px;">
        <div style="color: #4ecdc4; margin-bottom: 6px; font-weight: bold;">
          🎯 RECOMMANDATION
        </div>
        <div style="color: #e0e0e0; line-height: 1.5;">
          {{ recommendation.emoji }} <strong>{{ recommendation.decision }}</strong> — {{ recommendation.advice }}
        </div>
      </div>
    </div>

    <!-- Slot pour le contenu (MetricsGrid, etc.) -->
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { useMetricsCalculations } from '../../composables/useMetricsCalculations'
import ScoreWhipsawBadges from './ScoreWhipsawBadges.vue'
import { computed } from 'vue'
import {
  calculateAdjustedScore,
  getMovementQualityKey,
  generateRecommendation
} from './BestSliceCard.helpers'

interface Slice {
  startTime: string
  straddleScore: number
  win_rate_adjusted: number
}

interface Analysis {
  rank: number
  slice: Slice
}

interface VolatilityDuration {
  confidence_score: number
}

interface WhipsawAnalysis {
  whipsaw_frequency_percentage: number
}

interface MovementQuality {
  score: number
  label: string
}

const props = defineProps<{
  analysis: Analysis
  volatilityDuration: VolatilityDuration
  movementQualities?: Record<string, MovementQuality>
  whipsawAnalysis?: WhipsawAnalysis
}>()

const { getRankClass } = useMetricsCalculations()

const adjustedScore = computed(() => {
  const brut = props.analysis?.slice?.straddleScore || 0
  return calculateAdjustedScore(brut, props.whipsawAnalysis?.whipsaw_frequency_percentage)
})

const recommendation = computed(() => {
  const adjustedWinRate = props.analysis?.slice?.win_rate_adjusted || 0
  const whipsawFreq = props.whipsawAnalysis?.whipsaw_frequency_percentage || 0
  const confidence = props.volatilityDuration?.confidence_score || 0
  
  return generateRecommendation(adjustedScore.value, whipsawFreq, adjustedWinRate, confidence)
})
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
