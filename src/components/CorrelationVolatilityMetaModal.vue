<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Archive } from '../stores/archiveStore'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])
const fullArchives = ref<Archive[]>([])

type CorrelationArchiveEntry = {
  id: number
  title: string
  date?: string
  data: Record<string, unknown> & {
    volatilityIncreasePercent?: number
    noiseRatioDuring?: number
    eventCount?: number
    atrTimelineBefore?: number[]
    pointValue?: number
  }
}

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
 
  function formatEventLabel(rawLabel: unknown, fallback?: string): string {
    if (!rawLabel) return fallback || 'Événement inconnu'
    if (typeof rawLabel === 'string') return rawLabel
    if (typeof rawLabel === 'object') {
      const translation = rawLabel as { fr?: string; flag?: string }
      const base = fallback || 'Événement'
      const frPart = translation.fr ? ` (${translation.fr})` : ''
      const flagPart = translation.flag ? ` ${translation.flag}` : ''
      return `${base}${frPart}${flagPart}`.trim()
    }
    return String(rawLabel)
  }

const toNumber = (value: unknown): number | null => {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  return null
}

const computeContextVolPercent = (entry: CorrelationArchiveEntry): number | null => {
  const atrTimeline = Array.isArray(entry.data.atrTimelineBefore) ? entry.data.atrTimelineBefore : null
  const pointValue = typeof entry.data.pointValue === 'number' && entry.data.pointValue > 0 ? entry.data.pointValue : null
  if (!atrTimeline?.length || !pointValue) return null

  const total = atrTimeline.reduce((sum, value) => (typeof value === 'number' ? sum + value : sum), 0)
  const mean = total / atrTimeline.length
  if (!Number.isFinite(mean)) return null
  return (mean / pointValue) / 100
}

// Extraction des données Correlation de Volatilité
const isCorrelationArchive = (entry: CorrelationArchiveEntry | null): entry is CorrelationArchiveEntry => entry !== null

const parsedData = computed(() => {
  return fullArchives.value
    .filter(a => a.archive_type === 'Correlation de la volatilité Paire/Evenement')
    .map<CorrelationArchiveEntry | null>(a => {
      try {
        const data = JSON.parse(a.data_json)
        if (!data) return null

        return {
          id: a.id,
          title: a.title,
          date: a.created_at,
          data
        }
      } catch {
        return null
      }
    })
    .filter(isCorrelationArchive)
})

// Metrics for Correlation Volatility Meta Analysis

const totalAnalyses = computed(() => parsedData.value.length)

// Top Events by Impact
const topImpactfulEvents = computed(() => {
    const eventStats: Record<string, { sumImpact: number, count: number, maxImpact: number, totalOccurrences: number }> = {}

    for (const item of parsedData.value) {
        if (!item) continue
          const eventLabel = formatEventLabel(item.data.eventLabel, item.data.eventType)
        const impact = item.data.volatilityIncreasePercent || 0
        const occurrences = item.data.eventCount || 0

        if (!eventStats[eventLabel]) {
            eventStats[eventLabel] = { sumImpact: 0, count: 0, maxImpact: 0, totalOccurrences: 0 }
        }
        
        eventStats[eventLabel].sumImpact += impact
        eventStats[eventLabel].count++
        eventStats[eventLabel].maxImpact = Math.max(eventStats[eventLabel].maxImpact, impact)
        eventStats[eventLabel].totalOccurrences += occurrences
    }

    return Object.entries(eventStats)
        .map(([event, stats]) => ({
            event,
            avgImpact: parseFloat((stats.sumImpact / stats.count).toFixed(2)),
            maxImpact: parseFloat(stats.maxImpact.toFixed(2)),
            totalOccurrences: stats.totalOccurrences
        }))
        .sort((a, b) => b.avgImpact - a.avgImpact)
        .slice(0, 5)
})

// Top Pairs by Volatility
const topAffectedPairs = computed(() => {
    const pairStats: Record<string, { sumImpact: number, count: number, maxImpact: number }> = {}

    for (const item of parsedData.value) {
        if (!item) continue
        const pair = item.data.pair || 'Unknown'
        const impact = item.data.volatilityIncreasePercent || 0

        if (!pairStats[pair]) {
            pairStats[pair] = { sumImpact: 0, count: 0, maxImpact: 0 }
        }
        
        pairStats[pair].sumImpact += impact
        pairStats[pair].count++
        pairStats[pair].maxImpact = Math.max(pairStats[pair].maxImpact, impact)
    }

    return Object.entries(pairStats)
        .map(([pair, stats]) => ({
            pair,
            avgImpact: parseFloat((stats.sumImpact / stats.count).toFixed(2)),
            maxImpact: parseFloat(stats.maxImpact.toFixed(2))
        }))
        .sort((a, b) => b.avgImpact - a.avgImpact)
        .slice(0, 5)
})

