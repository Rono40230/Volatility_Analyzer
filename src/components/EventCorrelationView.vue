<template>
  <div class="main-container">
    <!-- Header et boutons : affichés SEULEMENT en mode standalone (sans prop viewMode) -->
    <div v-if="!props.viewMode" class="header-section">
      <div class="header-left">
        <h1 class="main-title">
          <span class="icon">📈</span>
          Corrélation Événements - Paires
        </h1>
        <p class="main-subtitle">Visualisez la Heatmap ou analysez les métriques rétrospectives en détail</p>
      </div>
      <CalendarFileSelector 
        class="file-selector-right"
        @file-selected="handleCalendarSelected"
      />
    </div>

    <div v-if="!props.viewMode" class="view-modes">
      <button 
        class="mode-button" 
        :class="{ active: viewMode === 'heatmap' }"
        @click="viewMode = 'heatmap'"
      >
        🔥 Heatmap de Corrélation
      </button>
      <button 
        class="mode-button" 
        :class="{ active: viewMode === 'retrospective' }"
        @click="viewMode = 'retrospective'"
      >
        📊 Métriques Rétrospectives
      </button>
    </div>

    <!-- Sélecteur calendrier simplifié : affiché SEULEMENT en mode intégré (avec prop viewMode) et en mode heatmap -->
    <div v-if="props.viewMode && viewMode === 'heatmap'" class="simple-calendar-selector">
      <CalendarFileSelector 
        class="file-selector-simple"
        @file-selected="handleCalendarSelected"
      />
    </div>

    <!-- Contenu principal : toujours affiché -->
    <div class="content-area">
      <EventCorrelationHeatmap
        v-if="viewMode === 'heatmap'"
        ref="heatmapComponentRef"
        :calendar-id="selectedCalendarId"
        :available-pairs="availablePairs"
        @archive-heatmap="openArchiveModal"
      />
      <RetroactiveAnalysisView
        v-if="viewMode === 'retrospective'"
        :calendar-id="selectedCalendarId"
        :show-calendar-selector="!!props.viewMode"
        @calendar-selected="handleCalendarSelected"
      />
    </div>

    <!-- Archive Modal -->
    <ArchiveModal
      :show="showArchiveModal"
      :archive-type="'Heatmap'"
      :period-start="archivePeriodStart"
      :period-end="archivePeriodEnd"
      :data-json="archiveDataJson"
      @close="showArchiveModal = false"
      @saved="showArchiveModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useVolatilityStore } from '../stores/volatility'
import { useAnalysisStore } from '../stores/analysisStore'
import { useHeatmapArchive } from '../composables/useHeatmapArchive'
import { useHeatmapState } from '../composables/useHeatmapState'
import EventCorrelationHeatmap from './EventCorrelationHeatmap.vue'
import CalendarFileSelector from './CalendarFileSelector.vue'
import RetroactiveAnalysisView from './RetroactiveAnalysisView.vue'
import ArchiveModal from './ArchiveModal.vue'

interface Props {
  viewMode?: 'heatmap' | 'retrospective'
}

const props = withDefaults(defineProps<Props>(), {
  viewMode: undefined
})

const volatilityStore = useVolatilityStore()
const analysisStore = useAnalysisStore()
const heatmapComponentRef = ref()

const {
  viewMode,
  availablePairs,
  selectedCalendarId,
  handleCalendarSelected,
} = useHeatmapState(props)

const {
  showArchiveModal,
  archiveDataJson,
  archivePeriodStart,
  archivePeriodEnd,
  openArchiveModal: openArchiveModalFn,
} = useHeatmapArchive()

function openArchiveModal() {
  openArchiveModalFn(heatmapComponentRef, selectedCalendarId.value)
}
</script>

<style scoped>
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  color: #e2e8f0;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.header-section {
  background: linear-gradient(135deg, #1c2128 0%, #161b22 100%);
  padding: 30px;
  border-bottom: 2px solid #30363d;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 30px;
}

.header-left {
  flex: 1;
}

.main-title {
  display: flex;
  align-items: center;
  gap: 15px;
  color: #e6edf3;
  font-size: 2em;
  margin: 0 0 10px 0;
  font-weight: 700;
}

.main-title .icon {
  font-size: 1.2em;
}

.main-subtitle {
  color: #8b949e;
  font-size: 1.1em;
  margin: 0;
  line-height: 1.5;
}

.view-modes {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 1px solid #30363d;
}

.simple-calendar-selector {
  padding: 15px 30px;
  background: #161b22;
  border-bottom: 1px solid #30363d;
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.mode-button {
  flex: 1;
  padding: 15px 20px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1.1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.mode-button:hover:not(:disabled) {
  background: #1c2128;
  border-color: #58a6ff;
  color: #58a6ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.3);
}

.mode-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 4px 12px rgba(88, 166, 255, 0.4);
}

.content-area {
  padding: 0;
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

:deep(.file-selector-right) {
  margin-left: auto;
}

:deep(.file-selector-simple) {
  margin: 0;
}
</style>
