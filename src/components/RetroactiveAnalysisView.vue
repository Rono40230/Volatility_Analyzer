<template>
  <div class="container">
    <RetroAnalysisControls
      :pairs="pairs"
      :selected-pair="store.selectedPair"
      :selected-event-type="store.selectedEventType"
      :min-deviation="store.minDeviation"
      :event-types="eventTypeOptions"
      :event-types-loading="eventTypesLoading"
      :event-types-error="eventTypesError"
      :show-calendar-selector="props.showCalendarSelector"
      @update:selected-pair="store.selectedPair = $event"
      @update:selected-event-type="store.selectedEventType = $event"
      @update:min-deviation="store.minDeviation = $event"
      @calendar-selected="onCalendarSelected"
      @load="load"
    />

    <div v-if="store.loading" class="spinner">Chargement...</div>
    <div v-else-if="store.error" class="error">{{ store.error }}</div>
    <div v-else-if="!store.peakDelayResults || !store.decayResults" class="empty">📭 Chargez une paire et sélectionnez un événement</div>

    <RetroAnalysisResults
      v-else
      :atr-timeline-before="store.graphData?.atr_timeline_before"
      :atr-timeline-after="store.graphData?.atr_timeline_after"
      :body-timeline-before="store.graphData?.body_timeline_before"
      :body-timeline-after="store.graphData?.body_timeline_after"
      :noise-ratio-before="store.graphData?.noise_ratio_before ?? 0"
      :noise-ratio-during="store.graphData?.noise_ratio_during ?? 0"
      :noise-ratio-after="store.graphData?.noise_ratio_after ?? 0"
      :volatility-increase-percent="store.graphData?.volatility_increase_percent ?? 0"
      :event-count="store.graphData?.event_count ?? 0"
      :event-type="store.selectedEventType"
      :pair="store.selectedPair"
      :event-datetime="store.graphData?.event_datetime"
      :timezone-offset="store.graphData?.timezone_offset"
      :meilleur-moment="store.graphData?.meilleur_moment ?? 0"
      :stop-loss="store.graphData?.stop_loss ?? 0"
      :trailing-stop="store.graphData?.trailing_stop ?? 0"
      :timeout="store.graphData?.timeout ?? 60"
      :offset="store.graphData?.offset ?? 0"
      :stop-loss-recovery="store.graphData?.stop_loss_recovery ?? 0"
      :offset-simultaneous="store.graphData?.offset_simultaneous ?? 0"
      :stop-loss-simultaneous="store.graphData?.stop_loss_simultaneous ?? 0"
      :trailing-stop-simultaneous="store.graphData?.trailing_stop_simultaneous ?? 0"
      :stop-loss-recovery-simultaneous="store.graphData?.stop_loss_recovery_simultaneous ?? 0"
      :point-value="store.graphData?.point_value"
      :avg-deviation="store.graphData?.avg_deviation"
      :surprise-event-count="store.graphData?.surprise_event_count"
      :event-label="getEventLabel(store.selectedEventType)"
      @archive="openArchiveModal"
    />

    <ArchiveModal 
      :show="showArchiveModal" 
      archive-type="Correlation de la volatilité Paire/Evenement" 
      :period-start="archivePeriodStart" 
      :period-end="archivePeriodEnd" 
      :symbol="store.selectedPair" 
      :event-name="store.selectedEventType" 
      :event-name-fr="getEventLabel(store.selectedEventType)" 
      :data-json="archiveDataJson" 
      :default-title="archiveDefaultTitle"
      @close="showArchiveModal = false" 
      @saved="handleArchiveSaved" 
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'
import { useRetroAnalysisGraphData } from '../composables/useRetroAnalysisGraphData'
import { useRetroAnalysisStore } from '../stores/retroAnalysisStore'
import { eventTranslations } from '../stores/eventTranslations'
import ArchiveModal from './ArchiveModal.vue'
import RetroAnalysisControls from './RetroAnalysisControls.vue'
import RetroAnalysisResults from './RetroAnalysisResults.vue'

interface SymbolItem { symbol: string; file_path?: string }

const props = defineProps<{ 
  calendarId: number | null
  showCalendarSelector?: boolean
  initialPair?: string
  initialEventType?: string
}>()

const emit = defineEmits<{
  'calendar-selected': [filename: string]
}>()

const store = useRetroAnalysisStore()

// Initialisation depuis les props (mode Modal/Planning)
onMounted(() => {
  if (props.initialPair) {
    store.selectedPair = props.initialPair
  }
  if (props.initialEventType) {
    store.selectedEventType = props.initialEventType
    // Si on a les deux, on lance le chargement automatiquement
    if (props.initialPair) {
      load()
    }
  }
})

const { peakDelayLoading, peakDelayError, peakDelayResults, analyzePeakDelay,
         decayLoading, decayError, decayResults, analyzeDecayProfile,
         eventTypes, eventTypesError, eventTypesLoading, loadEventTypes, getEventLabel } = useRetrospectiveAnalysis()
