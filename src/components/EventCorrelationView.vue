<template>
  <div class="main-container">
    <div class="view-mode-selector">
      <button class="mode-button" :class="{ active: viewMode === 'by-event' }" @click="viewMode = 'by-event'">📅 Par Événement</button>
      <button class="mode-button" :class="{ active: viewMode === 'by-pair' }" @click="viewMode = 'by-pair'">💱 Par Paire</button>
      <button class="mode-button" :class="{ active: viewMode === 'heatmap' }" @click="viewMode = 'heatmap'">🔥 Heatmap</button>
      
      <!-- Calendar file selector -->
      <CalendarFileSelector 
        @file-selected="handleCalendarSelected"
        class="file-selector-right"
      />
    </div>
    <div class="content-area">
      <EventCorrelationByEvent v-if="viewMode === 'by-event'" :pastEvents="pastEvents" :calendarId="selectedCalendarId" />
      <EventCorrelationByPair v-if="viewMode === 'by-pair'" :availablePairs="availablePairs" />
      <EventCorrelationHeatmap v-if="viewMode === 'heatmap'" :calendarId="selectedCalendarId" :availablePairs="availablePairs" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import EventCorrelationByEvent from './EventCorrelationByEvent.vue'
import EventCorrelationByPair from './EventCorrelationByPair.vue'
import EventCorrelationHeatmap from './EventCorrelationHeatmap.vue'
import CalendarFileSelector from './CalendarFileSelector.vue'

interface PastEvent {
  name: string
  count: number
}

const store = useVolatilityStore()
const viewMode = ref<'by-event' | 'by-pair' | 'heatmap'>('by-event')
const pastEvents = ref<PastEvent[]>([])
const availablePairs = ref<string[]>([])
const selectedCalendarId = ref<number | null>(null)

const { onPairDataRefresh } = useDataRefresh()
const unsubscribe = onPairDataRefresh(loadAvailablePairs)
onBeforeUnmount(() => unsubscribe())

// Écouter les changements du signal de rafraîchissement
watch(() => store.dataRefreshTrigger, async () => {
  console.log('🔄 Rafraîchissement des événements déclenché')
  await loadPastEvents()
})

onMounted(async () => {
  await loadAvailablePairs()
  await loadPastEvents()
})

async function handleCalendarSelected(filename: string) {
  console.log('📅 Calendrier sélectionné:', filename)
  try {
    // Récupérer l'ID du calendrier depuis le nom du fichier
    const calendarId = await invoke<number | null>('get_calendar_id_by_filename', { filename })
    selectedCalendarId.value = calendarId
    console.log('📅 Calendar ID:', calendarId)
    
    // Recharger les événements pour ce calendrier
    await loadPastEvents()
  } catch (error) {
    console.error('Erreur lors de la récupération du calendar ID:', error)
  }
}

async function loadPastEvents() {
  if (!selectedCalendarId.value) {
    pastEvents.value = []
    return
  }
  
  try {
    const result = await invoke<PastEvent[]>('get_past_events', { 
      monthsBack: 6,
      calendarId: selectedCalendarId.value
    })
    pastEvents.value = result
    console.log('📊 Événements chargés pour calendrier', selectedCalendarId.value, ':', pastEvents.value.length, 'événements')
  } catch (error) {
    console.error('Erreur:', error)
    pastEvents.value = []
  }
}

async function loadAvailablePairs() {
  try {
    const symbolData = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
    availablePairs.value = symbolData.map(item => item.symbol)
  } catch (error) {
    console.error('Erreur:', error)
    availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
  }
}
</script>

<style scoped>
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  color: #e2e8f0;
}

.view-mode-selector {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 2px solid #30363d;
  align-items: center;
}

.mode-button {
  flex: 0 1 auto;
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

.mode-button:hover {
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

.file-selector-right {
  margin-left: auto;
}

.content-area {
  padding: 30px;
  min-height: 400px;
}
</style>