const mostImpactfulEvent = computed(() => {
    if (topImpactfulEvents.value.length === 0) return 'N/A'
    return topImpactfulEvents.value[0].event
})

const mostAffectedPair = computed(() => {
    if (topAffectedPairs.value.length === 0) return 'N/A'
    return topAffectedPairs.value[0].pair
})

const averageImpact = computed(() => {
    if (parsedData.value.length === 0) return 0
    const total = parsedData.value.reduce((sum, item) => sum + (item.data.volatilityIncreasePercent || 0), 0)
    return parseFloat((total / parsedData.value.length).toFixed(2))
})

const impactStdDeviation = computed(() => {
  if (parsedData.value.length === 0) return 0
  const mean = averageImpact.value
  const variance = parsedData.value.reduce((sum, item) => {
    const impact = item.data.volatilityIncreasePercent || 0
    return sum + Math.pow(impact - mean, 2)
  }, 0) / parsedData.value.length
  return parseFloat(Math.sqrt(variance).toFixed(2))
})

const consistencyLabel = computed(() => {
  const std = impactStdDeviation.value
  if (std < 10) return 'Impact prévisible'
  if (std < 20) return 'Impact variable'
  return 'Impact erratique'
})

const noiseStats = computed(() => {
  if (parsedData.value.length === 0) return { highShare: 0, avgNoise: 0 }
  let noisySessions = 0
  let totalNoise = 0
  let considered = 0
  parsedData.value.forEach(entry => {
    const noise = toNumber(entry.data.noiseRatioDuring)
    if (noise === null) return
    considered++
    totalNoise += noise
    if (noise > 3) noisySessions++
  })

  if (considered === 0) return { highShare: 0, avgNoise: 0 }
  return {
    highShare: parseFloat(((noisySessions / considered) * 100).toFixed(1)),
    avgNoise: parseFloat((totalNoise / considered).toFixed(2))
  }
})

const highNoiseShare = computed(() => noiseStats.value.highShare)
const avgNoiseDuring = computed(() => noiseStats.value.avgNoise)

const contextVolatilityStats = computed(() => {
  let count = 0
  let contextSum = 0
  let alphaSum = 0

  parsedData.value.forEach(entry => {
    const contextVol = computeContextVolPercent(entry)
    if (contextVol === null) return
    const impact = entry.data.volatilityIncreasePercent || 0
    count++
    contextSum += contextVol
    alphaSum += impact - contextVol
  })

  if (count === 0) {
    return { avgContext: 0, avgAlpha: 0 }
  }

  return {
    avgContext: parseFloat((contextSum / count).toFixed(2)),
    avgAlpha: parseFloat((alphaSum / count).toFixed(2))
  }
})

const avgContextVolatility = computed(() => contextVolatilityStats.value.avgContext)
const avgAlphaImpact = computed(() => contextVolatilityStats.value.avgAlpha)

const confidenceScore = computed(() => {
  if (parsedData.value.length === 0) return 0
  const maxOccurrences = parsedData.value.reduce((max, entry) => Math.max(max, entry.data.eventCount || 0), 0)
  if (maxOccurrences === 0) return 0

  const aggregate = parsedData.value.reduce((sum, entry) => {
    const occurrences = entry.data.eventCount || 0
    const noise = toNumber(entry.data.noiseRatioDuring) ?? 0
    const occurrenceFactor = occurrences / maxOccurrences
    const noisePenalty = 1 - Math.min(Math.max(noise / 3.5, 0), 1)
    const score = (occurrenceFactor * 0.6) + (noisePenalty * 0.4)
    return sum + score
  }, 0)

  return Math.round((aggregate / parsedData.value.length) * 100)
})

const confidenceLabel = computed(() => {
  const score = confidenceScore.value
  if (score >= 75) return 'Setup solide'
  if (score >= 50) return 'À surveiller'
  return 'Peu fiable'
})