const { graphData, loading: graphLoading, chargerDonnéesGraph } = useRetroAnalysisGraphData()

const pairs = ref<string[]>([])
const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')
const archiveDefaultTitle = ref('')

const eventTypeOptions = computed(() =>
  eventTypes.value.map(et => ({ name: et.name, label: getEventLabel(et.name), count: et.count }))
)

const onCalendarSelected = (filename: string) => { emit('calendar-selected', filename) }

onMounted(async () => {
  try {
    const s = await invoke<SymbolItem[]>('load_symbols')
    if (s && s.length > 0) {
      pairs.value = s.map((x: SymbolItem) => x.symbol)
    } else {
      pairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
    }
  } catch (e) {
    pairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
  }
  await loadEventTypes(props.calendarId ?? undefined)
})

async function load() {
  if (!store.selectedPair || !store.selectedEventType) return
  store.loading = true
  store.error = null
  try {
    await analyzePeakDelay(store.selectedPair, store.selectedEventType)
    await analyzeDecayProfile(store.selectedPair, store.selectedEventType)
    await chargerDonnéesGraph(store.selectedPair, store.selectedEventType, store.minDeviation)
    
    // Sync to store
    store.peakDelayResults = peakDelayResults.value
    store.decayResults = decayResults.value
    store.graphData = graphData.value
  } catch (e) {
    store.error = String(e)
  } finally {
    store.loading = false
  }
}

function openArchiveModal() {
  if (!store.graphData || !store.selectedPair || !store.selectedEventType) return
  
  // Utiliser les dates min/max de l'analyse si disponibles, sinon fallback
  if (store.peakDelayResults?.event_date_min && store.peakDelayResults?.event_date_max) {
    archivePeriodStart.value = store.peakDelayResults.event_date_min
    archivePeriodEnd.value = store.peakDelayResults.event_date_max
  } else {
    // Fallback si pas de dates (ne devrait pas arriver avec une analyse valide)
    archivePeriodStart.value = new Date().toISOString()
    archivePeriodEnd.value = new Date().toISOString()
  }

  // Construction du titre personnalisé
  const eventName = store.selectedEventType
  const eventNameFr = getEventLabel(eventName)
  
  archiveDefaultTitle.value = `📊 Impact de l'événement ${eventNameFr} sur la volatilité de ${store.selectedPair}`

  archiveDataJson.value = JSON.stringify({
    atrTimelineBefore: store.graphData.atr_timeline_before,
    atrTimelineAfter: store.graphData.atr_timeline_after,
    bodyTimelineBefore: store.graphData.body_timeline_before,
    bodyTimelineAfter: store.graphData.body_timeline_after,
    noiseRatioBefore: store.graphData.noise_ratio_before,
    noiseRatioDuring: store.graphData.noise_ratio_during,
    noiseRatioAfter: store.graphData.noise_ratio_after,
    volatilityIncreasePercent: store.graphData.volatility_increase_percent,
    eventCount: store.graphData.event_count,
    pair: store.selectedPair,
    eventType: store.selectedEventType,
    eventLabel: getEventLabel(store.selectedEventType),
    eventDatetime: store.graphData.event_datetime,
    meilleurMoment: store.graphData.meilleur_moment,
    stopLoss: store.graphData.stop_loss,
    trailingStop: store.graphData.trailing_stop,
    timeout: store.graphData.timeout,
    offset: store.graphData.offset,
    stopLossRecovery: store.graphData.stop_loss_recovery,
    stopLossSimultaneous: store.graphData.stop_loss_simultaneous,
    trailingStopSimultaneous: store.graphData.trailing_stop_simultaneous,
    offsetSimultaneous: store.graphData.offset_simultaneous,
    stopLossRecoverySimultaneous: store.graphData.stop_loss_recovery_simultaneous,
    pointValue: store.graphData.point_value
  })
  showArchiveModal.value = true
}

function handleArchiveSaved() { showArchiveModal.value = false }
</script>

<style scoped>
.container { min-height: 100%; padding: 12px 20px 20px 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; display: flex; flex-direction: column; overflow: auto; }
.spinner { text-align: center; color: #8b949e; padding: 20px; font-size: 1.1em; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 15px; }
.spinner::before { content: '⏳'; font-size: 50px; animation: flip 1s ease-in-out infinite; display: block; order: -1; }
.empty { text-align: center; color: #8b949e; padding: 20px; font-size: 1.1em; display: flex; align-items: center; justify-content: center; }
.error { background: #3d2626; color: #f85149; padding: 15px; border-radius: 8px; margin-bottom: 20px; flex-shrink: 0; }
@keyframes flip { 0% { transform: scaleX(1) rotateY(0deg); } 50% { transform: scaleX(-1) rotateY(180deg); } 100% { transform: scaleX(1) rotateY(360deg); } }
@media (max-width: 768px) { .container { padding: 10px 15px 15px 15px; } }
@media (max-width: 480px) { .container { padding: 8px 10px 10px 10px; } .spinner { padding: 20px 10px; font-size: 1em; } }
</style>
