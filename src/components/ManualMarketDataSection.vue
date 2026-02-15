<template>
  <section class="import-section manual-import-section">
    <h3>📂 Import Manuel (CSV Local)</h3>
    <div class="manual-import-box">
      <p class="description">
        Importez vos propres fichiers CSV (format Dukascopy OHLC ou ticks) téléchargés manuellement.
      </p>
      
      <div class="import-actions">
        <button 
          @click="importLocalFiles" 
          class="btn-manual-import"
          :disabled="loading"
          title="Importe des fichiers OHLC M1"
        >
          <span v-if="loading">⏳...</span>
          <span v-else>📁 Importer CSV (OHLC)</span>
        </button>

        <button 
          @click="importTickFiles" 
          class="btn-manual-import btn-tick"
          :disabled="loading"
          title="Importe des fichiers Ticks (Bid/Ask)"
        >
          <span v-if="loading">⏳...</span>
          <span v-else>📊 Importer Ticks</span>
        </button>
      </div>

      <div v-if="lastReport" class="import-report">
        <div class="report-header">
          <span :class="['status-badge', lastReport.failed === 0 ? 'success' : 'warning']">
            {{ lastReport.successful }} / {{ lastReport.total_files }} fichiers importés
          </span>
        </div>
        <ul class="results-list">
          <li v-for="(res, idx) in lastReport.results" :key="idx" :class="res.import_status">
            <span class="file-name">{{ res.file_path.split('/').pop() }}</span>
            <span class="file-status">{{ res.import_message }}</span>
          </li>
        </ul>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

interface ImportReport {
  successful: number
  failed: number
  total_files: number
  results: Array<{
    file_path: string
    import_status: string
    import_message: string
  }>
}

const emit = defineEmits<{
  imported: []
}>()

const loading = ref(false)
const lastReport = ref<ImportReport | null>(null)

async function importLocalFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Fichiers CSV', extensions: ['csv'] }]
    })

    if (!selected) return

    loading.value = true
    lastReport.value = null

    const paths = Array.isArray(selected) ? selected : [selected]
    
    const report = await invoke<ImportReport>('import_and_clean_files', { paths })
    lastReport.value = report

    if (report.successful > 0) {
      emit('imported')
    }
  } catch (err) {
    alert(`Erreur d'import : ${err}`)
  } finally {
    loading.value = false
  }
}

async function importTickFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Fichiers Ticks', extensions: ['csv'] }]
    })

    if (!selected) return

    loading.value = true
    lastReport.value = null

    const paths = Array.isArray(selected) ? selected : [selected]
    let successCount = 0
    const results: Array<{
      file_path: string
      import_status: string
      import_message: string
    }> = []

    for (const path of paths) {
      try {
        const res = await invoke<{ minutes_generated: number }>('import_tick_file', { filePath: path })
        successCount++
        results.push({
          file_path: path,
          import_status: 'success',
          import_message: `Importé (${res.minutes_generated} min)`
        })
      } catch (e) {
        results.push({
          file_path: path,
          import_status: 'failed',
          import_message: String(e)
        })
      }
    }

    lastReport.value = {
      total_files: paths.length,
      successful: successCount,
      failed: paths.length - successCount,
      results
    }

    if (successCount > 0) {
      emit('imported')
    }
  } catch (err) {
    alert(`Erreur d'import : ${err}`)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.manual-import-section {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.manual-import-section h3 {
  color: #e2e8f0;
  margin-top: 0;
  margin-bottom: 12px;
}

.description {
  color: #94a3b8;
  font-size: 0.9rem;
  margin-bottom: 20px;
  line-height: 1.4;
}

.import-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.btn-manual-import {
  flex: 1;
  padding: 12px 10px;
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.85rem;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.btn-manual-import:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.4);
}

.btn-tick {
  background: linear-gradient(135deg, #6366f1, #4f46e5);
}

.btn-tick:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.4);
}

.btn-manual-import:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.import-report {
  margin-top: 20px;
  padding: 12px;
  background: rgba(30, 41, 59, 0.5);
  border-radius: 8px;
  border: 1px solid #334155;
}

.report-header {
  margin-bottom: 10px;
}

.status-badge {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: uppercase;
}

.status-badge.success { background: #065f46; color: #6ee7b7; }
.status-badge.warning { background: #92400e; color: #fcd34d; }

.results-list {
  list-style: none;
  padding: 0;
  margin: 0;
  max-height: 150px;
  overflow-y: auto;
}

.results-list li {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #2d3748;
  font-size: 0.8rem;
}

.results-list li:last-child { border-bottom: none; }

.results-list li.success .file-name { color: #10b981; }
.results-list li.failed .file-name { color: #ef4444; }

.file-status { color: #94a3b8; font-style: italic; }
</style>
