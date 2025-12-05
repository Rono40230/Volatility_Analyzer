<template>
  <div
    v-if="isOpen"
    class="modal-overlay"
    @click.self="close"
  >
    <div class="modal-content">
      <div class="modal-header">
        <h2>✨ IAnalyse Statistique</h2>
        <button
          class="close-button"
          @click="$emit('close')"
        >
          ×
        </button>
      </div>

      <!-- Barre de Filtres -->
      <GlobalFiltersBar
        v-model:start-date="startDate"
        v-model:end-date="endDate"
        v-model:selected-pairs="selectedPairs"
        :loading="loading"
        :available-pairs="availablePairs"
        @run-analysis="runAnalysis(false)"
      />

      <div class="modal-body">
        <!-- ÉTAT 1 : CHARGEMENT (Animation Wow) -->
        <LoadingState
          v-if="loading"
          :loading-step="loadingStep"
          :progress="progress"
          :logs="logs"
        />

        <!-- ÉTAT 2 : RÉSULTATS (Dashboard) -->
        <div
          v-else-if="result"
          class="results-container"
        >
          <!-- En-tête Stats Globales -->
          <GlobalStatsGrid :result="result" />

          <!-- Dashboard Grid (Top Paires & Golden Hours) -->
          <DashboardGrid
            :result="result"
            :sorted-golden-hours="sortedGoldenHours"
            :best-hour="bestHour"
            :best-hour-reliability="bestHourReliability"
            :best-pair="bestPair"
          />

          <!-- Section Événements Tradables (pleine largeur) -->
          <TradableEventsSection :result="result" />

          <!-- Section Taux de Réussite Straddle (pleine largeur) -->
          <StraddleSuccessSection :result="result" />

          <!-- Section Fenêtres Temporelles Optimales (pleine largeur) -->
          <OptimalTimingSection :result="result" />
        </div>

        <!-- ÉTAT 3 : ERREUR -->
        <ErrorState
          v-else-if="error"
          :error="error"
          @retry="runAnalysis()"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineEmits, defineProps, watch } from 'vue'
import { useGlobalAnalysis } from '../composables/useGlobalAnalysis'
import GlobalFiltersBar from './global/GlobalFiltersBar.vue'
import LoadingState from './global/LoadingState.vue'
import GlobalStatsGrid from './global/GlobalStatsGrid.vue'
import DashboardGrid from './global/DashboardGrid.vue'
import TradableEventsSection from './global/TradableEventsSection.vue'
import StraddleSuccessSection from './global/StraddleSuccessSection.vue'
import OptimalTimingSection from './global/OptimalTimingSection.vue'
import ErrorState from './global/ErrorState.vue'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const {
  loading,
  result,
  error,
  loadingStep,
  progress,
  logs,
  startDate,
  endDate,
  selectedPairs,
  availablePairs,
  sortedGoldenHours,
  bestHour,
  bestHourReliability,
  bestPair,
  runAnalysis
} = useGlobalAnalysis()

function close() {
  emit('close')
}

watch(() => props.isOpen, (newVal) => {
  if (newVal) runAnalysis(true)
})

</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: #13131f;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  width: calc(100vw - 32px);
  max-width: calc(100vw - 32px);
  height: auto;
  max-height: calc(100vh - 32px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  color: #e2e8f0;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: linear-gradient(90deg, #1a1a2e, #16213e);
}

.modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(90deg, #4ecdc4, #556270);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  display: flex;
  align-items: center;
  gap: 10px;
}

.close-button {
  background: none;
  border: none;
  color: #a0aec0;
  font-size: 24px;
  cursor: pointer;
  transition: color 0.2s;
}

.close-button:hover {
  color: #fff;
}

.modal-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px;
  background: radial-gradient(circle at top right, #1a1a2e 0%, #13131f 100%);
}

.results-container {
  animation: fadeIn 0.5s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Scrollbar */
.modal-body::-webkit-scrollbar {
  width: 8px;
}

.modal-body::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}

.modal-body::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}

.modal-body::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
