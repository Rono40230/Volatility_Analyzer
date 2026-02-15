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
      @archive="$emit('archive')"
    />

    <!-- 2. Main Content -->
    <div class="cockpit-content">
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
        :point-value="props.pointValue"
        :context-volatility-percent="contextVolatilityPercent"
        :avg-deviation="props.avgDeviation"
        :surprise-event-count="props.surpriseEventCount"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import RetroAnalysisGraph from './analysis/RetroAnalysisGraph.vue'
import RetroAnalysisHeader from './analysis/RetroAnalysisHeader.vue'

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
  pointValue?: number
  avgDeviation?: number
  surpriseEventCount?: number
}>()

defineEmits<{ archive: [] }>()

const contextVolatilityPercent = computed(() => {
  if (!props.atrTimelineBefore?.length) return 0
  const pointValue = props.pointValue && props.pointValue > 0 ? props.pointValue : 1
  if (!pointValue) return 0
  const meanAtr = props.atrTimelineBefore.reduce((sum, value) => sum + value, 0) / props.atrTimelineBefore.length
  if (!Number.isFinite(meanAtr)) return 0
  return meanAtr / pointValue / 100
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

.cockpit-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 0;
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
