<template>
  <div v-if="loadingHeatmap" class="loading"><span class="spinner">⏳</span><p>Génération de la heatmap...</p></div>

  <div v-if="heatmapData && !loadingHeatmap" class="heatmap-container">
    <HeatmapHeader 
      :current-filter="minVolatilityThreshold"
      :max-events="maxEventsToDisplay"
      @filter-click="minVolatilityThreshold = $event"
      @limit-change="handleLimitChange"
      @reload="handleReloadHeatmap"
      @archive="emit('archive-heatmap')"
      @archive-top-5="archiveTop5Events"
    />
    <!-- Filtres supprimés -->
    <HeatmapTable 
      :pairs="heatmapData.pairs" 
      :sorted-event-types="filteredEventTypes" 
      :min-volatility="minVolatilityThreshold" 
      :get-heatmap-value="getHeatmapValue" 
      :get-heatmap-count="getHeatmapCount"
      :get-heatmap-percentage="getHeatmapPercentage"
      :get-heatmap-class="getHeatmapClass" 
      :get-formatted-event-name="getFormattedEventName"
      :is-archived="isArchived"
      @analyze-cell="(eventName, pair) => emit('analyze-cell', eventName, pair)"
    />

    <div v-if="showArchiveResult" class="modal-overlay">
      <div class="modal-content">
        <h3>Archives créées</h3>
        <p class="modal-message">{{ archiveMessage }}</p>
        <p class="modal-detail">
          <span class="ok">Succès : {{ archiveSummary.success }}</span>
          <span class="fail" v-if="archiveSummary.failed > 0">Erreurs : {{ archiveSummary.failed }}</span>
        </p>
        <button class="btn-primary" @click="closeArchiveResult">Fermer</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useEventCorrelationHeatmap, type HeatmapData } from '../composables/useEventCorrelationHeatmap'
import { useArchiveStore } from '../stores/archiveStore'
import { useConversionStore } from '../stores/conversionStore'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'
import { useRetroAnalysisGraphData } from '../composables/useRetroAnalysisGraphData'
import { eventTranslations } from '../stores/eventTranslations'
import HeatmapHeader from './HeatmapHeader.vue'
import HeatmapTable from './HeatmapTable.vue'

const props = withDefaults(defineProps<{ availablePairs?: string[]; archiveData?: HeatmapData; isArchiveMode?: boolean; calendarId?: number | null }>(), {
  availablePairs: () => [],
  isArchiveMode: false,
  calendarId: null
})

const emit = defineEmits<{
  'archive-heatmap': []
  'analyze-cell': [eventName: string, pair: string]
}>()

const { loadingHeatmap, heatmapData, minVolatilityThreshold, maxEventsToDisplay, sortedEventTypes, loadHeatmapData, forceReloadHeatmap, getHeatmapValue, getHeatmapPercentage, getHeatmapCount, getHeatmapClass, getFormattedEventName, loadArchivedCells, isArchived } = useEventCorrelationHeatmap(props.isArchiveMode, props.archiveData)

const archiveStore = useArchiveStore()
const conversionStore = useConversionStore()

const selectedEventType = ref('')
const showArchiveResult = ref(false)
const archiveSummary = ref({ success: 0, failed: 0 })
const archiveMessage = ref('')

function formatArchiveEventLabel(rawEventName: string): string {
  if (!rawEventName) return 'Événement inconnu'
  const directTranslation = eventTranslations[rawEventName]
  if (directTranslation) {
    return `${rawEventName} (${directTranslation.fr}) ${directTranslation.flag}`
  }

  const trimmed = rawEventName.trim()
  if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
    try {
      const parsed = JSON.parse(trimmed) as { fr?: string; flag?: string }
      const frLabel = parsed.fr || 'Événement'
      const flag = parsed.flag ? ` ${parsed.flag}` : ''
      return `${frLabel}${flag}`.trim()
    } catch {
      // ignore parsing errors and fall back to raw string
    }
  }

  return rawEventName
}

