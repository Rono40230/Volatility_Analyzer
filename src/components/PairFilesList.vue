<template>
  <div class="files-list-section">
    <div class="section-header">
      <h4>💱 Paires de Trading Importées</h4>
    </div>

    <div v-if="importing" class="importing-overlay">
      <div class="hourglass">⏳</div>
      <p class="importing-text">Import en cours...</p>
    </div>

    <div v-if="importError" class="error-message">❌ {{ importError }}</div>

    <div v-if="loading" class="loading-indicator"><span>⏳ Chargement...</span></div>

    <div v-else-if="error" class="error-message">❌ {{ error }}</div>

    <div v-else-if="pairs.length === 0" class="no-files-message">
      📂 Aucune donnée de paire importée. Importez vos fichiers CSV pour commencer.
    </div>

    <div v-else class="files-table-container">
      <table class="files-table">
        <PairTableHeader />
        <tbody>
          <PairTableRow
            v-for="pair in pairs"
            :key="`${pair.symbol}-${pair.timeframe}`"
            :pair="pair"
          />
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'
import PairTableHeader from './PairTableHeader.vue'
import PairTableRow from './PairTableRow.vue'

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

const emit = defineEmits<{ filesRefreshed: [] }>()

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

defineExpose({ refreshFiles })

onMounted(() => {
  refreshFiles()
})
</script>

<style scoped>
.files-list-section { background: #0d1117; border-radius: 12px; padding: 20px; margin-bottom: 20px; border: 2px solid #30363d; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }
.section-header h4 { color: #e6edf3; margin: 0; font-size: 1.2em; }
.importing-overlay { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 15px; margin-bottom: 15px; background: rgba(56, 139, 253, 0.05); border: 1px solid rgba(56, 139, 253, 0.2); border-radius: 8px; }
.hourglass { font-size: 2em; animation: spin 2s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.importing-text { color: #58a6ff; font-weight: 500; margin: 0; }
.loading-indicator, .error-message, .no-files-message { text-align: center; padding: 20px; color: #8b949e; }
.error-message { color: #f97583; }
.files-table-container { overflow-x: auto; }
.files-table { width: 100%; border-collapse: collapse; background: #161b22; border-radius: 8px; overflow: hidden; }
.files-table thead { background: #1c2128; }
.files-table td { padding: 12px; color: #8b949e; border-bottom: 1px solid #30363d; }
.files-table tbody tr:hover { background: #1c2128; }
</style>
