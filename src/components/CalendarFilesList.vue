<template>
  <div class="files-list-section">
    <div class="section-header">
      <h4>📁 Fichiers disponibles</h4>
      <button class="btn-import-header" :disabled="importing" @click="handleImportClick">
        📅 Importer votre calendrier
      </button>
    </div>

    <div v-if="importing" class="importing-overlay">
      <div class="hourglass">⏳</div>
      <p class="importing-text">Import en cours...</p>
    </div>

    <div v-if="importError" class="error-message">❌ {{ importError }}</div>

    <div v-if="loading" class="loading-indicator"><span>⏳ Chargement...</span></div>

    <div v-else-if="error" class="error-message">❌ {{ error }}</div>

    <div v-else-if="files.length === 0" class="no-files-message">📂 Aucun fichier disponible</div>

    <div v-else class="files-table-container">
      <table class="files-table">
        <CalendarTableHeader />
        <tbody>
          <CalendarTableRow
            v-for="file in files"
            :key="file.path"
            :file="file"
            @delete="deleteFile(file.path)"
          />
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import CalendarTableHeader from './CalendarTableHeader.vue'
import CalendarTableRow from './CalendarTableRow.vue'

interface CalendarFileInfo {
  filename: string
  path: string
  size_bytes: number
  created: string
  modified: string
  event_count: number | null
}

const files = ref<CalendarFileInfo[]>([])
const loading = ref(false)
const error = ref('')
const importing = ref(false)
const importError = ref('')

async function refreshFiles() {
  loading.value = true
  error.value = ''
  try {
    const result = await invoke<CalendarFileInfo[]>('list_calendar_files')
    files.value = result
  } catch (e) {
    error.value = `Erreur lors du chargement: ${e}`
  } finally {
    loading.value = false
  }
}

async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Calendrier', extensions: ['csv', 'xlsx', 'xls'] }]
    })
    if (selected && typeof selected === 'string') return selected
  } catch (e) {
    importError.value = `Erreur sélection fichier: ${e}`
  }
  return null
}

async function handleImportClick() {
  const selectedPath = await selectFile()
  if (!selectedPath) return
  importing.value = true
  importError.value = ''
  try {
    await invoke<number>('import_and_convert_calendar', { sourcePath: selectedPath })
    await refreshFiles()
  } catch (e) {
    importError.value = `Échec import: ${e}`
  } finally {
    importing.value = false
  }
}

defineExpose({ refreshFiles })

async function deleteFile(filePath: string) {
  const confirmed = confirm('Êtes-vous sûr de vouloir supprimer ce fichier ?\nCette action est irréversible.')
  if (!confirmed) return
  loading.value = true
  error.value = ''
  try {
    await invoke('delete_calendar_file', { filePath })
    await refreshFiles()
    error.value = ''
  } catch (e) {
    error.value = `Erreur lors de la suppression: ${e}`
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  refreshFiles()
})
</script>

<style scoped>
.files-list-section { background: #0d1117; border-radius: 12px; padding: 20px; margin-bottom: 20px; border: 2px solid #30363d; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }
.section-header h4 { color: #e6edf3; margin: 0; font-size: 1.2em; }
.btn-import-header { background: linear-gradient(180deg, #2ea043 0%, #238636 100%); color: white; border: none; padding: 10px 20px; border-radius: 8px; font-weight: 600; cursor: pointer; transition: all 0.2s; box-shadow: 0 2px 8px rgba(46, 160, 67, 0.2); }
.btn-import-header:hover:not(:disabled) { background: linear-gradient(180deg, #3fb950 0%, #2ea043 100%); transform: translateY(-1px); box-shadow: 0 4px 12px rgba(46, 160, 67, 0.3); }
.btn-import-header:disabled { opacity: 0.5; cursor: not-allowed; }
.importing-overlay { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 15px; margin-bottom: 15px; background: rgba(56, 139, 253, 0.05); border: 1px solid rgba(56, 139, 253, 0.2); border-radius: 8px; }
.hourglass { font-size: 2em; animation: spin 2s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.importing-text { color: #58a6ff; font-weight: 500; margin: 0; }
.loading-indicator, .error-message, .no-files-message { text-align: center; padding: 20px; color: #8b949e; }
.error-message { color: #f97583; }
.files-table-container { overflow-x: auto; }
.files-table { width: 100%; border-collapse: collapse; background: #161b22; border-radius: 8px; overflow: hidden; }
.files-table thead { background: #1c2128; }
.files-table th { padding: 12px; text-align: left; color: #e6edf3; font-weight: 600; border-bottom: 2px solid #30363d; }
.files-table td { padding: 12px; color: #8b949e; border-bottom: 1px solid #30363d; }
.files-table tbody tr:hover { background: #1c2128; }
</style>