// Fonction exposée pour archiver la heatmap
function getHeatmapArchiveData() {
  return {
    heatmapData: heatmapData.value,
    minVolatilityThreshold: minVolatilityThreshold.value,
    // maxEventsToDisplay removed
    selectedEventType: selectedEventType.value
  }
}

function closeArchiveResult() {
  showArchiveResult.value = false
}

// Exposer la fonction
defineExpose({
  getHeatmapArchiveData,
  handleReloadHeatmap,
  loadArchivedCells
})

// Filter event types based on selected filter
const filteredEventTypes = computed(() => {
  if (!selectedEventType.value) {
    return sortedEventTypes.value
  }
  return sortedEventTypes.value.filter(et => et.name === selectedEventType.value)
})

// Watch availablePairs et charger la heatmap quand elles changent
watch(() => props.availablePairs, (newPairs) => {
  if (newPairs && newPairs.length > 0 && !props.isArchiveMode) {
    loadHeatmapData(newPairs, props.calendarId)
  }
}, { deep: true })

// Synchroniser les cellules archivées quand les archives changent
watch(() => archiveStore.archives.length, () => {
  if (!props.isArchiveMode) {
    loadArchivedCells()
  }
})

// Recharger le heatmap quand les conversions changent (conversions sauvegardées)
watch(() => conversionStore.updateSignal, () => {
  if (!props.isArchiveMode && props.availablePairs && props.availablePairs.length > 0) {
    handleReloadHeatmap()
  }
})

async function handleReloadHeatmap() {
  if (props.availablePairs && props.availablePairs.length > 0) {
    await forceReloadHeatmap(props.availablePairs, props.calendarId)
  }
}

function handleLimitChange(value: number) {
  if (value <= 0) {
    const confirmed = window.confirm('Afficher tous les evenements peut ralentir ou figer la vue. Continuer ?')
    if (!confirmed) return
  }
  maxEventsToDisplay.value = value
}

