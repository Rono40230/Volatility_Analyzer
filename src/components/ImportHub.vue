<template>
  <div class="import-hub">
    <!-- Boutons d'action -->
    <div class="action-buttons">
      <button @click="showConversionManager = true" class="btn-conversions" title="Gérer les conversions pips/points">
        ⚙️ Gérer Conversions
      </button>
    </div>

    <div class="sections-container">
      <CalendarImportSection
        :calendars-metadata="calendarsMetadata"
        :loading="loadingCalendars"
        :active-calendar-id="activeCalendarId"
        @import="importCalendars"
        @delete="deleteCalendar"
        @set-active="setActiveCalendar"
        @clean-rare="showRareEventsModal = true"
      />
      
      <div class="market-data-container">
        <DukascopyDownloadSection
          ref="dukascopySection"
          :pairs-metadata="pairsMetadata"
          @delete="deletePair"
          @imported="loadMetadata"
        />
        <ManualMarketDataSection 
          @imported="loadMetadata"
        />
      </div>
    </div>

    <div
      v-if="showDeleteConfirm"
      class="modal-overlay"
      @click.self="showDeleteConfirm = false"
    >
      <div class="confirmation-box">
        <div class="warning-icon">
          ⚠️
        </div>
        <h3>Confirmation</h3>
        <p>{{ deleteMessage }}</p>
        <p class="warning-text">
          Cette action est irréversible.
        </p>
        <div class="modal-buttons">
          <button
            class="btn-secondary"
            @click="showDeleteConfirm = false"
          >
            Annuler
          </button>
          <button
            class="btn-danger"
            @click="confirmDelete"
          >
            Confirmer la suppression
          </button>
        </div>
      </div>
    </div>

    <RareEventsModal
      v-if="showRareEventsModal"
      :min-occurrences="5"
      :calendar-id="activeCalendarId"
      @close="showRareEventsModal = false"
      @deleted="loadMetadata"
    />

    <ConversionManagerModal
      :is-open="showConversionManager"
      :imported-pairs="pairsMetadata.map(p => p.symbol)"
      @fermer="showConversionManager = false"
      @sauvegarde="onConversionsSauvegardees"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useVolatilityStore } from '../stores/volatility'
import CalendarImportSection from './CalendarImportSection.vue'
import DukascopyDownloadSection from './DukascopyDownloadSection.vue'
import ManualMarketDataSection from './ManualMarketDataSection.vue'
import RareEventsModal from './RareEventsModal.vue'
import ConversionManagerModal from './ConversionManagerModal.vue'
import { useConversionStore } from '../stores/conversionStore'

interface CalendarMetadata { id: number; name: string; event_count: number; start_date?: string; end_date?: string }
interface PairMetadataInfo { symbol: string; timeframe: string; row_count: number; last_updated: string; last_imported_file: string; quality_score: number; candle_count?: number; start_date?: string; end_date?: string; id?: number }

const store = useVolatilityStore()
const conversionStore = useConversionStore()
const calendarsMetadata = ref<CalendarMetadata[]>([])
const pairsMetadata = ref<PairMetadataInfo[]>([])
const loadingCalendars = ref(false)
const showDeleteConfirm = ref(false)
const showRareEventsModal = ref(false)
const showConversionManager = ref(false)
const deleteMessage = ref('')
const deleteType = ref<'calendar' | 'pair'>('calendar')

const dukascopySection = ref<any>(null)

const deleteId = ref(0)
const deleteSymbol = ref('')
const deleteTimeframe = ref('')
const activeCalendarId = ref<number | null>(null)

onMounted(async () => {
  await loadMetadata()
  const storedId = localStorage.getItem('activeCalendarId')
  if (storedId) activeCalendarId.value = parseInt(storedId, 10)
  else if (calendarsMetadata.value.length > 0) setActiveCalendar(calendarsMetadata.value[0].id)
})

function setActiveCalendar(id: number) {
  activeCalendarId.value = id
  localStorage.setItem('activeCalendarId', id.toString())
}