</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>📈 Méta-Analyse Corrélation Volatilité</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-body">
        <div v-if="parsedData.length === 0" class="empty-state">
          <p>Aucune archive de Corrélation de Volatilité trouvée.</p>
        </div>

        <div v-else class="dashboard">
            <!-- Summary metrics -->
            <div class="metrics-row">
                 <div class="metric-card">
                    <span class="label">Total Analyses</span>
                    <span class="value">{{ totalAnalyses }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Impact Moyen</span>
                    <span class="value accent">{{ averageImpact }}%</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Événement Majeur</span>
                    <span class="value accent">{{ mostImpactfulEvent }}</span>
                 </div>
                 <div class="metric-card">
                    <span class="label">Paire la plus Affectée</span>
                    <span class="value accent">{{ mostAffectedPair }}</span>
                 </div>
            </div>

            <div class="metrics-row secondary">
                <div class="metric-card">
                  <span class="label">Volatilité de l'impact</span>
                  <span class="value accent">{{ impactStdDeviation }}%</span>
                  <span class="hint">{{ consistencyLabel }}</span>
                </div>
                <div class="metric-card">
                  <span class="label">Sessions à bruit &gt; 3</span>
                  <span class="value accent">{{ highNoiseShare }}%</span>
                  <span class="hint">Noise moyen: {{ avgNoiseDuring }}</span>
                </div>
                <div class="metric-card">
                  <span class="label">Alpha vs volatilité locale</span>
                  <span class="value" :class="{ accent: avgAlphaImpact >= 0, negative: avgAlphaImpact < 0 }">
                    {{ avgAlphaImpact >= 0 ? '+' : '' }}{{ avgAlphaImpact }}%
                  </span>
                  <span class="hint">Volatilité horaire: {{ avgContextVolatility }}%</span>
                </div>
                <div class="metric-card">
                  <span class="label">Score de confiance</span>
                  <span class="value accent">{{ confidenceScore }}%</span>
                  <span class="hint">{{ confidenceLabel }}</span>
                </div>
            </div>

            <div class="analysis-grid">
                <!-- Top Events by Impact -->
                <div class="panel">
                    <h3>Événements les Plus Impactants</h3>
                    <ul class="list-stats">
                        <li v-for="(item, i) in topImpactfulEvents" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <div class="name-col">
                              <span class="name">{{ item.event }}</span>
                              <span class="sub-text">{{ item.totalOccurrences }} occurrence{{ item.totalOccurrences > 1 ? 's' : '' }}</span>
                            </div>
                            <span class="score">{{ item.avgImpact }}%</span>
                        </li>
                    </ul>
                </div>

                <!-- Top Pairs Affected -->
                <div class="panel">
                    <h3>Paires les Plus Affectées</h3>
                     <ul class="list-stats">
                        <li v-for="(item, i) in topAffectedPairs" :key="i">
                            <span class="rank">#{{ i+1 }}</span>
                            <span class="name">{{ item.pair }}</span>
                            <span class="score">{{ item.avgImpact }}%</span>
                        </li>
                    </ul>
                </div>
            </div>

             <div class="info-note">
                <p>ℹ️ Cette vue agrège l'analyse de corrélation entre événements économiques et volatilité de vos paires.</p>
                <p class="dev-warning">Les impacts affichés représentent l'augmentation de volatilité moyenne suite à ces événements.</p>
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
  background: #151515;
  width: min(1100px, 94vw);
  max-height: 90vh;
  border-radius: 18px;
  border: 1px solid #333;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 25px 80px rgba(0, 0, 0, 0.65);
}
.modal-header {
  padding: 20px; background: #222; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center;
}
.close-btn { background: none; border: none; font-size: 24px; color: #aaa; cursor: pointer; }
.modal-body { padding: 36px; overflow-y: auto; color: #ddd; }

.metrics-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 15px;
  margin-bottom: 24px;
}
.metrics-row.secondary {
  margin-top: -6px;
  margin-bottom: 32px;
}
.metric-card { background: #222; padding: 15px; border-radius: 8px; text-align: center; border: 1px solid #333; }
.metric-card .label { display: block; font-size: 0.8em; color: #888; margin-bottom: 8px; }
.metric-card .value { font-size: 1.5em; font-weight: bold; color: #fff; }
.metric-card .value.accent { color: #4a9eff; }
.metric-card .value.negative { color: #f87171; }
.metric-card .hint { display: block; margin-top: 6px; font-size: 0.75em; color: #7d7d7d; }

.analysis-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 30px; margin-bottom: 30px; }
.panel { background: #1e1e1e; padding: 20px; border-radius: 8px; border: 1px solid #333; }
.panel h3 { margin-top: 0; border-bottom: 1px solid #333; padding-bottom: 10px; margin-bottom: 15px; color: #aaa; font-size: 1.1em; }

.list-stats { list-style: none; padding: 0; margin: 0; }
.list-stats li { display: flex; align-items: center; padding: 12px 0; border-bottom: 1px solid #2a2a2a; gap: 12px; }
.list-stats li:last-child { border-bottom: none; }
.rank { font-weight: bold; color: #666; min-width: 30px; }
.name-col { flex: 1; display: flex; flex-direction: column; gap: 3px; }
.name { font-weight: 500; }
.sub-text { font-size: 0.8em; color: #666; }
.score { font-weight: bold; color: #4a9eff; min-width: 60px; text-align: right; }

.dev-warning { font-size: 0.85em; color: #666; margin-top: 5px; font-style: italic; }
</style>
