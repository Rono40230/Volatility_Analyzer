<template>
  <div class="files-list-section">
    <div class="section-header">
      <h4>💱 Paires de Trading Importées</h4>
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

    <div v-if="loading" class="loading-indicator">
      <span>⏳ Chargement...</span>
    </div>

    <div v-else-if="error" class="error-message">
      ❌ {{ error }}
    </div>

    <div v-else-if="pairs.length === 0" class="no-files-message">
      📂 Aucune donnée de paire importée. Importez vos fichiers CSV pour commencer.
    </div>

    <div v-else class="files-table-container">
      <table class="files-table">
        <thead>
          <tr>
            <th>Paire</th>
            <th>Timeframe</th>
            <th>Candles</th>
            <th>Dernier Import</th>
            <th>Fichier Source</th>
            <th>Score Qualité</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pair in pairs" :key="`${pair.symbol}-${pair.timeframe}`">
            <td>
              <span class="badge badge-pair">{{ pair.symbol }}</span>
            </td>
            <td>
              <span class="badge badge-timeframe">{{ pair.timeframe }}</span>
            </td>
            <td class="text-right">{{ pair.row_count.toLocaleString() }}</td>
            <td>{{ formatDate(pair.last_updated) }}</td>
            <td class="filename-small">{{ pair.last_imported_file }}</td>
            <td class="text-center">
              <span class="quality-score" :class="`quality-${qualityLevel(pair.quality_score)}`">
                ★ {{ (pair.quality_score * 100).toFixed(0) }}%
              </span>
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
import { useVolatilityStore } from '../stores/volatility'

interface PairMetadataInfo {
  symbol: string
  timeframe: string
  row_count: number
  last_updated: string
  last_imported_file: string
  quality_score: number
}

const volatilityStore = useVolatilityStore()
const pairs = ref<PairMetadataInfo[]>([])
const loading = ref(false)
const error = ref('')
const importing = ref(false)
const importError = ref('')

const emit = defineEmits<{
  filesRefreshed: []
}>()

async function refreshFiles() {
  loading.value = true
  error.value = ''
  
  try {
    const result = await invoke<PairMetadataInfo[]>('get_pair_metadata_from_db')
    pairs.value = result
    emit('filesRefreshed')
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

function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('fr-FR', { year: 'numeric', month: '2-digit', day: '2-digit' })
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

.btn-import-header {
  background: linear-gradient(180deg, #2ea043 0%, #238636 100%);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(46, 160, 67, 0.2);
}

.btn-import-header:hover:not(:disabled) {
  background: linear-gradient(180deg, #3fb950 0%, #2ea043 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(46, 160, 67, 0.3);
}

.btn-import-header:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.importing-overlay {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 15px;
  margin-bottom: 15px;
  background: rgba(56, 139, 253, 0.05);
  border: 1px solid rgba(56, 139, 253, 0.2);
  border-radius: 8px;
}

.hourglass {
  font-size: 2em;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.importing-text {
  color: #58a6ff;
  font-weight: 500;
  margin: 0;
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

.text-right {
  text-align: right;
}

.text-center {
  text-align: center;
}

.filename-small {
  font-size: 0.9em;
  color: #8b949e;
  max-width: 200px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quality-score {
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.9em;
}

.quality-excellent {
  background: #238636;
  color: #ffffff;
}

.quality-good {
  background: #3fb950;
  color: #ffffff;
}

.quality-fair {
  background: #d29922;
  color: #ffffff;
}

.quality-poor {
  background: #da3633;
  color: #ffffff;
}
</style>
