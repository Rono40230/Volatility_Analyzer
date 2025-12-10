<template>
  <div class="retro-viewer">
    <div class="results-section">
      <RetroAnalysisResults
        :atr-timeline-before="data?.atrTimelineBefore"
        :atr-timeline-after="data?.atrTimelineAfter"
        :body-timeline-before="data?.bodyTimelineBefore"
        :body-timeline-after="data?.bodyTimelineAfter"
        :noise-ratio-before="data?.noiseRatioBefore ?? 0"
        :noise-ratio-during="data?.noiseRatioDuring ?? 0"
        :noise-ratio-after="data?.noiseRatioAfter ?? 0"
        :volatility-increase-percent="data?.volatilityIncreasePercent ?? 0"
        :event-count="data?.eventCount ?? 0"
        :event-type="data?.eventType ?? ''"
        :pair="data?.pair ?? ''"
        :meilleur-moment="data?.meilleurMoment ?? 0"
        :stop-loss="data?.stopLoss ?? 0"
        :trailing-stop="data?.trailingStop ?? 0"
        :timeout="data?.timeout ?? 60"
        :event-label="data?.eventLabel"
        :is-archive-mode="true"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import RetroAnalysisResults from './RetroAnalysisResults.vue'

interface ArchivedRetroData {
  atrTimelineBefore?: number[]
  atrTimelineAfter?: number[]
  bodyTimelineBefore?: number[]
  bodyTimelineAfter?: number[]
  noiseRatioBefore?: number
  noiseRatioDuring?: number
  noiseRatioAfter?: number
  volatilityIncreasePercent?: number
  eventCount?: number
  eventType?: string
  pair?: string
  eventLabel?: string
  meilleurMoment?: number
  stopLoss?: number
  trailingStop?: number
  timeout?: number
}

const props = defineProps<{
  data: ArchivedRetroData | null
}>()

const data = computed(() => props.data)
</script>

<style scoped lang="css">
.retro-viewer {
  width: 100%;
  height: 100%;
  overflow: hidden;
  padding: 10px;
  background: #0d1117;
  color: #e2e8f0;
  display: flex;
  flex-direction: column;
}

.results-section {
  width: 100%;
  flex: 1;
  min-height: 0;
}
</style>
