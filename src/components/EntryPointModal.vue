<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { EntryAnalysisResult, EntryPointParams } from '../types/entryAnalysis'
import type { AnalysisResult } from '../stores/volatility'
import { calculateVolatilityScore } from '../utils/volatilityScore'
import { getScoreColor } from './metrics/BestSliceCard.helpers'
import EntryCard from './EntryCard.vue'
import MinuteBreakdown from './MinuteBreakdown.vue'
import MovementProfileChart from './MovementProfileChart.vue'
import EntrySummary from './EntrySummary.vue'
import ArchiveModal from './ArchiveModal.vue'

const props = defineProps<{
  isOpen: boolean
  symbol: string
  hour: number
  quarter: number
  analysisResult?: AnalysisResult | null
}>()

const emit = defineEmits<{
  close: []
}>()

const loading = ref(false)
const error = ref<string | null>(null)
const result = ref<EntryAnalysisResult | null>(null)

const quarterStartMinute = () => props.hour * 60 + props.quarter * 15

watch(
  () => props.isOpen,
  async (open) => {
    if (!open || !props.symbol) return
    await analyzeEntryPoint()
  }
)

async function analyzeEntryPoint() {
  loading.value = true
  error.value = null
  result.value = null
  try {
    const params: EntryPointParams = {
      symbol: props.symbol,
      hour: props.hour,
      quarter: props.quarter,
    }
    result.value = await invoke<EntryAnalysisResult>('analyze_entry_points', { params })
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function close() {
  emit('close')
}

// Score de volatilité depuis Stats15Min
const volatilityScore = computed(() => {
  if (!props.analysisResult?.stats_15min) return null
  const stats = props.analysisResult.stats_15min.find(
    s => s.hour === props.hour && s.quarter === props.quarter
  )
  if (!stats) return null
  return Math.round(calculateVolatilityScore(stats))
})

const scoreColor = computed(() => getScoreColor(volatilityScore.value ?? 0))

// Archive
const showArchiveModal = ref(false)

function openArchiveModal() {
  showArchiveModal.value = true
}

const archivePeriodStart = computed(() =>
  props.analysisResult?.period_start || new Date(new Date().getFullYear() - 1, new Date().getMonth(), new Date().getDate()).toISOString()
)
const archivePeriodEnd = computed(() =>
  props.analysisResult?.period_end || new Date().toISOString()
)
const archiveDefaultTitle = computed(() => {
  const h = String(props.hour).padStart(2, '0')
  const m = String(props.quarter * 15).padStart(2, '0')
  return `Analyse ${props.symbol} ${h}:${m}`
})
const archiveDataJson = computed(() =>
  JSON.stringify({
    symbol: props.symbol,
    hour: props.hour,
    quarter: props.quarter,
    entryAnalysis: result.value,
    volatilityScore: volatilityScore.value,
  })
)
</script>

<template>
  <div
    v-if="isOpen"
    class="modal-overlay"
    @click.self="close"
  >
    <div class="modal-content">
      <div class="modal-header">
        <div class="header-title">
          📊 Analyse — {{ symbol }}
          <span class="quarter-label">
            {{ String(hour).padStart(2, '0') }}:{{ String(quarter * 15).padStart(2, '0') }}
          </span>
          <span
            v-if="volatilityScore !== null"
            class="score-badge"
            :style="{ background: scoreColor + '22', borderColor: scoreColor, color: scoreColor }"
          >
            SCORE {{ volatilityScore }}/100
          </span>
        </div>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <div class="modal-body">
        <!-- Loading -->
        <div v-if="loading" class="state-loading">
          <div class="spinner" />
          <p>Analyse du point d'entrée en cours...</p>
        </div>

        <!-- Error -->
        <div v-else-if="error" class="state-error">
          <p>❌ {{ error }}</p>
          <button class="btn-retry" @click="analyzeEntryPoint">Réessayer</button>
        </div>

        <!-- Results -->
        <template v-else-if="result">
          <div class="results-layout">
            <div class="left-col">
              <EntryCard :result="result" />
              <MovementProfileChart
                :details="result.minute_details"
                :peak-minute="result.peak_minute"
                :decay-speed="result.decay_speed"
              />
            </div>
            <div class="right-col">
              <MinuteBreakdown
                :details="result.minute_details"
                :optimal-offset="result.optimal_offset_minutes"
                :quarter-start-minute="quarterStartMinute()"
                :symbol="symbol"
              />
              <EntrySummary :result="result" />
            </div>
          </div>
        </template>
      </div>

      <div class="modal-footer">
        <button v-if="result" class="btn-archive" @click="openArchiveModal">💾 Archiver</button>
        <button class="btn-primary" @click="close">Fermer</button>
      </div>
    </div>
  </div>
  <ArchiveModal
    :show="showArchiveModal"
    archive-type="Analyse Point d'Entrée"
    :period-start="archivePeriodStart"
    :period-end="archivePeriodEnd"
    :symbol="symbol"
    timeframe="M1"
    :data-json="archiveDataJson"
    :default-title="archiveDefaultTitle"
    @close="showArchiveModal = false"
    @saved="showArchiveModal = false"
  />
</template>

<style scoped>
.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; width: 100vw; height: 100vh; background: rgba(0, 0, 0, 0.85); display: flex; align-items: stretch; justify-content: stretch; z-index: 9999; padding: 0; margin: 0; }
.modal-content { background: #0d1117; border: none; border-radius: 0; width: 100%; height: 100%; max-width: none; max-height: none; display: flex; flex-direction: column; overflow: hidden; }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 10px 20px; border-bottom: 1px solid #21262d; background: rgba(13, 17, 23, 0.9); flex-shrink: 0; }
.header-title { font-size: 1.1em; font-weight: 700; color: #e6edf3; }
.quarter-label { margin-left: 10px; padding: 3px 10px; background: #1f6feb; border-radius: 6px; font-size: 0.85em; color: #fff; }
.close-btn { background: none; border: none; color: #8b949e; font-size: 1.2em; cursor: pointer; padding: 4px 8px; border-radius: 6px; transition: all 0.15s; }
.close-btn:hover { background: #21262d; color: #e6edf3; }
.score-badge { margin-left: 12px; padding: 3px 12px; border-radius: 6px; font-size: 0.8em; font-weight: 700; border: 1px solid; letter-spacing: 0.5px; }
.modal-body { flex: 1; overflow-y: auto; padding: 16px 24px; display: flex; flex-direction: column; gap: 12px; }
.results-layout { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; flex: 1; }
.left-col { display: flex; flex-direction: column; gap: 16px; }
.right-col { display: flex; flex-direction: column; gap: 16px; overflow-y: auto; }
.state-loading { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 40px; color: #8b949e; }
.spinner { width: 32px; height: 32px; border: 3px solid #21262d; border-top-color: #1f6feb; border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.state-error { text-align: center; padding: 30px; color: #f85149; }
.btn-retry { margin-top: 12px; padding: 8px 20px; background: #21262d; border: 1px solid #30363d; border-radius: 6px; color: #e6edf3; cursor: pointer; }
.btn-retry:hover { background: #30363d; }
.modal-footer { padding: 8px 20px; border-top: 1px solid #21262d; display: flex; justify-content: flex-end; gap: 12px; flex-shrink: 0; }
.btn-archive { padding: 8px 24px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border: none; border-radius: 8px; color: #fff; font-weight: 600; cursor: pointer; transition: all 0.15s; }
.btn-archive:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }
.btn-primary { padding: 8px 24px; background: #1f6feb; border: none; border-radius: 8px; color: #fff; font-weight: 600; cursor: pointer; transition: background 0.15s; }
.btn-primary:hover { background: #388bfd; }
@media (max-width: 900px) { .results-layout { grid-template-columns: 1fr; } }
</style>
