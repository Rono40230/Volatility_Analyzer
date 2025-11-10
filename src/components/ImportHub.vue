<template>
  <div class="import-hub">
    <div class="sections-container">
    <!-- SECTION 1: Calendrier Économique -->
    <section class="import-section calendar-section">
      <div class="section-title">
        <h3>📅 Calendriers Économique</h3>
      </div>

      <!-- Info sur l'import existant -->
      <div v-if="calendarsMetadata.length > 0" class="info-box success">
        <div class="info-row">
          <span class="label">� Calendriers en base :</span>
          <span class="value">{{ calendarsMetadata.length }}</span>
        </div>
        <div class="info-row">
          <span class="label">� Événements importés :</span>
          <span class="value">{{ calendarsMetadata.reduce((sum, c) => sum + c.event_count, 0).toLocaleString() }}</span>
        </div>
        <div class="info-row" v-if="calendarsMetadata[0]?.last_updated">
          <span class="label">� Dernier import :</span>
          <span class="value">{{ formatDate(calendarsMetadata[0].last_updated) }}</span>
        </div>
      </div>
      <div v-else-if="!loadingCalendars" class="info-box warning">
        ⚠️ Aucun calendrier importé. Importez un fichier pour commencer.
      </div>

      <!-- Tableau des calendriers -->
      <div v-if="calendarsMetadata.length > 0" class="calendars-table-container">
        <table class="calendars-table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Événements</th>
              <th>Période</th>
              <th>Fichier Source</th>
              <th>Actif</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cal in calendarsMetadata" :key="cal.id">
              <td><strong>{{ cal.name }}</strong></td>
              <td>{{ cal.event_count.toLocaleString() }}</td>
              <td>{{ cal.start_date }} → {{ cal.end_date }}</td>
              <td><small>{{ cal.last_imported_file }}</small></td>
              <td>
                <input 
                  type="radio" 
                  :name="'active_calendar'"
                  :value="cal.id"
                  v-model="activeCalendarId"
                  @change="saveActiveCalendarId"
                />
              </td>
              <td class="actions-cell">
                <button 
                  @click="confirmDeleteCalendar(cal.id, cal.name)"
                  class="btn btn-danger btn-small"
                  title="Supprimer ce calendrier"
                >
                  🗑️
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Bouton import -->
      <div class="import-button-container">
        <button @click="selectCalendarFiles" class="btn btn-primary" :disabled="importingCalendars">
          📥 {{ importingCalendars ? 'Import en cours...' : 'Importer un calendrier' }}
        </button>
      </div>
    </section>

    <!-- SECTION 2: Données de Paires -->
    <section class="import-section pairs-section">
      <div class="section-title">
        <h3>💱 Paires</h3>
      </div>

      <!-- Info sur l'import existant -->
      <div v-if="pairsMetadata.length > 0" class="info-box success">
        <div class="info-row">
          <span class="label">💱 Paires en base :</span>
          <span class="value">{{ pairsMetadata.length }}</span>
        </div>
        <div class="info-row">
          <span class="label">📊 Bougies importées :</span>
          <span class="value">{{ pairsMetadata.reduce((sum, p) => sum + p.row_count, 0).toLocaleString() }}</span>
        </div>
        <div class="info-row" v-if="pairsMetadata[0]?.last_updated">
          <span class="label">🕒 Dernier import :</span>
          <span class="value">{{ formatDate(pairsMetadata[0].last_updated) }}</span>
        </div>
      </div>
      <div v-else-if="!loadingPairs" class="info-box warning">
        ⚠️ Aucune donnée de paire importée. Importez des fichiers CSV pour commencer.
      </div>

      <!-- Tableau des paires -->
      <div v-if="pairsMetadata.length > 0" class="pairs-table-container">
        <table class="pairs-table">
          <thead>
            <tr>
              <th>Paire</th>
              <th>Timeframe</th>
              <th>Bougies</th>
              <th>Fichier Source</th>
              <th>Qualité</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="pair in pairsMetadata" :key="`${pair.symbol}-${pair.timeframe}`">
              <td><strong>{{ pair.symbol }}</strong></td>
              <td>{{ pair.timeframe }}</td>
              <td>{{ pair.row_count.toLocaleString() }}</td>
              <td><small>{{ pair.last_imported_file }}</small></td>
              <td>
                <span :class="['quality-badge', qualityLevel(pair.quality_score)]">
                  {{ Math.round(pair.quality_score * 100) }}%
                </span>
              </td>
              <td class="actions-cell">
                <button 
                  @click="confirmDeletePair(pair.symbol, pair.timeframe)"
                  class="btn btn-danger btn-small"
                  title="Supprimer cette paire"
                >
                  🗑️
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Bouton import -->
      <div class="import-button-container">
        <button @click="selectPairsFiles" class="btn btn-primary" :disabled="importingPairs">
          📥 {{ importingPairs ? 'Import en cours...' : 'Importer une paire' }}
        </button>
      </div>
    </section>
    </div>

    <!-- MODAL DE SUPPRESSION -->
    <div v-if="deleteModal.show" class="modal-overlay" @click="cancelDelete">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>⚠️ Confirmer la suppression</h2>
          <button @click="cancelDelete" class="modal-close">&times;</button>
        </div>
        <div class="modal-body">
          <p>{{ deleteModal.message }}</p>
        </div>
        <div class="modal-footer">
          <button @click="cancelDelete" class="btn btn-secondary">Annuler</button>
          <button @click="confirmDelete" class="btn btn-danger">Supprimer</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useDataRefresh } from '../composables/useDataRefresh'
