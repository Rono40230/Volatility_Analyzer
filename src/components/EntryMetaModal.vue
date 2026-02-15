<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>🎯 Méta-Analyse Points d'Entrée</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-body">
        <div v-if="archives.length === 0" class="empty-state">
          <p>Aucune archive d'analyse de point d'entrée trouvée.</p>
        </div>
        
        <div v-else class="analysis-dashboard">
          <!-- Header Summary -->
          <div class="summary-header">
            <div class="summary-stat">
              <span class="label">Total Analyses</span>
              <span class="value">{{ totalAnalyses }}</span>
            </div>
            <div class="summary-stat">
              <span class="label">Paires Analysées</span>
              <span class="value">{{ topPairs.length }}</span>
            </div>
          </div>

          <!-- Paires Table -->
          <div class="table-box">
            <h3>📊 Points d'Entrée Optimaux par Paire</h3>
            <table class="data-table">
              <thead>
                <tr>
                  <th>Paire</th>
                  <th>Analyses</th>
                  <th>Pic Moyen (min)</th>
                  <th>Durée Mvmt (min)</th>
                  <th>Mouvement Max (pips)</th>
                  <th>Win Rate Moyen</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="pair in topPairs" :key="pair.pair">
                  <td class="pair-cell"><strong>{{ pair.pair }}</strong></td>
                  <td>{{ pair.analyses.length }}</td>
                  <td>{{ pair.avgPeakMinute }}</td>
                  <td>{{ pair.avgDuration }}</td>
                  <td>{{ pair.maxMovement.toFixed(1) }}</td>
                  <td>
                    <span class="score-badge" :class="pair.avgConsistency >= 70 ? 'high' : pair.avgConsistency >= 50 ? 'medium' : 'low'">
                      {{ pair.avgConsistency }}%
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Archive } from '../stores/archiveStore'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])
const fullArchives = ref<Archive[]>([])
const archives = ref<any[]>([])

onMounted(async () => {
  await loadData()
})

watch(() => props.isOpen, async (newVal) => {
  if (newVal) {
    await loadData()
  }
})

async function loadData() {
  try {
    fullArchives.value = await invoke<Archive[]>('list_archives')
    archives.value = fullArchives.value
      .filter(a => a.archive_type === "Analyse Point d'Entrée")
      .map(a => {
        try {
          const raw = JSON.parse(a.data_json)
          // Structure: { symbol, hour, quarter, entryAnalysis, volatilityScore }
          const entryAnalysis = raw.entryAnalysis
          
          if (!entryAnalysis) return null
          
          return {
            id: a.id,
            date: a.created_at,
            symbol: raw.symbol || entryAnalysis.symbol || 'Unknown',
            pair: a.pair || raw.symbol || entryAnalysis.symbol || 'Unknown',
            hour: raw.hour,
            quarter: raw.quarter,
            entryAnalysis: entryAnalysis
          }
        } catch {
          return null
        }
      })
      .filter(d => d !== null)
  } catch (e) {
    console.error(e)
  }
}

function close() {
  emit('close')
}

// Computed Metrics - Grouped by Pair
const pairStats = computed(() => {
  const stats: Record<string, {
    analyses: any[]
    avgPeakMinute: number
    avgDuration: number
    maxMovement: number
    avgConsistency: number
  }> = {}

  archives.value.forEach(a => {
    const pair = a.pair
    if (!stats[pair]) {
      stats[pair] = {
        analyses: [],
        avgPeakMinute: 0,
        avgDuration: 0,
        maxMovement: 0,
        avgConsistency: 0
      }
    }
    stats[pair].analyses.push(a)
  })

  // Calculer les stats pour chaque paire
  Object.keys(stats).forEach(pair => {
    const analyses = stats[pair].analyses
    stats[pair].avgPeakMinute = Math.round(
      analyses.reduce((sum, a) => sum + (a.entryAnalysis?.peak_minute || 0), 0) / analyses.length
    )
    stats[pair].avgDuration = Math.round(
      analyses.reduce((sum, a) => sum + (a.entryAnalysis?.movement_duration_minutes || 0), 0) / analyses.length
    )
    
    stats[pair].maxMovement = Math.max(
      ...analyses.map(a => a.entryAnalysis?.avg_movement_pips || 0)
    )
    
    // Consistance = moyenne du win_rate (qui est déjà 0-1 ou 0-100)
    const avgWinRate = analyses.reduce((sum, a) => sum + (a.entryAnalysis?.real_win_rate || 0), 0) / analyses.length
    // Si > 1, c'est déjà en %, sinon multiplier par 100
    stats[pair].avgConsistency = Math.round(avgWinRate > 1 ? avgWinRate : avgWinRate * 100)
  })

  return stats
})

const topPairs = computed(() => {
  return Object.entries(pairStats.value)
    .map(([pair, stats]) => ({
      pair,
      ...stats
    }))
    .sort((a, b) => b.avgConsistency - a.avgConsistency)
    .slice(0, 10)
})

const totalAnalyses = computed(() => archives.value.length)

function formatDate(d: string) {
    return new Date(d).toLocaleDateString()
}
</script>

<style scoped>
.modal-overlay {
  position: fixed; top: 0; left: 0; width: 100%; height: 100%;
  background: rgba(0, 0, 0, 0.85); backdrop-filter: blur(5px);
  display: flex; justify-content: center; align-items: center; z-index: 2000;
}
.modal-content {
  background: #111; color: #eee; width: 90%; max-width: 1000px; height: 80vh;
  border-radius: 12px; border: 1px solid #333; display: flex; flex-direction: column;
}
.modal-header {
  padding: 20px; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center;
}
.close-btn { background: none; border: none; font-size: 24px; color: #888; cursor: pointer; }
.modal-body { padding: 30px; overflow-y: auto; }

.summary-header { display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; margin-bottom: 30px; }
.summary-stat { background: #1e1e1e; padding: 15px; border-radius: 8px; text-align: center; border: 1px solid #333; }
.summary-stat .label { display: block; font-size: 0.85em; color: #888; margin-bottom: 8px; }
.summary-stat .value { display: block; font-size: 1.6em; font-weight: bold; color: #4a9eff; }

.table-box { background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; }
.table-box h3 { margin-top: 0; color: #ccc; font-size: 1.1em; margin-bottom: 15px; }

.data-table { width: 100%; border-collapse: collapse; }
.data-table th { text-align: left; color: #888; padding: 12px; border-bottom: 1px solid #333; font-weight: 600; font-size: 0.9em; }
.data-table td { padding: 12px; border-bottom: 1px solid #222; }
.pair-cell { font-weight: bold; color: #4a9eff; }
.score-badge { 
  display: inline-block; padding: 4px 8px; border-radius: 4px; font-weight: bold; font-size: 0.85em;
}
.score-badge.high { background: #22c55e22; color: #22c55e; border: 1px solid #22c55e44; }
.score-badge.medium { background: #f59e0b22; color: #f59e0b; border: 1px solid #f59e0b44; }
.score-badge.low { background: #ef444422; color: #ef4444; border: 1px solid #ef444444; }
</style>