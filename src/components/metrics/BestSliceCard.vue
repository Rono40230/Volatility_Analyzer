<template>
  <div
    class="slice-card"
    :class="getRankClass(analysis.rank)"
  >
    <!-- Rang + Heure + Score/Whipsaw -->
    <div
      class="slice-header"
      style="display: flex; justify-content: space-between; align-items: center; gap: 12px;"
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
          <div style="font-size: 13px; color: #8b949e; font-weight: 600; margin-top: 4px;">
            {{ symbol }}
          </div>
        </div>
      </div>

      <!-- Analysis Comment -->
      <div class="analysis-comment" style="flex: 1; text-align: center; color: #94a3b8; font-size: 0.95em; font-style: italic; display: flex; align-items: center; justify-content: center; gap: 8px;">
        <span style="font-style: normal;">{{ recommendation.emoji }}</span>
        <span>{{ recommendation.advice }}</span>
      </div>

      <!-- Score & Whipsaw Blocs (extracted component) -->
      <ScoreWhipsawBadges
        :score="adjustedScore"
        :whipsaw-frequency="props.whipsawAnalysis?.whipsaw_frequency_percentage || 0"
        :confidence="confidence"
      />
    </div>

    <!-- Slot pour le contenu (MetricsGrid, etc.) -->
    <div style="flex: 1; min-height: 0; display: flex; flex-direction: column;">
      <slot></slot>
    </div>
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

interface ConfidenceMetric { score: number; sample_size_warning: boolean }

const props = defineProps<{
  analysis: Analysis
  symbol?: string
  volatilityDuration: VolatilityDuration
  movementQualities?: Record<string, MovementQuality>
  whipsawAnalysis?: WhipsawAnalysis
  confidence?: ConfidenceMetric
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
  padding: 8px;
  margin-bottom: 0; /* Removed margin */
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0; /* Important for nested flex */
}

.slice-card.rank-1 {
  border: 2px solid #ffd700;
  background: rgba(255, 215, 0, 0.05);
  box-shadow: 0 0 20px rgba(255, 215, 0, 0.1);
}

.rank-badge {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
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
  font-size: 18px;
  font-weight: 800;
  color: #fff;
  line-height: 1;
  margin-bottom: 2px;
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

@media (max-width: 900px) {
  .slice-header {
    flex-wrap: wrap;
  }

  .analysis-comment {
    order: 3;
    flex-basis: 100%;
    text-align: left;
    justify-content: flex-start;
  }
}
</style>