async function loadMetadata() {
  try {
    calendarsMetadata.value = await invoke<CalendarMetadata[]>('get_calendars_metadata') || []
    pairsMetadata.value = await invoke<PairMetadataInfo[]>('get_pairs_metadata') || []
  } catch (err) { /* Silent */ }
}

async function importCalendars() {
  loadingCalendars.value = true
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] })
    if (!selected) return
    await invoke('import_calendar_files', { paths: Array.isArray(selected) ? selected : [selected] })
    await loadMetadata()
    store.triggerDataRefresh()
  } catch (err) { /* Silent */ } finally { loadingCalendars.value = false }
}

function deleteCalendar(id: number) {
  deleteType.value = 'calendar'; deleteId.value = id; deleteMessage.value = 'Supprimer ce calendrier et tous ses événements?'
  showDeleteConfirm.value = true
}

function deletePair(pair: PairMetadataInfo) {
  deleteType.value = 'pair'; deleteId.value = pair.id || 0; deleteSymbol.value = pair.symbol; deleteTimeframe.value = pair.timeframe
  deleteMessage.value = `Supprimer la paire ${pair.symbol} (${pair.timeframe}) et toutes ses bougies?`
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
    store.triggerDataRefresh() // Force refresh UI
  } catch (err) {
    alert(`Erreur lors de la suppression : ${err}`);
  } finally {
    showDeleteConfirm.value = false
  }
}

async function onConversionsSauvegardees() {
  // 1. Rafraîchir les conversions en cache dans le store
  await conversionStore.loadConversions()

  // 2. Rafraîchir les instruments dans le dropdown Dukascopy
  if (dukascopySection.value) {
    dukascopySection.value.refreshInstruments()
  }

  // 3. Nettoyer les analyses existantes et forcer le rafraîchissement
  store.clearAnalysis()
  store.triggerDataRefresh()
}
</script>

<style scoped>
.import-hub { 
  padding: 30px; 
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.btn-conversions {
  padding: 10px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.2s;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.btn-conversions:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
  background: linear-gradient(135deg, #764ba2 0%, #667eea 100%);
}

.sections-container { 
  display: grid; 
  grid-template-columns: 1fr 1fr; 
  gap: 30px; 
}

.market-data-container {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.modal-overlay { 
  position: fixed; 
  inset: 0; 
  background: rgba(15, 23, 42, 0.9); 
  backdrop-filter: blur(4px); 
  display: flex; 
  align-items: center; 
  justify-content: center; 
  z-index: 1000; 
  animation: fadeIn 0.2s; 
}

.confirmation-box { 
  background: #1e293b; 
  border: 1px solid #ef4444; 
  padding: 32px; 
  border-radius: 16px; 
  text-align: center; 
  max-width: 400px; 
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); 
  animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1); 
}

.warning-icon { 
  font-size: 48px; 
  margin-bottom: 16px; 
}
.confirmation-box h3 { color: #f8fafc; font-size: 1.25rem; margin: 0 0 12px; }
.confirmation-box p { color: #cbd5e1; margin: 0 0 8px; line-height: 1.5; }
.warning-text { color: #ef4444 !important; font-weight: 600; font-size: 0.9rem; }
.modal-buttons { display: flex; gap: 12px; margin-top: 24px; justify-content: center; }
.btn-danger { padding: 10px 20px; background: #ef4444; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; transition: background 0.2s; }
.btn-danger:hover { background: #dc2626; }
.btn-secondary { padding: 10px 20px; background: #334155; color: #e2e8f0; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; transition: background 0.2s; }
.btn-secondary:hover { background: #475569; }
.conversion-btn-wrapper { margin-top: 20px; text-align: center; }
.btn-conversion { padding: 10px 24px; background: linear-gradient(135deg, #6c5ce7 0%, #a29bfe 100%); color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; font-size: 0.95rem; transition: all 0.2s; box-shadow: 0 2px 8px rgba(108, 92, 231, 0.3); }
.btn-conversion:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(108, 92, 231, 0.4); }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
</style>