import CalendarFileSelector from './CalendarFileSelector.vue'

const { triggerPairDataRefresh } = useDataRefresh()

// État de la modal de suppression
interface DeleteModalState {
  show: boolean
  type: 'pair' | 'calendar' | null
  message: string
  symbol?: string
  timeframe?: string
  calendarId?: number
  calendarName?: string
}

const deleteModal = ref<DeleteModalState>({
  show: false,
  type: null,
  message: ''
})

interface CalendarImportInfo {
  total_events: number
  last_import_date: string | null
  oldest_event_date: string | null
  newest_event_date: string | null
}

interface CalendarMetadata {
  id: number
  name: string
  event_count: number
  start_date: string | null
  end_date: string | null
  last_updated: string
  last_imported_file: string
}

interface PairMetadata {
  symbol: string
  timeframe: string
  row_count: number
  last_updated: string
  last_imported_file: string
  quality_score: number
}

const calendarFileSelectorRef = ref<any>()
const calendarsMetadata = ref<CalendarMetadata[]>([])
const pairsMetadata = ref<PairMetadata[]>([])
const activeCalendarId = ref<number | null>(null)
const loadingCalendars = ref(false)
const loadingPairs = ref(false)
const importingCalendars = ref(false)
const importingPairs = ref(false)

async function loadCalendarsInfo() {
  loadingCalendars.value = true
  try {
    const rawData = await invoke<any[]>('get_calendar_imports_from_db')
    console.log('📋 Calendar metadata (raw):', rawData)
    
    // Mapper les données brutes vers CalendarMetadata
    calendarsMetadata.value = rawData.map(cal => ({
      id: cal.id,
      name: cal.name,
      event_count: cal.event_count,
      start_date: cal.oldest_event_date,
      end_date: cal.newest_event_date,
      last_updated: cal.imported_at,
      last_imported_file: cal.filename
    }))
    
    // Charger le calendrier actif sauvegardé
    const saved = localStorage.getItem('activeCalendarId')
    if (saved) {
      activeCalendarId.value = parseInt(saved, 10)
    } else if (calendarsMetadata.value.length > 0) {
      // Sélectionner le premier par défaut
      activeCalendarId.value = calendarsMetadata.value[0].id
      saveActiveCalendarId()
    }
  } catch (e) {
    console.error('❌ Erreur calendriers:', e)
  } finally {
    loadingCalendars.value = false
  }
}

async function loadPairsInfo() {
  loadingPairs.value = true
  try {
    const metadata = await invoke<PairMetadata[]>('get_pair_metadata_from_db')
    console.log('📋 Pair metadata returned:', metadata)
    console.log('📊 Pair count:', metadata.length)
    pairsMetadata.value = metadata
  } catch (e) {
    console.error('❌ Erreur paires:', e)
  } finally {
    loadingPairs.value = false
  }
}

function saveActiveCalendarId() {
  if (activeCalendarId.value) {
    localStorage.setItem('activeCalendarId', activeCalendarId.value.toString())
    console.log('💾 Active calendar saved:', activeCalendarId.value)
  }
}

async function selectCalendarFiles() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Fichiers CSV',
        extensions: ['csv']
      }]
    })

    if (!selected) return

    importingCalendars.value = true
    const path = Array.isArray(selected) ? selected[0] : selected

    console.log('📥 Starting calendar import for:', path)
    await invoke('import_calendar_files', { paths: [path] })
    console.log('✅ Calendar import complete')
    
    // Recharger les données
    await loadCalendarsInfo()
  } catch (e) {
    console.error('❌ Erreur import calendrier:', e)
  } finally {
    importingCalendars.value = false
  }
}

async function refreshAll() {
  console.log('🔄 Refreshing all data...')
  await loadCalendarsInfo()
  await loadPairsInfo()
  
  // Déclencher le rafraîchissement des symboles dans les autres composantes
  await triggerPairDataRefresh()
}

