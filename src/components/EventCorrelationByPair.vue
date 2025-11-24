<template>
  <div v-if="loading" class="loading"><div class="spinner" /><p>Analyse de la corrélation par paire...</p></div>

  <PairSelectorPanel
    v-model="selectedPair"
    :available-pairs="availablePairs"
    :selected-pair="selectedPair"
    :pair-correlation="pairCorrelation"
    :loading="loading"
    :is-archive-mode="isArchiveMode"
    @load="loadPairCorrelation"
    @archive="openArchiveModal"
  />

  <div v-if="pairCorrelation && !loading" class="pair-correlation-results">
    <PairCorrelationTable :top-events="topEvents" :get-score-class="getScoreClass" />

    

    <div class="observations-card">
      <h3>💡 Observations</h3>
      <ul>
        <li v-for="(obs, index) in observations" :key="index">{{ obs }}</li>
        <li v-if="!observations.length">Données insuffisantes pour générer des observations.</li>
      </ul>
    </div>
  </div>

  <ArchiveModal
    :show="showArchiveModal"
    archive-type="Corrélation paire/événement"
    :period-start="archivePeriodStart"
    :period-end="archivePeriodEnd"
    :symbol="selectedPair"
    :timeframe="'M1'"
    :data-json="archiveDataJson"
    @close="showArchiveModal = false"
    @saved="handleArchiveSaved"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useEventCorrelationByPair } from '../composables/useEventCorrelationByPair'
import ArchiveModal from './ArchiveModal.vue'
import PairSelectorPanel from './PairSelectorPanel.vue'
import PairCorrelationTable from './PairCorrelationTable.vue'

const props = withDefaults(defineProps<{ availablePairs?: string[]; archiveData?: any; isArchiveMode?: boolean }>(), {
  availablePairs: () => [],
  isArchiveMode: false
})

const {
  selectedPair,
  pairCorrelation,
  topEvents,
  observations,
  loading,
  loadPairCorrelation,
  getScoreClass
} = useEventCorrelationByPair(props.availablePairs, props.isArchiveMode, props.archiveData)

const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')

function openArchiveModal() {
  if (!pairCorrelation.value) return
  if (pairCorrelation.value.period_start && pairCorrelation.value.period_end) {
    archivePeriodStart.value = pairCorrelation.value.period_start
    archivePeriodEnd.value = pairCorrelation.value.period_end
  } else {
    const now = new Date()
    const oneYearAgo = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
    archivePeriodStart.value = oneYearAgo.toISOString()
    archivePeriodEnd.value = now.toISOString()
  }
  archiveDataJson.value = JSON.stringify({ pairCorrelation: pairCorrelation.value, selectedPair: selectedPair.value })
  showArchiveModal.value = true
}

function handleArchiveSaved() {
  showArchiveModal.value = false
}
</script>

<style scoped>
.loading { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 400px; gap: 20px; }
.spinner { width: 40px; height: 40px; border: 4px solid #30363d; border-top-color: #58a6ff; border-radius: 50%; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.pair-correlation-results { display: flex; flex-direction: column; gap: 20px; padding: 20px 0; }
.observations-card { padding: 20px; background-color: #161b22; border-radius: 8px; border: 1px solid #30363d; }
.observations-card h3 { margin: 0 0 15px 0; color: #c9d1d9; font-size: 16px; }
.observations-card ul { list-style: none; padding: 0; margin: 0; }
.observations-card li { padding: 8px 0; color: #8b949e; font-size: 14px; border-bottom: 1px solid #30363d; }
.observations-card li:last-child { border-bottom: none; }
</style>
