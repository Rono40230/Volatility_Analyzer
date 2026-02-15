<template>
  <section class="import-section dukascopy-section">
    <h3>📡 Données de marché (Dukascopy)</h3>
    <div class="download-form">
      <div class="form-row">
        <label>Symbole</label>
        <select
          v-model="selectedSymbol"
          class="form-select"
        >
          <option value="">
            — Choisir —
          </option>
          <optgroup
            v-for="cat in categories"
            :key="cat"
            :label="cat"
          >
            <option
              v-for="inst in instrumentsByCategory(cat)"
              :key="inst.id"
              :value="inst.id"
            >
              {{ inst.display }}
            </option>
          </optgroup>
        </select>
      </div>
      <div class="form-row dates">
        <div>
          <label>Du</label>
          <input
            v-model="dateFrom"
            type="date"
            class="form-input"
          >
        </div>
        <div>
          <label>Au</label>
          <input
            v-model="dateTo"
            type="date"
            class="form-input"
          >
        </div>
      </div>
      <button
        class="btn-download"
        :disabled="!canDownload || downloading"
        @click="startDownload"
      >
        <span v-if="downloading">⏳ Téléchargement…</span>
        <span v-else>🚀 Télécharger</span>
      </button>
    </div>
    <div
      v-if="downloading"
      class="progress-section"
    >
      <div class="progress-bar-bg">
        <div
          class="progress-bar-fill"
          :style="{ width: progress.percent + '%' }"
        />
      </div>
      <div class="progress-text">
        {{ progress.percent.toFixed(0) }}% — {{ progress.hours_done }}/{{ progress.hours_total }}h
        ({{ progress.hours_with_data }} avec données)
        <span v-if="progress.current_date"> — {{ progress.current_date }}</span>
      </div>
    </div>
    <div
      v-if="lastResult"
      class="import-result"
    >
      <h4>✅ Téléchargement terminé</h4>
      <div class="result-grid">
        <div
          v-for="item in resultItems"
          :key="item.label"
          class="result-item"
        >
          <span class="label">{{ item.label }}</span>
          <span
            class="value"
            :class="{ highlight: item.highlight }"
          >
            {{ item.value }}
          </span>
        </div>
      </div>
    </div>

    <div
      v-if="errorMessage"
      class="error-message"
    >
      ❌ {{ errorMessage }}
    </div>
    <div
      v-if="pairsMetadata.length > 0"
      class="pairs-list"
    >
      <h4>💱 Paires téléchargées</h4>
      <div class="table-wrapper">
        <table class="data-table">
          <thead>
            <tr>
              <th>Paire</th>
              <th>Bougies</th>
              <th>Période</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="pair in pairsMetadata"
              :key="`${pair.symbol}-${pair.timeframe}`"
            >
              <td><strong>{{ pair.symbol }}</strong></td>
              <td>{{ (pair.candle_count || 0).toLocaleString() }}</td>
              <td>{{ formatPeriod(pair) }}</td>
              <td class="actions">
                <button
                  class="btn-sm btn-update"
                  title="Mettre à jour"
                  @click="updatePair(pair)"
                >
                  🔄
                </button>
                <button
                  class="btn-sm btn-delete"
                  title="Supprimer"
                  @click="$emit('delete', pair)"
                >
                  🗑️
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <div
      v-else
      class="info-box warning"
    >
      Aucune paire téléchargée.
    </div>
  </section>
</template>
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
interface InstrumentInfo { id: string; display: string; category: string }
interface ImportResult { symbol: string; minutes_generated: number; total_ticks: number; date_start: string; date_end: string; avg_spread: number; avg_ticks_per_minute: number }
interface Progress { hours_total: number; hours_done: number; hours_with_data: number; symbol: string; current_date: string; percent: number }
interface PairMeta { symbol: string; timeframe: string; row_count: number; last_updated: string; last_imported_file: string; quality_score: number; candle_count?: number; start_date?: string; end_date?: string; id?: number }
defineProps<{ pairsMetadata: PairMeta[] }>()
const emit = defineEmits<{ delete: [pair: PairMeta]; imported: [] }>()

const instruments = ref<InstrumentInfo[]>([])
const selectedSymbol = ref('')

async function refreshInstruments() {
  try { 
    instruments.value = await invoke<InstrumentInfo[]>('get_dukascopy_instruments') 
  } catch { /* silent */ }
}

defineExpose({ refreshInstruments })

const dateFrom = ref('')
const dateTo = ref('')
const downloading = ref(false)
const lastResult = ref<ImportResult | null>(null)
const errorMessage = ref('')
const progress = ref<Progress>({ hours_total: 0, hours_done: 0, hours_with_data: 0, symbol: '', current_date: '', percent: 0 })
let unlisten: UnlistenFn | null = null