async function archiveTop5Events() {
  if (!heatmapData.value) return

  await loadArchivedCells()

  // Récupérer les composables pour les analyses
  const { analyzePeakDelay } = useRetrospectiveAnalysis()
  const { analyzeDecayProfile } = useRetrospectiveAnalysis()  
  const { chargerDonnéesGraph, graphData } = useRetroAnalysisGraphData()

  let successCount = 0
  let failedCount = 0

  try {
    // Déterminer les dates
    let periodStart = ''
    let periodEnd = ''
    if (props.calendarId) {
      try {
        const period = await invoke<{ start_date: string | null; end_date: string | null }>(
          'get_calendar_period_by_id',
          { calendarId: props.calendarId }
        )
        periodStart = period.start_date || ''
        periodEnd = period.end_date || ''
      } catch {
        const today = new Date()
        periodEnd = today.toISOString().split('T')[0]
        const thirtyDaysAgo = new Date(today.getTime() - 30 * 24 * 60 * 60 * 1000)
        periodStart = thirtyDaysAgo.toISOString().split('T')[0]
      }
    } else {
      const today = new Date()
      periodEnd = today.toISOString().split('T')[0]
      const thirtyDaysAgo = new Date(today.getTime() - 30 * 24 * 60 * 60 * 1000)
      periodStart = thirtyDaysAgo.toISOString().split('T')[0]
    }
    
    // Pour chaque paire
    for (const pair of heatmapData.value.pairs) {
      // Collecter les événements avec leurs valeurs (exclure les cellules archivées)
      const pairEvents = sortedEventTypes.value.map(eventType => ({
        name: eventType.name,
        value: getHeatmapValue(eventType.name, pair)
      }))
      
      // Top 5 décroissant en ignorant :
      // - cellules archivées (hachurées)
      // - valeurs masquées (val < minVolatilityThreshold ou -1)
      // - échantillons non représentatifs (N < 5)
      const top5 = pairEvents
        .filter(pe => {
          const count = getHeatmapCount(pe.name, pair)
          return (
            pe.value > 0 &&
            pe.value !== -1 &&
            pe.value >= minVolatilityThreshold.value &&
            count >= 5 &&
            !isArchived(pe.name, pair)
          )
        })
        .sort((a, b) => b.value - a.value)
        .slice(0, 5)
      
      // Pour chaque événement du top 5
      for (const eventData of top5) {
        try {
          // Charger les données COMPLÈTES (comme RetroactiveAnalysisView)
          await analyzePeakDelay(pair, eventData.name)
          await analyzeDecayProfile(pair, eventData.name)
          await chargerDonnéesGraph(pair, eventData.name)
          
          // Vérifier que les données existent
          if (!graphData.value) {
            console.warn(`Pas de données graphiques pour ${pair} - ${eventData.name}`)
            failedCount++
            continue
          }
          
          // Construire l'archive EXACTEMENT comme RetroactiveAnalysisView le fait
          const eventLabel = formatArchiveEventLabel(eventData.name)
          const archiveTitle = `📊 Impact de l'événement ${eventLabel} sur la volatilité de ${pair}`
          
          const archiveDataJson = JSON.stringify({
            atrTimelineBefore: graphData.value.atr_timeline_before,
            atrTimelineAfter: graphData.value.atr_timeline_after,
            bodyTimelineBefore: graphData.value.body_timeline_before,
            bodyTimelineAfter: graphData.value.body_timeline_after,
            noiseRatioBefore: graphData.value.noise_ratio_before,
            noiseRatioDuring: graphData.value.noise_ratio_during,
            noiseRatioAfter: graphData.value.noise_ratio_after,
            volatilityIncreasePercent: graphData.value.volatility_increase_percent,
            eventCount: graphData.value.event_count,
            pair,
            eventType: eventData.name,
            eventLabel,
            eventDatetime: graphData.value.event_datetime,
            pointValue: graphData.value.point_value
          })
          
          // Sauvegarder via le store avec le BON type d'archive
          await archiveStore.saveArchive(
            archiveTitle,
            'Correlation de la volatilité Paire/Evenement',
            periodStart,
            periodEnd,
            `Archive automatisée Top 5 pour ${pair}`,
            archiveDataJson
          )
          
          successCount++
        } catch (err) {
          console.error(`Erreur ${pair} - ${eventData.name}:`, err)
          failedCount++
        }
      }
    }
  } catch (err) {
    console.error('Erreur globale Top 5:', err)
  }

  archiveSummary.value = { success: successCount, failed: failedCount }
  archiveMessage.value = failedCount > 0
    ? `✅ ${successCount} archives créées • ❌ ${failedCount} erreurs`
    : `✅ ${successCount} archives créées`
  showArchiveResult.value = true
}

onMounted(() => {
  loadArchivedCells() // Charger les paires/événements archivés
  if (props.isArchiveMode) {
    heatmapData.value = props.archiveData || null
  } else {
    if (!heatmapData.value && props.availablePairs && props.availablePairs.length > 0) {
      loadHeatmapData(props.availablePairs, props.calendarId)
    }
  }
})
</script>

<style scoped>
.loading { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 400px; gap: 20px; }
.spinner { font-size: 60px; animation: hourglassFlip 1s ease-in-out infinite; display: inline-block; }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes hourglassFlip {
  0% { transform: scaleX(1) rotateY(0deg); }
  50% { transform: scaleX(-1) rotateY(180deg); }
  100% { transform: scaleX(1) rotateY(360deg); }
}
.heatmap-container { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; }

.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.65); display: flex; align-items: center; justify-content: center; z-index: 2000; }
.modal-content { background: #0d1117; color: #e6edf3; padding: 24px; border-radius: 12px; border: 1px solid #30363d; min-width: 320px; box-shadow: 0 12px 40px rgba(0,0,0,0.4); text-align: center; display: flex; flex-direction: column; gap: 12px; }
.modal-message { margin: 0; font-weight: 600; }
.modal-detail { margin: 0; display: flex; gap: 12px; justify-content: center; font-size: 0.95em; }
.modal-detail .ok { color: #3fb950; }
.modal-detail .fail { color: #f78166; }
.btn-primary { background: #238636; color: #fff; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 700; cursor: pointer; transition: background 0.2s; }
.btn-primary:hover { background: #2ea043; }
</style>
