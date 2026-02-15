<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Archive } from '../stores/archiveStore'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])
const fullArchives = ref<Archive[]>([])

onMounted(async () => {
    await loadData()
})

watch(() => props.isOpen, async (newVal) => {
    if (newVal) await loadData()
})

async function loadData() {
    try {
        fullArchives.value = await invoke<Archive[]>('list_archives')
    } catch {
       // Silent
    }
}

function close() {
  emit('close')
}

// Extraction des données Heatmap
interface HeatmapRawData {
  pairs: string[]
  data: Record<string, Record<string, number>> // Event -> Pair -> Score
}

const parsedData = computed(() => {
  return fullArchives.value
    .filter(a => a.archive_type === 'Heatmap' || a.archive_type === 'HEATMAP')
    .map(a => {
      try {
        const data = JSON.parse(a.data_json) as HeatmapRawData
        return {
          id: a.id,
          title: a.title,
          date: a.created_at,
          data: data
        }
      } catch {
        return null
      }
    })
    .filter(d => d !== null) as Array<{
      id: number
      title: string,
      date: string,
      data: HeatmapRawData
    }>
})

// Metrics for Heatmap Meta Analysis

const totalHeatmaps = computed(() => parsedData.value.length)

// Compute aggregates
const aggregates = computed(() => {
  return {} // Placeholder to avoid breaking if referenced elsewhere, though logic is now split
})

const topCorrelatedPairs = computed(() => {
  const pairsAcc: Record<string, { sum: number, count: number }> = {}

  for (const archive of parsedData.value) {
    // Structure expected:
    // 1. Current: archive.data.heatmapData.data
    // 2. Possible: archive.data.data
    // 3. Fallback: archive.data
    const rootData = archive.data
    const heatmapData = rootData.heatmapData || rootData
    const dataMap: Record<string, Record<string, number>> = heatmapData.data || heatmapData

    if (!dataMap || typeof dataMap !== 'object') continue

    for (const [eventName, pairMap] of Object.entries(dataMap)) {
      if (!pairMap || typeof pairMap !== 'object') continue
      
      // Basic validation to check if it looks like a score map (values are numbers)
      const values = Object.values(pairMap)
      if (values.length === 0 || values.some(v => typeof v !== 'number')) continue

      for (const [pairName, score] of Object.entries(pairMap)) {
        if (typeof score !== 'number') continue

        if (!pairsAcc[pairName]) pairsAcc[pairName] = { sum: 0, count: 0 }
        pairsAcc[pairName].sum += score
        pairsAcc[pairName].count++
      }
    }
  }

  return Object.entries(pairsAcc)
    .map(([pair, stats]) => ({
      pair,
      score: Math.round(stats.sum / stats.count)
    }))
    .sort((a, b) => b.score - a.score)
    .slice(0, 5)
})

const topImpactEvents = computed(() => {
  const eventsAcc: Record<string, { sum: number, count: number }> = {}

  for (const archive of parsedData.value) {
    const rootData = archive.data
    const heatmapData = rootData.heatmapData || rootData
    const dataMap: Record<string, Record<string, number>> = heatmapData.data || heatmapData

    if (!dataMap || typeof dataMap !== 'object') continue

    for (const [eventName, pairMap] of Object.entries(dataMap)) {
      if (!pairMap || typeof pairMap !== 'object') continue
      
      // Basic check to ensure it's a map of scores
      const values = Object.values(pairMap)
      if (values.length === 0 || values.some(v => typeof v !== 'number')) continue

      for (const score of Object.values(pairMap)) {
        if (typeof score !== 'number') continue
        
        if (!eventsAcc[eventName]) eventsAcc[eventName] = { sum: 0, count: 0 }
        eventsAcc[eventName].sum += score
        eventsAcc[eventName].count++
      }
    }
  }

  return Object.entries(eventsAcc)
    .map(([name, stats]) => ({
      name,
      impact_score: parseFloat((stats.sum / stats.count / 10).toFixed(1))
    }))
    .sort((a, b) => b.impact_score - a.impact_score)
    .slice(0, 5)
})

