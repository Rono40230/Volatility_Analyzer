<template>
  <div class="controls">
    <div v-if="props.showCalendarSelector" class="control-group">
      <CalendarFileSelector 
        class="file-selector-inline"
        @file-selected="(f) => $emit('calendar-selected', f)"
      />
    </div>

    <label for="pair-select">Paire:</label>
    <select id="pair-select" v-model="selected" @change="handlePairChange" class="pair-select">
      <option value="">-- Choisir --</option>
      <option v-for="p in pairs" :key="p" :value="p">{{ p }}</option>
    </select>

    <label for="event-type-select">Type d'événement:</label>
    <SearchableEventDropdown 
      id="event-type-select"
      class="event-type-select"
      v-model="selectedEventType"
      :events="eventTypeOptions"
      :loading="eventTypesLoading"
      :error="eventTypesError"
      @update:modelValue="handleEventTypeChange"
    />

    <div v-if="eventTypesError" class="error-small">⚠️ {{ eventTypesError }}</div>
    <div v-if="!eventTypesError && eventTypeOptions.length === 0 && !eventTypesLoading" class="warning-small">📭 Aucun événement trouvé</div>
    <div v-if="eventTypesLoading" class="warning-small">⏳ Chargement des événements...</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import CalendarFileSelector from './CalendarFileSelector.vue'
import SearchableEventDropdown from './SearchableEventDropdown.vue'
import MetricTooltip from './MetricTooltip.vue'

interface EventType {
  name: string
  label: string
  count: number
}

const props = defineProps<{
  pairs: string[]
  selectedPair: string
  selectedEventType: string
  eventTypes: EventType[]
  eventTypesLoading: boolean
  eventTypesError: string | null
  showCalendarSelector?: boolean
}>()

const emit = defineEmits<{
  'update:selected-pair': [value: string]
  'update:selected-event-type': [value: string]
  'calendar-selected': [filename: string]
  'load': []
}>()

const selected = computed({
  get: () => props.selectedPair,
  set: (v) => emit('update:selected-pair', v)
})

const selectedEventType = computed({
  get: () => props.selectedEventType,
  set: (v) => emit('update:selected-event-type', v)
})

const eventTypeOptions = computed(() => props.eventTypes)

const handlePairChange = () => { emit('load') }
const handleEventTypeChange = () => { emit('load') }
</script>

<style scoped>
.controls { display: flex; gap: 10px; margin-bottom: 10px; align-items: center; flex-wrap: wrap; padding: 10px; background: #161b22; border-radius: 8px; border: 1px solid #30363d; }
.control-group { display: flex; align-items: center; gap: 8px; }
.label-with-tooltip { cursor: help; border-bottom: 1px dotted #8b949e; }
label { color: #8b949e; font-weight: 600; font-size: 0.95em; white-space: nowrap; }
.pair-select { padding: 10px 14px; border: 2px solid #30363d; border-radius: 8px; background: #ffffff; color: #000000; cursor: pointer; transition: all 0.3s; }
.pair-select:hover:not(:disabled) { border-color: #58a6ff; }
.pair-select:focus { outline: none; border-color: #58a6ff; box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.2); }
.event-type-select { min-width: 600px; width: 700px; }
.error-small { color: #f85149; font-size: 0.85em; margin-top: 5px; }
.warning-small { color: #fbbf24; font-size: 0.85em; margin-top: 5px; }

@media (max-width: 900px) {
  .event-type-select { min-width: 260px; width: 100%; }
}
</style>
