<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Archive } from '../stores/archiveStore'
import type { AnalysisResult } from '../stores/volatilityTypes'

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

// Extraction des données Volatilité
const parsedData = computed(() => {
  return fullArchives.value
    .filter(a => a.archive_type === 'Volatilité brute' || a.archive_type === 'Volatilité brute Paire/Période' || a.archive_type === 'Correlation de la volatilité Paire/Evenement' || a.archive_type === 'METRICS')
    .map(a => {
      try {
        const raw = JSON.parse(a.data_json)
        // Les archives récentes encapsulent dans 'analysisResult', les anciennes peuvent l'avoir à la racine
        let data: AnalysisResult | any = raw.analysisResult || raw
        
        // Si on a une structure de retro-analysis, extraire les données
        if (!data.symbol && raw.analysis_result?.symbol) {
          data = raw.analysis_result
        }
        
        // Validation minimale - chercher un symbol partout
        if (!data || (!data.symbol && !a.pair)) return null
        
        // Utiliser le pair de l'archive si pas de symbol dans les données
        if (!data.symbol && a.pair) {
          data = { ...data, symbol: a.pair }
        }

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
    .filter(d => d !== null)
})

// Metrics for Volatility Meta Analysis

const totalArchives = computed(() => parsedData.value.length)

// Top Pairs by Volatility (Mean ATR or Mean Volatility)
const topVolatilePairs = computed(() => {
    const pairStats: Record<string, { sumVol: number, count: number, maxVol: number }> = {}

    for (const item of parsedData.value) {
        if (!item) continue
        const symbol = item.data.symbol
        // Utilisons mean_volatility ou mean_range comme métrique principale
        const vol = item.data.global_metrics?.mean_volatility || 0

        if (!pairStats[symbol]) pairStats[symbol] = { sumVol: 0, count: 0, maxVol: 0 }
        
        pairStats[symbol].sumVol += vol
        pairStats[symbol].count++
        pairStats[symbol].maxVol = Math.max(pairStats[symbol].maxVol, vol)
    }

    return Object.entries(pairStats)
        .map(([pair, stats]) => ({
            pair,
            avgScore: parseFloat((stats.sumVol / stats.count).toFixed(2)),
            maxScore: parseFloat(stats.maxVol.toFixed(2))
        }))
        .sort((a, b) => b.avgScore - a.avgScore)
        .slice(0, 5)
})

// Best Time Slots (Distribution of best_quarter)
const bestTimeSlots = computed(() => {
    const slots: Record<string, number> = {}

    for (const item of parsedData.value) {
        if (!item || !item.data.best_quarter) continue
        const [hour, quarter] = item.data.best_quarter
        
        // Format HH:MM
        const minute = quarter * 15
        const timeStr = `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`
        
        slots[timeStr] = (slots[timeStr] || 0) + 1
    }

    return Object.entries(slots)
        .map(([time, count]) => ({ time, count }))
        .sort((a, b) => b.count - a.count)
        .slice(0, 5)
})

const dominantPair = computed(() => {
    if (topVolatilePairs.value.length === 0) return 'N/A'
    // Retourne la paire la plus volatile en moyenne
    return topVolatilePairs.value[0].pair
})

const mostFrequentTime = computed(() => {
    if (bestTimeSlots.value.length === 0) return 'N/A'
    return bestTimeSlots.value[0].time
})

</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>📊 Méta-Analyse Volatilité</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-body">
        <div v-if="parsedData.length === 0" class="empty-state">
          <p>Aucune archive de Volatilité trouvée.</p>
        </div>

        <div v-else class="dashboard">
            <!-- Summary metrics -->
            <div class="metrics-row">
                 <div class="metric-card">
                    <span class="label">Total Analyses</span>
                    <span class="value">{{ totalArchives }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Paire la plus Volatile</span>
                    <span class="value accent">{{ dominantPair }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Créneau le plus Actif</span>
                    <span class="value accent">{{ mostFrequentTime }}</span>
                 </div>
            </div>

            <div class="analysis-grid">
                <!-- Top Pairs Volatility -->
                <div class="panel">
                    <h3>Paires les plus Volatiles (Moyenne)</h3>
                    <ul class="list-stats">
                        <li v-for="(item, i) in topVolatilePairs" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <span class="name">{{ item.pair }}</span>
                            <span class="score">{{ item.avgScore }} pts</span>
                        </li>
                    </ul>
                </div>

                <!-- Best Time Slots -->
                <div class="panel">
                    <h3>Meilleurs Créneaux Horaires (Récurrence)</h3>
                     <ul class="list-stats">
                        <li v-for="(item, i) in bestTimeSlots" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <span class="name">{{ item.time }} - {{ item.time.split(':')[0] }}:{{ parseInt(item.time.split(':')[1]) + 15 }}</span>
                            <span class="score">{{ item.count }} fois</span>
                        </li>
                    </ul>
                </div>
            </div>

             <div class="info-note">
                <p>ℹ️ Cette vue agrège les métriques de volatilité de toutes vos analyses archivées.</p>
                <p class="dev-warning">Les scores sont basés sur la volatilité moyenne calculée lors de chaque analyse.</p>
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

.dev-warning { font-size: 0.85em; color: #666; margin-top: 5px; font-style: italic; }
</style>