const dominantPair = computed(() => {
    if (topCorrelatedPairs.value.length === 0) return 'N/A'
    return topCorrelatedPairs.value[0].pair
})

const keyEvent = computed(() => {
    if (topImpactEvents.value.length === 0) return 'N/A'
    return topImpactEvents.value[0].name
})


function formatDate(d: string) {
    return new Date(d).toLocaleDateString()
}
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>🗺️ Méta-Analyse Heatmaps</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-body">
        <div v-if="parsedData.length === 0" class="empty-state">
          <p>Aucune archive de Heatmap trouvée.</p>
        </div>
        
        <div v-else class="dashboard">
            <!-- Summary metrics -->
            <div class="metrics-row">
                 <div class="metric-card">
                    <span class="label">Total Heatmaps</span>
                    <span class="value">{{ totalHeatmaps }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Paire Dominante</span>
                    <span class="value accent">{{ dominantPair }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Événement Clé</span>
                    <span class="value accent">{{ keyEvent }}</span>
                 </div>
            </div>

            <div class="analysis-grid">
                <!-- Top Pairs -->
                <div class="panel">
                    <h3>Paires les plus corrélées (Global)</h3>
                    <ul class="list-stats">
                        <li v-for="(item, i) in topCorrelatedPairs" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <span class="name">{{ item.pair }}</span>
                            <span class="score">{{ item.score }}%</span>
                        </li>
                    </ul>
                </div>

                <!-- Top Events -->
                <div class="panel">
                    <h3>Événements à Plus Fort Impact</h3>
                     <ul class="list-stats">
                        <li v-for="(item, i) in topImpactEvents" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <span class="name">{{ item.name }}</span>
                            <span class="score">{{ item.impact_score }}/10</span>
                        </li>
                    </ul>
                </div>
            </div>

             <div class="info-note">
                <p>ℹ️ Cette vue agrège les données de toutes vos heatmaps archivées pour identifier les tendances de fond du marché.</p>
                <p class="dev-warning">⚠️ Note: Les métriques détaillées nécessitent que le format des archives Heatmap contienne les scores bruts (mise à jour à venir).</p>
            </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100%; height: 100%;
  background: rgba(0, 0, 0, 0.85); display: flex; justify-content: center; align-items: center; z-index: 1000;
  backdrop-filter: blur(5px);
}
.modal-content {
  background: #151515; width: 85%; max-width: 900px; max-height: 85vh; border-radius: 12px;
  border: 1px solid #333; display: flex; flex-direction: column; overflow: hidden;
}
.modal-header {
  padding: 20px; background: #222; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center;
}
.close-btn { background: none; border: none; font-size: 24px; color: #aaa; cursor: pointer; }
.modal-body { padding: 30px; overflow-y: auto; color: #ddd; }

.metrics-row { display: flex; gap: 20px; margin-bottom: 30px; }
.metric-card { flex: 1; background: #222; padding: 20px; border-radius: 8px; text-align: center; border: 1px solid #333; }
.metric-card .label { display: block; font-size: 0.9em; color: #888; margin-bottom: 5px; }
.metric-card .value { font-size: 2em; font-weight: bold; color: #fff; }
.metric-card .value.accent { color: #4a9eff; }

.analysis-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 30px; margin-bottom: 30px; }
.panel { background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; }
.panel h3 { margin-top: 0; border-bottom: 1px solid #333; padding-bottom: 10px; margin-bottom: 15px; color: #aaa; font-size: 1.1em; }

.list-stats { list-style: none; padding: 0; margin: 0; }
.list-stats li { display: flex; align-items: center; padding: 10px 0; border-bottom: 1px solid #2a2a2a; }
.list-stats li:last-child { border-bottom: none; }
.rank { font-weight: bold; color: #666; width: 30px; }
.name { flex: 1; font-weight: 500; }
.score { font-weight: bold; color: #4a9eff; }

.dev-warning { font-size: 0.85em; color: #e6a23c; margin-top: 5px; }
</style>
