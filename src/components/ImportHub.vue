<template>
  <div class="import-hub">
    <div class="sections-container">
      <CalendarImportSection
        :calendars-metadata="calendarsMetadata"
        :loading="loadingCalendars"
        :active-calendar-id="activeCalendarId"
        @import="importCalendars"
        @delete="deleteCalendar"
        @set-active="setActiveCalendar"
      />

      <PairImportSection
        :pairs-metadata="pairsMetadata"
        :loading="loadingPairs"
        @import="importPairs"
        @delete="deletePair"
      />
    </div>

    <div v-if="showDeleteConfirm" class="modal-overlay" @click.self="showDeleteConfirm = false">
      <div class="modal">
        <h3>Confirmation</h3>
        <p>{{ deleteMessage }}</p>
        <div class="modal-buttons">
          <button class="btn-confirm" @click="confirmDelete">✅ Confirmer</button>
          <button class="btn-cancel" @click="showDeleteConfirm = false">❌ Annuler</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useVolatilityStore } from '../stores/volatility'
import CalendarImportSection from './CalendarImportSection.vue'
import PairImportSection from './PairImportSection.vue'

interface CalendarMetadata {
  id: number
  name: string
  event_count: number
  start_date?: string
  end_date?: string
}

interface PairMetadataInfo {
  symbol: string
  timeframe: string
  row_count: number
  last_updated: string
  last_imported_file: string
  quality_score: number
  candle_count?: number
  start_date?: string
  end_date?: string
}

const store = useVolatilityStore()

const calendarsMetadata = ref<CalendarMetadata[]>([])
const pairsMetadata = ref<PairMetadataInfo[]>([])
const loadingCalendars = ref(false)
const loadingPairs = ref(false)
const showDeleteConfirm = ref(false)
const deleteMessage = ref('')
const deleteType = ref<'calendar' | 'pair'>('calendar')
const deleteId = ref(0)
const deleteSymbol = ref('')
const deleteTimeframe = ref('')
const activeCalendarId = ref<number | null>(null)

onMounted(async () => {
  await loadMetadata()
  const storedId = localStorage.getItem('activeCalendarId')
  if (storedId) {
    activeCalendarId.value = parseInt(storedId, 10)
  } else if (calendarsMetadata.value.length > 0) {
    setActiveCalendar(calendarsMetadata.value[0].id)
  }
})

function setActiveCalendar(id: number) {
  activeCalendarId.value = id
  localStorage.setItem('activeCalendarId', id.toString())
}

async function loadMetadata() {
  try {
    const calendars = await invoke<CalendarMetadata[]>('get_calendars_metadata')
    const pairs = await invoke<PairMetadataInfo[]>('get_pairs_metadata')
    calendarsMetadata.value = calendars || []
    pairsMetadata.value = pairs || []
  } catch (err) {
    // Erreur silencieuse
  }
}

async function importCalendars() {
  loadingCalendars.value = true
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    await invoke('import_calendar_files', { paths })
    await loadMetadata()
    store.triggerDataRefresh()
  } catch (err) {
    // Erreur silencieuse
  } finally {
    loadingCalendars.value = false
  }
}

async function importPairs() {
  loadingPairs.value = true
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    await invoke('import_pair_data', { paths })
    await loadMetadata()
    store.triggerDataRefresh()
  } catch (err) {
    // Erreur silencieuse
  } finally {
    loadingPairs.value = false
  }
}

function deleteCalendar(id: number) {
  deleteType.value = 'calendar'
  deleteId.value = id
  deleteMessage.value = 'Supprimer ce calendrier et tous ses événements?'
  showDeleteConfirm.value = true
}

function deletePair(symbol: string) {
  const pair = pairsMetadata.value.find(p => p.symbol === symbol)
  if (!pair) return
  deleteType.value = 'pair'
  deleteId.value = pair.id
  deleteSymbol.value = pair.symbol
  deleteTimeframe.value = pair.timeframe
  deleteMessage.value = 'Supprimer cette paire et toutes ses bougies?'
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  try {
    if (deleteType.value === 'calendar') {
      await invoke('delete_calendar_from_db', { calendarId: deleteId.value })
    } else {
      await invoke('delete_pair_from_db', { symbol: deleteSymbol.value, timeframe: deleteTimeframe.value })
    }
    await loadMetadata()
  } catch (err) {
    // Erreur silencieuse
  } finally {
    showDeleteConfirm.value = false
  }
}
</script>

<style scoped>
.import-hub { padding: 30px; }
.sections-container { display: grid; grid-template-columns: 1fr 1fr; gap: 30px; }
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal { background: #1a202c; padding: 30px; border-radius: 12px; border: 1px solid #2d3748; max-width: 400px; }
.modal h3 { color: #e2e8f0; }
.modal p { color: #cbd5e0; }
.modal-buttons { display: flex; gap: 10px; margin-top: 20px; }
.btn-confirm { flex: 1; padding: 10px; background: #10b981; color: white; border: none; border-radius: 6px; cursor: pointer; }
.btn-cancel { flex: 1; padding: 10px; background: #6b7280; color: white; border: none; border-radius: 6px; cursor: pointer; }
</style>
