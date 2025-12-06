<script setup lang="ts">
import { computed } from 'vue'
import { useEventPairCorrelation } from '../../composables/useEventPairCorrelation'
import { useEventTranslation } from '../../composables/useEventTranslation'
import EventDetailPairs from './EventDetailPairs.vue'
import EventDetailStraddle from './EventDetailStraddle.vue'
import type { EventDetailState } from '../../composables/useEventDetail'

interface Props {
  isOpen: boolean
  event: EventDetailState
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const { getPairsByEvent } = useEventPairCorrelation()
const { translateEventName } = useEventTranslation()

const allPairs = computed(() => {
  if (!props.event.eventType) return []
  return getPairsByEvent.value(props.event.eventType)
})

// Les helpers getImpactColor et getImpactIcon ont été movés dans EventDetailPairs.vue
</script>

<template>
  <Teleport to="body">
    <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content" @click.stop>
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <h2 class="event-title">{{ translateEventName(event.eventType || '') }}</h2>
          <span class="tradability-badge" :data-level="(event.tradability || 'risqué').toLowerCase()">
            {{ event.tradability }}
          </span>
        </div>
        <button class="close-button" @click="emit('close')">✕</button>
      </div>

      <!-- Main Stats -->
      <div class="stats-row">
        <div class="stat-box">
          <div class="stat-label">Score</div>
          <div class="stat-value">{{ Math.round(event.score || 0) }}/100</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">Volatilité ATR</div>
          <div class="stat-value">{{ Math.round((event.avgATR || 0) * 10) / 10 }}p</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">Pic (+/-)</div>
          <div class="stat-value">+{{ Math.round((event.avgPeakDelay || 0) * 10) / 10 }}min</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">Confiance</div>
          <div class="stat-value">{{ Math.round(event.avgConfidence || 0) }}%</div>
        </div>
      </div>

      <!-- All Pairs Grid -->
      <EventDetailPairs :pairs="allPairs" />

      <!-- Straddle Setup -->
      <EventDetailStraddle :avg-atr="event.avgATR || 0" :avg-peak-delay="event.avgPeakDelay || 0" />
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.2s ease-out;
}

.modal-content {
  background: #13131f;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  max-width: 800px;
  width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
  animation: slideUp 0.3s ease-out;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  background: linear-gradient(135deg, #1a1a2e, #16213e);
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex: 1;
}

.event-title {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
}

.tradability-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.tradability-badge[data-level="optimal"] {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.tradability-badge[data-level="bon"] {
  background: rgba(251, 146, 60, 0.2);
  color: #fb923c;
}

.tradability-badge[data-level="risqué"] {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.close-button {
  background: none;
  border: none;
  color: #a0aec0;
  font-size: 28px;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s;
}

.close-button:hover {
  color: #ffffff;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
  padding: 20px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.stat-box {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
}

.stat-label {
  font-size: 11px;
  color: #a0aec0;
  text-transform: uppercase;
  margin-bottom: 6px;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: #3b82f6;
}

/* Styles pour les sous-composants EventDetailPairs et EventDetailStraddle */
/* Voir leurs fichiers respectifs pour les styles détaillés */
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Scrollbar */
.modal-content::-webkit-scrollbar {
  width: 8px;
}

.modal-content::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}

.modal-content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}

.modal-content::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
