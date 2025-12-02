<template>
  <div v-if="loadingHeatmap" class="loading"><div class="spinner" /><p>Génération de la heatmap...</p></div>

  <div v-if="heatmapData && !loadingHeatmap" class="heatmap-container">
    <HeatmapHeader />
    <HeatmapFilters 
      :min-volatility="minVolatilityThreshold" 
      :max-events="maxEventsToDisplay" 
      :available-event-types="availableEventTypes"
      @update:min-volatility="minVolatilityThreshold = $event" 
      @update:max-events="maxEventsToDisplay = $event"
      @update:selected-event-type="selectedEventType = $event"
    />
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
import { useEventCorrelationHeatmap } from '../composables/useEventCorrelationHeatmap'
import HeatmapHeader from './HeatmapHeader.vue'
import HeatmapFilters from './HeatmapFilters.vue'
import HeatmapTable from './HeatmapTable.vue'

const props = withDefaults(defineProps<{ availablePairs?: string[]; archiveData?: any; isArchiveMode?: boolean; calendarId?: number | null }>(), {
  availablePairs: () => [],
  isArchiveMode: false,
  calendarId: null
})

const { loadingHeatmap, heatmapData, minVolatilityThreshold, maxEventsToDisplay, sortedEventTypes, loadHeatmapData, getHeatmapValue, getHeatmapClass, getFormattedEventName } = useEventCorrelationHeatmap(props.isArchiveMode, props.archiveData)

const selectedEventType = ref('')

// Available event types for dropdown
const availableEventTypes = computed(() => {
  return sortedEventTypes.value?.map(et => et.name) ?? []
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
    loadHeatmapData(newPairs)
  }
}, { deep: true })

onMounted(() => {
  if (props.isArchiveMode) {
    heatmapData.value = props.archiveData || null
  }
})
</script>

<style scoped>
.loading { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 400px; gap: 20px; }
.spinner { width: 40px; height: 40px; border: 4px solid #30363d; border-top-color: #58a6ff; border-radius: 50%; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.heatmap-container { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; }
</style>
