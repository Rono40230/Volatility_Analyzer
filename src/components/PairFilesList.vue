<template>
  <div class="files-list-section">
    <div class="section-header">
      <h4>📁 Fichiers CSV disponibles</h4>
    </div>

    <div v-if="loading" class="loading-indicator">
      <span>⏳ Chargement...</span>
    </div>

    <div v-else-if="error" class="error-message">
      ❌ {{ error }}
    </div>

    <div v-else-if="files.length === 0" class="no-files-message">
      📂 Aucun fichier CSV disponible
    </div>

    <div v-else class="files-table-container">
      <table class="files-table">
        <thead>
          <tr>
            <th>Fichier</th>
            <th>Paire</th>
            <th>Timeframe</th>
            <th>Période</th>
            <th>Lignes</th>
            <th>Taille</th>
            <th>Modifié</th>
            <th class="actions-col">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="file in files" :key="file.path">
            <td>
              <div class="filename-content">
                <span class="file-icon">📊</span>
                <span>{{ file.filename }}</span>
              </div>
            </td>
            <td>
              <span class="badge badge-pair">{{ file.pair || 'N/A' }}</span>
            </td>
            <td>
              <span class="badge badge-timeframe">{{ file.timeframe || 'N/A' }}</span>
            </td>
            <td>{{ file.period || 'N/A' }}</td>
            <td>{{ file.line_count ? file.line_count.toLocaleString() : 'N/A' }}</td>
            <td>{{ formatSize(file.size_bytes) }}</td>
            <td>{{ file.modified }}</td>
            <td class="actions-col">
              <button @click="deleteFile(file.path)" class="btn-delete" title="Supprimer ce fichier">
                🗑️
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface PairFileInfo {
  filename: string
  path: string
  pair: string | null
  timeframe: string | null
  period: string | null
  size_bytes: number
  line_count: number | null
  date_range: string | null
  created: string
  modified: string
}

const files = ref<PairFileInfo[]>([])
const loading = ref(false)
const error = ref('')

async function refreshFiles() {
  loading.value = true
  error.value = ''
  
  try {
    const result = await invoke<PairFileInfo[]>('list_pair_csv_files')
    files.value = result
  } catch (e) {
    error.value = `Erreur lors du chargement: ${e}`
  } finally {
    loading.value = false
  }
}

// Exposer la fonction pour les composants parents
defineExpose({
  refreshFiles
})

async function deleteFile(filePath: string) {
  const confirmed = confirm(
    `Êtes-vous sûr de vouloir supprimer ce fichier CSV ?\nCette action est irréversible.`
  )
  
  if (!confirmed) return
  
  loading.value = true
  error.value = ''
  
  try {
    await invoke<number>('delete_pair_files', { filePaths: [filePath] })
    
    // Rafraîchir la liste
    await refreshFiles()
    alert(`✅ Fichier supprimé avec succès`)
  } catch (e) {
    error.value = `Erreur lors de la suppression: ${e}`
  } finally {
    loading.value = false
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(() => {
  refreshFiles()
})
</script>

<style scoped>
.files-list-section {
  background: #0d1117;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  border: 2px solid #30363d;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.section-header h4 {
  color: #e6edf3;
  margin: 0;
  font-size: 1.2em;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.loading-indicator,
.error-message,
.no-files-message {
  text-align: center;
  padding: 20px;
  color: #8b949e;
}

.error-message {
  color: #f97583;
}

.files-table-container {
  overflow-x: auto;
}

.files-table {
  width: 100%;
  border-collapse: collapse;
  background: #161b22;
  border-radius: 8px;
  overflow: hidden;
}

.files-table thead {
  background: #1c2128;
}

.files-table th {
  padding: 12px;
  text-align: left;
  color: #e6edf3;
  font-weight: 600;
  border-bottom: 2px solid #30363d;
  white-space: nowrap;
}

.files-table td {
  padding: 12px;
  color: #8b949e;
  border-bottom: 1px solid #30363d;
}

.files-table tbody tr:hover {
  background: #1c2128;
}

.filename-content {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e6edf3;
  font-weight: 500;
}

.file-icon {
  font-size: 1.2em;
}

.badge {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.85em;
  font-weight: 600;
}

.badge-pair {
  background: linear-gradient(135deg, #1f6feb, #388bfd);
  color: white;
}

.badge-timeframe {
  background: linear-gradient(135deg, #238636, #2ea043);
  color: white;
}

.date-range-col {
  font-family: 'Courier New', monospace;
  font-size: 0.9em;
}

.actions-col {
  width: 80px;
  text-align: center;
}

.btn-delete {
  padding: 6px 12px;
  background: #da3633;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1.1em;
  transition: all 0.2s;
}

.btn-delete:hover {
  background: #f85149;
  transform: scale(1.1);
}

.btn-delete:active {
  transform: scale(0.95);
}
</style>
