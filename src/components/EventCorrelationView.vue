<template>
  <div class="main-container">
    <!-- Header et boutons : SUPPRIMÉS -->

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

    <!-- Sélecteur calendrier simplifié : SUPPRIMÉ car inutile (un seul calendrier) -->

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
import { useHeatmapArchive } from '../composables/useHeatmapArchive'
import { useHeatmapState } from '../composables/useHeatmapState'
import EventCorrelationHeatmap from './EventCorrelationHeatmap.vue'
import RetroactiveAnalysisView from './RetroactiveAnalysisView.vue'
import ArchiveModal from './ArchiveModal.vue'

interface Props {
  viewMode?: 'heatmap' | 'retrospective'
}

const props = withDefaults(defineProps<Props>(), {
  viewMode: undefined
})

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

.view-modes {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 1px solid #30363d;
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
  overflow-y: auto; /* Enable vertical scrolling */
  display: flex;
  flex-direction: column;
}
</style>
