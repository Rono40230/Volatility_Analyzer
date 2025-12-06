<template>
  <div class="container">
    <div class="controls">
      <div v-if="props.showCalendarSelector" class="control-group">
        <CalendarFileSelector 
          class="file-selector-inline"
          @file-selected="onCalendarSelected"
        />
      </div>

      <label for="pair-select">Paire:</label>
      <select id="pair-select" v-model="selected" @change="load" class="pair-select">
        <option value="">-- Choisir --</option>
        <option v-for="p in pairs" :key="p" :value="p">{{ p }}</option>
      </select>
      <label for="event-type-select">Type d'événement:</label>
      <SearchableEventDropdown 
        id="event-type-select"
        v-model="selectedEventType"
        :events="eventTypeOptions"
        :loading="eventTypesLoading"
        :error="eventTypesError"
        @update:modelValue="load"
      />
      <div v-if="eventTypesError" class="error-small">⚠️ {{ eventTypesError }}</div>
      <div v-if="!eventTypesError && eventTypes.length === 0 && !eventTypesLoading" class="warning-small">📭 Aucun événement trouvé</div>
      <div v-if="eventTypesLoading" class="warning-small">⏳ Chargement des événements...</div>
    </div>

    <div v-if="loading" class="spinner">⏳ Chargement...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="peakDelayLoading || decayLoading" class="spinner">⏳ Chargement des analyses...</div>
    <div v-else-if="peakDelayError" class="error">❌ Erreur Peak Delay: {{ peakDelayError }}</div>
    <div v-else-if="decayError" class="error">❌ Erreur Décroissance: {{ decayError }}</div>
    <div v-else-if="!peakDelayResults || !decayResults" class="empty">📭 Chargez une paire et sélectionnez un événement</div>

    <RetroAnalysisResults
      v-else
      :peak-delay="peakDelayResults.peak_delay_minutes"
      :decay-timeout="decayResults.recommended_timeout_minutes"
      :peak-atr="peakDelayResults.peak_atr"
      :decay-rate="decayResults.decay_rate_pips_per_minute"
      :decay-speed="decayResults.decay_speed"
      :confidence="Math.round(peakDelayResults.confidence * 100)"
      :event-count="peakDelayResults.event_count"
      :event-label="getEventLabel(selectedEventType)"
      @archive="openArchiveModal"
    />

    <ArchiveModal 
      :show="showArchiveModal" 
      archive-type="Métriques Rétrospectives" 
      :period-start="archivePeriodStart" 
      :period-end="archivePeriodEnd" 
      :symbol="selected" 
      :event-name="selectedEventType" 
      :event-name-fr="getEventLabel(selectedEventType)" 
      :data-json="archiveDataJson" 
      @close="showArchiveModal = false" 
      @saved="handleArchiveSaved" 
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'
import ArchiveModal from './ArchiveModal.vue'
import CalendarFileSelector from './CalendarFileSelector.vue'
import RetroAnalysisResults from './RetroAnalysisResults.vue'
import SearchableEventDropdown from './SearchableEventDropdown.vue'

interface Symbol { symbol: string; file_path?: string }

const props = defineProps<{ 
  calendarId: number | null
  showCalendarSelector?: boolean
}>()

const emit = defineEmits<{
  'calendar-selected': [filename: string]
}>()

const onCalendarSelected = (filename: string) => {
  emit('calendar-selected', filename)
}

const { peakDelayLoading, peakDelayError, peakDelayResults, analyzePeakDelay, 
         decayLoading, decayError, decayResults, analyzeDecayProfile, 
         eventTypes, eventTypesError, eventTypesLoading, loadEventTypes, getEventLabel } = useRetrospectiveAnalysis()

const pairs = ref<string[]>([])
const selected = ref('')
const selectedEventType = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')

const eventTypeOptions = computed(() =>
  eventTypes.value.map(et => ({
    name: et.name,
    label: getEventLabel(et.name),
    count: et.count
  }))
)

onMounted(async () => {
  try {
    const s = await invoke<Symbol[]>('load_symbols')
    pairs.value = s.map((x: Symbol) => x.symbol)
  } catch (e) {
    pairs.value = ['EURUSD']
  }
  await loadEventTypes(props.calendarId ?? undefined)
})

async function load() {
  if (!selected.value || !selectedEventType.value) return
  loading.value = true
  error.value = null
  try {
    await analyzePeakDelay(selected.value, selectedEventType.value)
    await analyzeDecayProfile(selected.value, selectedEventType.value)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function openArchiveModal() {
  if (!peakDelayResults.value || !decayResults.value || !selected.value || !selectedEventType.value) return
  archivePeriodStart.value = peakDelayResults.value.event_date_min
  archivePeriodEnd.value = peakDelayResults.value.event_date_max
  archiveDataJson.value = JSON.stringify({
    peakDelayResults: peakDelayResults.value,
    decayResults: decayResults.value,
    pair: selected.value,
    eventType: selectedEventType.value,
    eventLabel: getEventLabel(selectedEventType.value)
  })
  showArchiveModal.value = true
}

function handleArchiveSaved() {
  showArchiveModal.value = false
}
</script>

<style scoped>
.container { height: 100dvh; padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; display: flex; flex-direction: column; overflow: hidden; }
.controls { margin-bottom: 20px; display: flex; gap: 20px; align-items: flex-end; flex-wrap: nowrap; flex-shrink: 0; }
.control-group { display: flex; align-items: flex-end; gap: 10px; flex-shrink: 0; }
label { display: block; color: #e2e8f0; font-weight: 600; margin-bottom: 8px; }
:deep(.file-selector-inline) { margin: 0; }
.pair-select { width: 200px; padding: 12px 16px; font-size: 1em; border: 2px solid #4a5568; border-radius: 8px; background: #2d3748; color: #000000 !important; cursor: pointer; transition: all 0.3s; }
.pair-select:hover { border-color: #667eea; background: #374151; }
.pair-select:focus { outline: none; border-color: #667eea; box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2); }
.pair-select option { background: #ffffff; color: #000000 !important; }
.spinner { text-align: center; color: #8b949e; padding: 40px; font-size: 1.2em; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 20px; min-height: 300px; }
.spinner::before { content: '⏳'; font-size: 60px; animation: hourglassFlip 1s ease-in-out infinite; display: block; order: -1; }
.empty { text-align: center; color: #8b949e; padding: 40px; font-size: 1.2em; }
.error { background: #3d2626; color: #f85149; padding: 15px; border-radius: 8px; margin-bottom: 20px; }
.error-small { font-size: 0.85em; color: #f85149; margin-top: 5px; }
.warning-small { font-size: 0.85em; color: #d29922; margin-top: 5px; }
@keyframes hourglassFlip { 0% { transform: scaleX(1) rotateY(0deg); } 50% { transform: scaleX(-1) rotateY(180deg); } 100% { transform: scaleX(1) rotateY(360deg); } }
</style>
