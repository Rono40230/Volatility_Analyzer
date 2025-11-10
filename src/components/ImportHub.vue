<template>
  <div class="import-hub">
    <div class="sections-container">
      <section class="import-section">
        <h3>📅 Calendriers Économique</h3>
        <div v-if="calendarsMetadata.length > 0" class="info-box">
          <div>Calendriers: {{ calendarsMetadata.length }}</div>
          <div>Événements: {{ calendarsMetadata.reduce((s, c) => s + c.event_count, 0).toLocaleString() }}</div>
        </div>
        <div v-else class="info-box warning">Aucun calendrier importé.</div>
        <div v-if="calendarsMetadata.length > 0" class="table-container">
          <table class="data-table">
            <thead>
              <tr><th>Nom</th><th>Événements</th><th>Période</th><th>Actions</th></tr>
            </thead>
            <tbody>
              <tr v-for="cal in calendarsMetadata" :key="cal.id">
                <td><strong>{{ cal.name }}</strong></td>
                <td>{{ cal.event_count.toLocaleString() }}</td>
                <td>{{ cal.start_date }} → {{ cal.end_date }}</td>
                <td><button @click="deleteCalendar(cal.id)" class="btn-delete">🗑️ Supprimer</button></td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="file-input-group">
          <input type="file" ref="calendarFile" accept=".csv" />
          <button @click="importCalendars" class="btn-import">📥 Importer calendrier</button>
        </div>
      </section>

      <section class="import-section">
        <h3>💱 Paires de Trading</h3>
        <div v-if="pairsMetadata.length > 0" class="info-box">
          <div>Paires: {{ pairsMetadata.length }}</div>
          <div>Bougies: {{ pairsMetadata.reduce((s, p) => s + p.candle_count, 0).toLocaleString() }}</div>
        </div>
        <div v-else class="info-box warning">Aucune paire importée.</div>
        <div v-if="pairsMetadata.length > 0" class="table-container">
          <table class="data-table">
            <thead>
              <tr><th>Paire</th><th>Bougies</th><th>Période</th><th>Actions</th></tr>
            </thead>
            <tbody>
              <tr v-for="pair in pairsMetadata" :key="pair.id">
                <td><strong>{{ pair.symbol }}</strong></td>
                <td>{{ pair.candle_count.toLocaleString() }}</td>
                <td>{{ pair.start_date }} → {{ pair.end_date }}</td>
                <td><button @click="deletePair(pair.id)" class="btn-delete">🗑️ Supprimer</button></td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="file-input-group">
          <input type="file" ref="pairFile" accept=".csv" />
          <button @click="importPairs" class="btn-import">📥 Importer paires</button>
        </div>
      </section>
    </div>

    <div v-if="showDeleteConfirm" class="modal-overlay" @click.self="showDeleteConfirm = false">
      <div class="modal">
        <h3>Confirmation</h3>
        <p>{{ deleteMessage }}</p>
        <div class="modal-buttons">
          <button @click="confirmDelete" class="btn-confirm">✅ Confirmer</button>
          <button @click="showDeleteConfirm = false" class="btn-cancel">❌ Annuler</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const calendarsMetadata = ref<any[]>([])
const pairsMetadata = ref<any[]>([])
const calendarFile = ref<HTMLInputElement>()
const pairFile = ref<HTMLInputElement>()
const loadingCalendars = ref(false)
const loadingPairs = ref(false)
const showDeleteConfirm = ref(false)
const deleteMessage = ref('')
const deleteType = ref<'calendar' | 'pair'>('calendar')
const deleteId = ref(0)

onMounted(async () => {
  await loadMetadata()
})

async function loadMetadata() {
  try {
    const calendars = await invoke<any[]>('get_calendars_metadata')
    const pairs = await invoke<any[]>('get_pairs_metadata')
    calendarsMetadata.value = calendars || []
    pairsMetadata.value = pairs || []
  } catch (err) {
    console.error('Erreur chargement métadonnées:', err)
  }
}

async function importCalendars() {
  if (!calendarFile.value?.files?.length) return
  loadingCalendars.value = true
  try {
    await invoke('import_calendar_file', { filePath: calendarFile.value.files[0].name })
    await loadMetadata()
  } catch (err) {
    console.error('Erreur import calendrier:', err)
  } finally {
    loadingCalendars.value = false
  }
}

async function importPairs() {
  if (!pairFile.value?.files?.length) return
  loadingPairs.value = true
  try {
    await invoke('import_pair_file', { filePath: pairFile.value.files[0].name })
    await loadMetadata()
  } catch (err) {
    console.error('Erreur import paires:', err)
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

function deletePair(id: number) {
  deleteType.value = 'pair'
  deleteId.value = id
  deleteMessage.value = 'Supprimer cette paire et toutes ses bougies?'
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  try {
    if (deleteType.value === 'calendar') {
      await invoke('delete_calendar', { calendarId: deleteId.value })
    } else {
      await invoke('delete_pair', { pairId: deleteId.value })
    }
    await loadMetadata()
  } catch (err) {
    console.error('Erreur suppression:', err)
  } finally {
    showDeleteConfirm.value = false
  }
}
</script>

<style scoped>
.import-hub { padding: 30px; }
.sections-container { display: grid; gap: 30px; }
.import-section { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.import-section h3 { color: #e2e8f0; margin-top: 0; }
.info-box { padding: 15px; background: #2d3748; border-radius: 8px; color: #e2e8f0; margin-bottom: 20px; }
.info-box.warning { background: #7f3f1f; color: #fbbf24; }
.table-container { overflow-x: auto; margin-bottom: 20px; }
.data-table { width: 100%; border-collapse: collapse; }
.data-table th { background: #2d3748; padding: 12px; text-align: left; font-weight: 600; color: #e2e8f0; border-bottom: 2px solid #4a5568; }
.data-table td { padding: 12px; border-bottom: 1px solid #2d3748; color: #e2e8f0; }
.file-input-group { display: flex; gap: 10px; align-items: center; }
.file-input-group input { flex: 1; padding: 10px; border: 1px solid #4a5568; border-radius: 6px; background: #2d3748; color: #e2e8f0; }
.btn-import { padding: 10px 20px; background: #1f6feb; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; }
.btn-delete { padding: 6px 12px; background: #dc2626; color: white; border: none; border-radius: 6px; cursor: pointer; }
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal { background: #1a202c; padding: 30px; border-radius: 12px; border: 1px solid #2d3748; max-width: 400px; }
.modal h3 { color: #e2e8f0; }
.modal p { color: #cbd5e0; }
.modal-buttons { display: flex; gap: 10px; margin-top: 20px; }
.btn-confirm { flex: 1; padding: 10px; background: #10b981; color: white; border: none; border-radius: 6px; cursor: pointer; }
.btn-cancel { flex: 1; padding: 10px; background: #6b7280; color: white; border: none; border-radius: 6px; cursor: pointer; }
</style>