async function selectPairsFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Fichiers CSV',
        extensions: ['csv']
      }]
    })

    if (!selected || selected.length === 0) return

    importingPairs.value = true
    const paths = Array.isArray(selected) ? selected : [selected]

    console.log('📥 Starting import for:', paths)
    await invoke('import_and_clean_files', { paths })
    console.log('✅ Import complete')
    
    // Recharger les données après import
    await refreshAll()
  } catch (e) {
    console.error('❌ Erreur import:', e)
  } finally {
    importingPairs.value = false
  }
}

function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('fr-FR', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  } catch {
    return dateString
  }
}

function qualityLevel(score: number): string {
  if (score >= 0.95) return 'excellent'
  if (score >= 0.80) return 'good'
  if (score >= 0.60) return 'fair'
  return 'poor'
}

function confirmDeletePair(symbol: string, timeframe: string) {
  const pairData = pairsMetadata.value.find(p => p.symbol === symbol)
  const candleCount = pairData?.row_count.toLocaleString() ?? 'N/A'
  
  deleteModal.value = {
    show: true,
    type: 'pair',
    message: `Êtes-vous certain de vouloir supprimer la paire ${symbol}/${timeframe} ?\n\nCette action est irréversible et supprimera TOUS les candles (${candleCount} lignes) de la base de données.`,
    symbol,
    timeframe
  }
}

async function deletePair(symbol: string, timeframe: string) {
  try {
    const result = await invoke<string>('delete_pair_from_db', { symbol, timeframe })
    console.log('✅ Paire supprimée:', result)
    
    // Recharger les données
    await loadPairsInfo()
  } catch (e) {
    const error = e as Error
    console.error('❌ Erreur suppression paire:', error)
    alert(`Erreur suppression: ${error.message}`)
  }
}

function confirmDeleteCalendar(calendarId: number, calendarName: string) {
  deleteModal.value = {
    show: true,
    type: 'calendar',
    message: `Êtes-vous certain de vouloir supprimer le calendrier "${calendarName}" ?\n\nCette action est irréversible et supprimera TOUS les événements de la base de données.`,
    calendarId,
    calendarName
  }
}

async function deleteCalendar(calendarId: number, calendarName: string) {
  try {
    const result = await invoke<string>('delete_calendar_from_db', { calendarId })
    console.log('✅ Calendrier supprimé:', result)
    
    // Si le calendrier supprimé était sélectionné, désélectionner
    if (activeCalendarId.value === calendarId) {
      activeCalendarId.value = null
      localStorage.removeItem('activeCalendarId')
      console.log('📝 Calendrier sélectionné désélectionné')
    }
    
    // Recharger les données
    await loadCalendarsInfo()
  } catch (e) {
    const error = e as Error
    console.error('❌ Erreur suppression calendrier:', error)
    alert(`Erreur suppression: ${error.message}`)
  }
}

function cancelDelete() {
  deleteModal.value = {
    show: false,
    type: null,
    message: ''
  }
}

async function confirmDelete() {
  try {
    if (deleteModal.value.type === 'pair' && deleteModal.value.symbol && deleteModal.value.timeframe) {
      await deletePair(deleteModal.value.symbol, deleteModal.value.timeframe)
    } else if (deleteModal.value.type === 'calendar' && deleteModal.value.calendarId && deleteModal.value.calendarName) {
      await deleteCalendar(deleteModal.value.calendarId, deleteModal.value.calendarName)
    }
  } finally {
    cancelDelete()
  }
}

onMounted(() => {
  loadCalendarsInfo()
  loadPairsInfo()
})
</script>

<style scoped>
.import-hub {
  padding: 20px;
  width: 100%;
  margin: 0;
}

.sections-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 25px;
  margin-bottom: 25px;
  width: 100%;
}

.hub-header {
  margin-bottom: 30px;
  text-align: center;
}

.hub-header h2 {
  font-size: 2em;
  color: #58a6ff;
  margin-bottom: 10px;
}

.subtitle {
  color: #8b949e;
  font-size: 1.05em;
}

.import-section {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 25px;
  margin-bottom: 0;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  width: 100%;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 15px;
  margin-bottom: 20px;
  border-bottom: 2px solid #30363d;
  padding-bottom: 15px;
}

.section-title h3 {
  margin: 0;
  color: #58a6ff;
  font-size: 1.3em;
}

.db-badge {
  display: inline-block;
  background: #238636;
  color: white;
  padding: 4px 12px;
  border-radius: 16px;
  font-size: 0.85em;
  font-weight: 600;
}

.info-box {
  background: rgba(46, 160, 67, 0.1);
  border-left: 4px solid #238636;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
}

.info-box.warning {
  background: rgba(210, 153, 34, 0.1);
  border-left-color: #d29922;
  color: #d29922;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.info-row:last-child {
  border-bottom: none;
}

.label {
  font-weight: 600;
  color: #c9d1d9;
}

