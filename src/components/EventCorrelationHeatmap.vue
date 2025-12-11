<template>
  <div v-if="loadingHeatmap" class="loading"><span class="spinner">⏳</span><p>Génération de la heatmap...</p></div>

  <div v-if="heatmapData && !loadingHeatmap" class="heatmap-container">
    <HeatmapHeader 
      :current-filter="minVolatilityThreshold"
      @filter-click="minVolatilityThreshold = $event"
      @reload="handleReloadHeatmap"
      @archive="emit('archive-heatmap')"
    />
    <!-- Filtres supprimés -->
    <HeatmapTable 
      :pairs="heatmapData.pairs" 
      :sorted-event-types="filteredEventTypes" 
      :min-volatility="minVolatilityThreshold" 
      :get-heatmap-value="getHeatmapValue" 
      :get-heatmap-class="getHeatmapClass" 
      :get-formatted-event-name="getFormattedEventName" 
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useEventCorrelationHeatmap, type HeatmapData } from '../composables/useEventCorrelationHeatmap'
import HeatmapHeader from './HeatmapHeader.vue'
import HeatmapTable from './HeatmapTable.vue'

const props = withDefaults(defineProps<{ availablePairs?: string[]; archiveData?: HeatmapData; isArchiveMode?: boolean; calendarId?: number | null }>(), {
  availablePairs: () => [],
  isArchiveMode: false,
  calendarId: null
})

const emit = defineEmits<{
  'archive-heatmap': []
}>()

const { loadingHeatmap, heatmapData, minVolatilityThreshold, sortedEventTypes, loadHeatmapData, forceReloadHeatmap, getHeatmapValue, getHeatmapClass, getFormattedEventName } = useEventCorrelationHeatmap(props.isArchiveMode, props.archiveData)

const selectedEventType = ref('')

// Fonction exposée pour archiver la heatmap
function getHeatmapArchiveData() {
  return {
    heatmapData: heatmapData.value,
    minVolatilityThreshold: minVolatilityThreshold.value,
    // maxEventsToDisplay removed
    selectedEventType: selectedEventType.value
  }
}

// Exposer la fonction
defineExpose({
  getHeatmapArchiveData,
  handleReloadHeatmap
})

// Filter event types based on selected filter
const filteredEventTypes = computed(() => {
  if (!selectedEventType.value) {
    return sortedEventTypes.value
  }
  return sortedEventTypes.value.filter(et => et.name === selectedEventType.value)
})

// Watch availablePairs et charger la heatmap quand elles changent
watch(() => props.availablePairs, (newPairs) => {
  if (newPairs && newPairs.length > 0 && !props.isArchiveMode) {
    loadHeatmapData(newPairs, props.calendarId)
  }
}, { deep: true })

async function handleReloadHeatmap() {
  if (props.availablePairs && props.availablePairs.length > 0) {
    await forceReloadHeatmap(props.availablePairs, props.calendarId)
  }
}

onMounted(() => {
  if (props.isArchiveMode) {
    heatmapData.value = props.archiveData || null
  } else {
    // Si heatmapData est déjà cachée (restaurée du localStorage), affiche-la
    // Sinon, charge-la depuis les paires disponibles
    if (!heatmapData.value && props.availablePairs && props.availablePairs.length > 0) {
      loadHeatmapData(props.availablePairs, props.calendarId)
    }
  }
})
</script>

<style scoped>
.loading { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 400px; gap: 20px; }
.spinner { font-size: 60px; animation: hourglassFlip 1s ease-in-out infinite; display: inline-block; }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes hourglassFlip {
  0% { transform: scaleX(1) rotateY(0deg); }
  50% { transform: scaleX(-1) rotateY(180deg); }
  100% { transform: scaleX(1) rotateY(360deg); }
}
.heatmap-container { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; }
</style>
