<template>
  <div class="retro-viewer">
    <div class="results-section">
      <RetroAnalysisResults
        :peak-delay="data?.peakDelayResults?.peak_delay_minutes ?? 0"
        :decay-timeout="data?.decayResults?.recommended_timeout_minutes ?? 0"
        :peak-atr="data?.peakDelayResults?.peak_atr ?? 0"
        :decay-rate="data?.decayResults?.decay_rate_pips_per_minute ?? 0"
        :decay-speed="data?.decayResults?.decay_speed ?? 'N/A'"
        :confidence="Math.round((data?.peakDelayResults?.confidence ?? 0) * 100)"
        :event-count="data?.peakDelayResults?.event_count ?? 0"
        :entry-seconds="data?.peakDelayResults?.optimal_entry_seconds_before ?? 0"
        :is-archive-mode="true"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, computed } from 'vue'
import RetroAnalysisResults from './RetroAnalysisResults.vue'

interface ArchivedRetroData {
  peakDelayResults?: {
    peak_delay_minutes: number
    peak_atr: number
    confidence: number
    event_count: number
    optimal_entry_seconds_before: number
  }
  decayResults?: {
    recommended_timeout_minutes: number
    decay_rate_pips_per_minute: number
    decay_speed: string
  }
  pair?: string
  eventType?: string
  eventLabel?: string
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
  overflow-y: auto;
  padding: 20px;
  background: #0d1117;
  color: #e2e8f0;
}

.results-section {
  width: 100%;
  max-width: 100%;
}
</style>
