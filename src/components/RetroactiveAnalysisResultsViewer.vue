<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content retro-modal-content">
      <div class="modal-header">
        <h2>{{ data?.eventLabel || 'Analyse Rétrospective' }}</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>
      <div class="modal-body">
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
              :offset="data?.offset ?? 0"
              :stop-loss-recovery="data?.stopLossRecovery ?? 0"
              :stop-loss-simultaneous="data?.stopLossSimultaneous"
              :trailing-stop-simultaneous="data?.trailingStopSimultaneous"
              :offset-simultaneous="data?.offsetSimultaneous"
              :stop-loss-recovery-simultaneous="data?.stopLossRecoverySimultaneous"
              :point-value="data?.pointValue"
              :event-label="data?.eventLabel"
              :avg-deviation="data?.avgDeviation"
              :surprise-event-count="data?.surpriseEventCount"
              :is-archive-mode="true"
            />
          </div>
        </div>
      </div>
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
  offset?: number
  stopLossRecovery?: number
  stopLossSimultaneous?: number
  trailingStopSimultaneous?: number
  offsetSimultaneous?: number
  stopLossRecoverySimultaneous?: number
  pointValue?: number
  avgDeviation?: number
  surpriseEventCount?: number
}

const props = defineProps<{
  data: ArchivedRetroData | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const data = computed(() => props.data)
</script>

<style scoped lang="css">
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

.modal-content {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 12px;
  width: 95vw;
  height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid #30363d;
  background: #161b22;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.2em;
  color: #e2e8f0;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: #8b949e;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
  line-height: 1;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.modal-body {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.retro-viewer {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.results-section {
  width: 100%;
  flex: 1;
  min-height: 0;
}
</style>
