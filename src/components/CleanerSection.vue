<template>
  <div class="cleaner-section">
    <h4>📥 Import de Données</h4>
    <p class="info-text">
      Importez vos fichiers CSV (formats européen ou international). 
      Le nettoyage et l'import sont effectués automatiquement.
    </p>
    
    <div class="cleaner-controls">
      <button @click="selectFilesToImport" class="btn btn-import-unified" :disabled="importing">
        📥 Importer vos données
      </button>
    </div>

    <!-- Import en cours : sablier tournant -->
    <div v-if="importing" class="importing-overlay">
      <div class="hourglass">⏳</div>
      <p class="importing-text">Import en cours...</p>
    </div>

    <!-- Message d'erreur uniquement -->
    <div v-if="importError" class="error-message">
      ❌ {{ importError }}
    </div>

    <!-- Import Results -->
    <div v-if="importResults.length > 0" class="import-results-list">
      <h5>✅ Fichiers importés ({{ importResults.length }})</h5>
      <div class="result-item" v-for="(result, index) in importResults" :key="index">
        <span v-if="result.import_status === 'success'" class="status-icon">✅</span>
        <span v-else-if="result.import_status === 'partial'" class="status-icon">⚠️</span>
        <span v-else class="status-icon">❌</span>
        <span class="file-name">{{ getFileName(result.original_file) }}</span>
        <span class="file-stats">{{ result.lines_imported }} lignes</span>
        <span v-if="result.cleaning_stats && result.cleaning_stats.errors > 0" class="file-errors">
          {{ result.cleaning_stats.errors }} erreurs nettoyées
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

interface FileCleaningStats {
  lines_processed: number
  lines_cleaned: number
  errors: number
  warnings: string[]
}

interface ImportCleanResult {
  original_file: string
  import_status: string
  lines_imported: number
  cleaning_stats?: FileCleaningStats
  error_message?: string
}

interface ImportCleanReport {
  total_files: number
  successful: number
  failed: number
  results: ImportCleanResult[]
}

const emit = defineEmits<{
  importCompleted: [success: boolean]
  error: [message: string]
}>()

const importing = ref(false)
const importError = ref('')
const importResults = ref<ImportCleanResult[]>([])

function getFileName(path: string): string {
  return path.split('/').pop() || path
}

async function selectFilesToImport() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Fichiers CSV',
        extensions: ['csv']
      }]
    })

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      await importFiles(paths)
    }
  } catch (error) {
    console.error('Erreur sélection fichiers:', error)
    emit('error', `Erreur: ${error}`)
  }
}

async function importFiles(paths: string[]) {
  importing.value = true
  importError.value = ''
  importResults.value = []
  
  try {
    const report = await invoke<ImportCleanReport>('import_and_clean_files', { paths })
    
    importResults.value = report.results
    
    // Afficher les résultats pendant 5 secondes puis les cacher
    setTimeout(() => {
      importResults.value = []
      importing.value = false
    }, 5000)
    
    emit('importCompleted', report.failed === 0)
    
  } catch (error) {
    console.error('Erreur import:', error)
    importError.value = `Erreur lors de l'import : ${error}`
    importing.value = false
    emit('importCompleted', false)
    
    // Cacher le message d'erreur après 10 secondes
    setTimeout(() => {
      importError.value = ''
    }, 10000)
  }
}
</script>

<style scoped>
.cleaner-section {
  background: rgba(56, 139, 253, 0.05);
  border: 1px solid rgba(56, 139, 253, 0.2);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 25px;
}

.cleaner-section h4 {
  color: #58a6ff;
  margin-bottom: 10px;
  font-size: 1.1em;
}

.info-text {
  color: #8b949e;
  font-size: 0.9em;
  margin-bottom: 15px;
  line-height: 1.5;
}

.cleaner-controls {
  margin-bottom: 15px;
}

.btn-import-unified {
  background: linear-gradient(180deg, #2ea043 0%, #238636 100%);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(46, 160, 67, 0.2);
}

.btn-import-unified:hover:not(:disabled) {
  background: linear-gradient(180deg, #3fb950 0%, #2ea043 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(46, 160, 67, 0.3);
}

.btn-import-unified:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Overlay de chargement avec sablier */
.importing-overlay {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  margin: 20px 0;
}

.hourglass {
  font-size: 4em;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.importing-text {
  color: #58a6ff;
  font-size: 1.1em;
  font-weight: 600;
  margin-top: 15px;
}

/* Message d'erreur */
.error-message {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid #f85149;
  border-radius: 8px;
  padding: 15px;
  margin: 15px 0;
  color: #f85149;
  font-weight: 500;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.import-results-list {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  padding: 15px;
  margin-top: 15px;
}

.import-results-list h5 {
  color: #7ee787;
  margin-bottom: 12px;
  font-size: 0.95em;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 6px;
  margin-bottom: 8px;
  font-size: 0.9em;
  flex-wrap: wrap;
}

.status-icon {
  font-size: 1.2em;
}

.file-name {
  flex: 1;
  min-width: 200px;
  color: #c9d1d9;
}

.file-stats {
  color: #7ee787;
  font-weight: 500;
}

.file-errors {
  color: #ffa657;
  font-weight: 500;
  font-size: 0.85em;
}
</style>