.value {
  color: #58a6ff;
  font-weight: 500;
}

.pairs-table-container {
  overflow-x: auto;
  margin-bottom: 20px;
}

.pairs-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.95em;
}

.pairs-table thead {
  background: rgba(56, 139, 253, 0.1);
  border-bottom: 2px solid #30363d;
}

.pairs-table th {
  padding: 12px;
  text-align: left;
  color: #58a6ff;
  font-weight: 600;
}

.pairs-table td {
  padding: 12px;
  border-bottom: 1px solid #30363d;
}

.pairs-table tbody tr:hover {
  background: rgba(56, 139, 253, 0.05);
}

.calendars-table-container {
  overflow-x: auto;
  margin-bottom: 20px;
}

.calendars-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.95em;
}

.calendars-table thead {
  background: rgba(56, 139, 253, 0.1);
  border-bottom: 2px solid #30363d;
}

.calendars-table th {
  padding: 12px;
  text-align: left;
  color: #58a6ff;
  font-weight: 600;
}

.calendars-table td {
  padding: 12px;
  border-bottom: 1px solid #30363d;
}

.calendars-table tbody tr:hover {
  background: rgba(56, 139, 253, 0.05);
}

.calendars-table input[type="radio"] {
  cursor: pointer;
  width: 18px;
  height: 18px;
  accent-color: #238636;
}

.quality-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.9em;
}

.quality-badge.excellent {
  background: #238636;
  color: white;
}

.quality-badge.good {
  background: #2da44e;
  color: white;
}

.quality-badge.fair {
  background: #d29922;
  color: #1a1a1a;
}

.quality-badge.poor {
  background: #da3633;
  color: white;
}

.import-button-container {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary {
  background: linear-gradient(135deg, #238636 0%, #2da44e 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #2da44e 0%, #3fb950 100%);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(46, 160, 67, 0.3);
}

.btn-danger {
  background: #da3633;
  color: white;
  padding: 8px 12px;
  font-size: 0.9em;
}

.btn-danger:hover:not(:disabled) {
  background: #f85149;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(218, 54, 51, 0.3);
}

.btn-small {
  padding: 6px 10px;
  font-size: 0.85em;
}

.actions-cell {
  text-align: center;
  white-space: nowrap;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status-section {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  border: none;
}

.status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.status-card {
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 15px;
}

.status-icon {
  font-size: 2.5em;
}

.status-info {
  flex: 1;
}

.status-label {
  font-size: 0.95em;
  color: rgba(255,255,255,0.8);
  margin-bottom: 5px;
}

.status-value {
  font-size: 1.2em;
  font-weight: 600;
  color: white;
}

.status-value.error {
  color: #ff6b6b;
}

.file-selector {
  margin-bottom: 15px;
}

.status-section {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  border: none;
}

.status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.status-card {
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 15px;
}

.status-icon {
  font-size: 2.5em;
}

.status-info {
  flex: 1;
}

.status-label {
  font-size: 0.95em;
  color: rgba(255,255,255,0.8);
  margin-bottom: 5px;
}

.status-value {
  font-size: 1.2em;
  font-weight: 600;
  color: white;
}

.status-value.error {
  color: #ff6b6b;
}

@media (max-width: 1024px) {
  .sections-container {
    grid-template-columns: 1fr;
  }

  .status-grid {
    grid-template-columns: 1fr;
  }

  .info-row {
    flex-direction: column;
    gap: 5px;
  }

  .pairs-table {
    font-size: 0.85em;
  }

  .pairs-table th,
  .pairs-table td {
    padding: 8px;
  }
}

/* STYLES MODAL DE SUPPRESSION */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border: 1px solid #30363d;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  max-width: 500px;
  width: 90%;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border-bottom: 1px solid #30363d;
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
}

.modal-header h2 {
  margin: 0;
  color: white;
  font-size: 1.3em;
}

.modal-close {
  background: none;
  border: none;
  color: white;
  font-size: 28px;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity 0.2s ease;
}

.modal-close:hover {
  opacity: 0.8;
}

.modal-body {
  padding: 24px;
  color: #c9d1d9;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 24px;
  border-top: 1px solid #30363d;
  background: rgba(30, 30, 30, 0.5);
}

.btn-secondary {
  background: #30363d;
  color: #c9d1d9;
  border: 1px solid #444c56;
}

.btn-secondary:hover:not(:disabled) {
  background: #444c56;
  border-color: #58a6ff;
}

.btn-danger {
  background: linear-gradient(135deg, #da3633 0%, #f85149 100%);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: linear-gradient(135deg, #f85149 0%, #ff7b72 100%);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(248, 81, 73, 0.3);
}

.btn-small {
  padding: 6px 12px;
  font-size: 0.9em;
}

.actions-cell {
  text-align: center;
  width: 60px;
}
</style>

