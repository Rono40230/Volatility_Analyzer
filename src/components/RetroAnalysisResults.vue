<template>
  <div class="cockpit-container">
    <!-- 1. Header (Titre + Conclusion + Archive) -->
    <RetroAnalysisHeader
      :event-label="props.eventLabel"
      :event-type="props.eventType"
      :pair="props.pair"
      :volatility-increase-percent="props.volatilityIncreasePercent"
      :noise-ratio-after="props.noiseRatioAfter"
      :is-archive-mode="props.isArchiveMode"
      :avg-deviation="props.avgDeviation"
      :surprise-event-count="props.surpriseEventCount"
      @archive="$emit('archive')"
    />

    <!-- 2. Main Grid (2 Colonnes) -->
    <div class="cockpit-grid">
      <!-- Colonne Gauche: Graphique (Métriques) -->
      <div class="col-center">
        <RetroAnalysisGraph
          :atr-timeline-before="props.atrTimelineBefore"
          :atr-timeline-after="props.atrTimelineAfter"
          :body-timeline-before="props.bodyTimelineBefore"
          :body-timeline-after="props.bodyTimelineAfter"
          :noise-ratio-before="props.noiseRatioBefore"
          :noise-ratio-during="props.noiseRatioDuring"
          :noise-ratio-after="props.noiseRatioAfter"
          :volatility-increase-percent="props.volatilityIncreasePercent"
          :event-count="props.eventCount"
          :event-type="props.eventType"
          :pair="props.pair"
          :event-datetime="props.eventDatetime"
          :timezone-offset="props.timezoneOffset"
          :is-archive-mode="props.isArchiveMode"
          :event-label="props.eventLabel"
          :meilleur-moment="props.meilleurMoment"
          :point-value="props.pointValue"
        />
      </div>

      <!-- Colonne Droite: Simultané -->
      <div class="col-right">
        <StraddleSimultaneousCard
          :meilleur-moment="props.meilleurMoment"
          :offset="props.offsetSimultaneous"
          :stop-loss-recovery="props.stopLossRecoverySimultaneous"
          :hard-tp="hardTpSimultaneous"
          :trailing-stop="props.trailingStopSimultaneous"
          :timeout="props.timeout"
          :pair="props.pair"
          :point-value="props.pointValue"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import RetroAnalysisGraph from './analysis/RetroAnalysisGraph.vue'
import RetroAnalysisHeader from './analysis/RetroAnalysisHeader.vue'
import StraddleSimultaneousCard from './trading/StraddleSimultaneousCard.vue'

const props = defineProps<{
  atrTimelineBefore?: number[]
  atrTimelineAfter?: number[]
  bodyTimelineBefore?: number[]
  bodyTimelineAfter?: number[]
  noiseRatioBefore: number
  noiseRatioDuring: number
  noiseRatioAfter: number
  volatilityIncreasePercent: number
  eventCount: number
  eventType: string
  pair: string
  eventDatetime?: string
  timezoneOffset?: string
  isArchiveMode?: boolean
  eventLabel?: string
  meilleurMoment?: number
  timeout?: number
  pointValue?: number
  stopLossSimultaneous?: number
  trailingStopSimultaneous?: number
  offsetSimultaneous?: number
  stopLossRecoverySimultaneous?: number
  avgDeviation?: number
  surpriseEventCount?: number
}>()

defineEmits<{ archive: [] }>()

const hardTpSimultaneous = computed(() => {
  const slRec = props.stopLossRecoverySimultaneous ?? 0
  const minRiskReward = 1.5
  return slRec * minRiskReward
})

</script>

<style scoped>
.cockpit-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
  padding: 10px;
  box-sizing: border-box;
}

.cockpit-grid {
  display: grid;
  grid-template-columns: 3fr 1fr;
  gap: 15px;
  flex: 1;
  min-height: 0; /* Important for nested scrolling if needed */
}

.col-right {
  height: 100%;
  overflow-y: auto;
}

.col-center {
  display: flex;
  flex-direction: column;
  justify-content: center;
  height: 100%;
}

/* Responsive adjustments */
@media (max-width: 1400px) {
  .cockpit-grid {
    grid-template-columns: 2fr 1fr;
  }
}

@media (max-width: 1024px) {
  .cockpit-container {
    overflow-y: auto;
  }
  
  .cockpit-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto auto;
    gap: 20px;
  }

  .col-center {
    order: -1; /* Graph first on mobile */
  }
}
</style>