const categories = computed(() => [...new Set(instruments.value.map(i => i.category))])
const canDownload = computed(() => selectedSymbol.value && dateFrom.value && dateTo.value)
const resultItems = computed(() => {
  const r = lastResult.value
  if (!r) return []
  return [
    { label: 'Paire', value: r.symbol },
    { label: 'M1 générées', value: r.minutes_generated.toLocaleString() },
    { label: 'Spread moyen', value: formatSpread(r.avg_spread), highlight: true },
    { label: 'Période', value: `${fmtDate(r.date_start)} → ${fmtDate(r.date_end)}` },
  ]
})
function instrumentsByCategory(cat: string) { return instruments.value.filter(i => i.category === cat) }
onMounted(async () => {
  await refreshInstruments()
  const now = new Date()
  dateTo.value = now.toISOString().substring(0, 10)
  const ago = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
  dateFrom.value = ago.toISOString().substring(0, 10)
  unlisten = await listen<Progress>('dukascopy-progress', (e) => { progress.value = e.payload })
})
onUnmounted(() => { if (unlisten) unlisten() })
async function startDownload() {
  downloading.value = true; errorMessage.value = ''; lastResult.value = null
  progress.value = { hours_total: 0, hours_done: 0, hours_with_data: 0, symbol: '', current_date: '', percent: 0 }
  try {
    lastResult.value = await invoke<ImportResult>('download_dukascopy_data', { symbol: selectedSymbol.value, dateFrom: dateFrom.value, dateTo: dateTo.value })
    emit('imported')
  } catch (err) { errorMessage.value = String(err) } finally { downloading.value = false }
}
function updatePair(pair: PairMeta) {
  selectedSymbol.value = pair.symbol
  if (pair.end_date) {
    const d = new Date(pair.end_date); d.setDate(d.getDate() + 1)
    dateFrom.value = d.toISOString().substring(0, 10)
  }
  dateTo.value = new Date().toISOString().substring(0, 10)
}
function formatSpread(s: number): string { return s < 0.01 ? `${(s * 10000).toFixed(1)} pips` : s.toFixed(2) }
function fmtDate(d: string): string { return d ? d.substring(0, 10) : '—' }
function formatPeriod(p: PairMeta): string {
  if (!p.start_date && !p.end_date) return 'N/A'
  const f = (d?: string | null) => { if (!d) return '?'; try { return new Date(d).toLocaleDateString('fr-FR', { year: 'numeric', month: '2-digit', day: '2-digit' }) } catch { return d.substring(0, 10) } }
  return `${f(p.start_date)} → ${f(p.end_date)}`
}
</script>
<style scoped>
.dukascopy-section { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.dukascopy-section h3 { color: #e2e8f0; margin-top: 0; }
.download-form { display: flex; flex-direction: column; gap: 12px; margin-bottom: 16px; }
.form-row { display: flex; flex-direction: column; gap: 4px; }
.form-row.dates { flex-direction: row; gap: 12px; }
.form-row.dates > div { flex: 1; display: flex; flex-direction: column; gap: 4px; }
.form-row label { color: #94a3b8; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.05em; }
.form-select, .form-input { padding: 8px 12px; background: #1e293b; border: 1px solid #475569; border-radius: 6px; color: #e2e8f0; -webkit-appearance: none; color-scheme: dark; }
.form-select option, .form-select optgroup { background: #1e293b; color: #e2e8f0; }
.btn-download { padding: 10px 20px; background: linear-gradient(135deg, #059669, #10b981); color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; transition: all 0.2s; }
.btn-download:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(16,185,129,0.4); }
.btn-download:disabled { opacity: 0.5; cursor: not-allowed; }
.progress-bar-bg { height: 8px; background: #334155; border-radius: 4px; overflow: hidden; }
.progress-bar-fill { height: 100%; background: linear-gradient(90deg, #10b981, #3b82f6); border-radius: 4px; transition: width 0.3s; }
.progress-text { color: #94a3b8; font-size: 0.8rem; margin-top: 4px; }
.import-result { margin: 12px 0; padding: 12px; background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); border-radius: 10px; }
.result-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
.result-item { display: flex; flex-direction: column; }
.result-item .label { color: #94a3b8; font-size: 0.7rem; text-transform: uppercase; }
.result-item .value { color: #f8fafc; font-size: 0.9rem; }
.result-item .value.highlight { color: #fbbf24; font-weight: 700; }
.error-message { margin: 12px 0; padding: 10px; background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.3); border-radius: 8px; color: #ef4444; }
.pairs-list { margin-top: 20px; }
.pairs-list h4, .data-table th, .data-table td { color: #e2e8f0; }
.table-wrapper { max-height: 400px; overflow-y: auto; border: 1px solid #2d3748; border-radius: 8px; }
.data-table { width: 100%; border-collapse: collapse; }
.data-table th { background: #2d3748; padding: 8px 10px; text-align: left; font-weight: 600; border-bottom: 2px solid #4a5568; }
.data-table td { padding: 8px 10px; border-bottom: 1px solid #2d3748; }
.actions { display: flex; gap: 6px; }
.btn-sm { padding: 4px 8px; border: none; border-radius: 4px; cursor: pointer; }
.btn-update { background: #2563eb; color: white; }
.btn-delete { background: #dc2626; color: white; }
.info-box { padding: 12px; background: #2d3748; border-radius: 8px; color: #e2e8f0; margin-top: 16px; }
</style>
