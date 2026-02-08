<template>
  <div class="container">
    <RetroAnalysisControls
      :pairs="pairs"
      :selected-pair="store.selectedPair"
      :selected-event-type="store.selectedEventType"
      :event-types="eventTypeOptions"
      :event-types-loading="eventTypesLoading"
      :event-types-error="eventTypesError"
      :show-calendar-selector="props.showCalendarSelector"
      @update:selected-pair="store.selectedPair = $event"
      @update:selected-event-type="store.selectedEventType = $event"
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
      :timeout="store.graphData?.timeout ?? 60"
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
import { ref, onMounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'
import { useRetroAnalysisGraphData } from '../composables/useRetroAnalysisGraphData'
import { useEventTypeResolver } from '../composables/useEventTypeResolver'
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
  debugLog?: (message: string) => void
}>()

const emit = defineEmits<{
  'calendar-selected': [filename: string]
}>()

const store = useRetroAnalysisStore()

function logDebug(message: string) {
  if (props.debugLog) props.debugLog(message)
}

// Initialisation depuis les props (mode Modal/Planning)
onMounted(() => {
  if (props.initialPair) {
    store.selectedPair = props.initialPair
    logDebug(`Init pair = ${props.initialPair}`)
  }
  if (props.initialEventType) {
    store.selectedEventType = props.initialEventType
    logDebug(`Init event = ${props.initialEventType}`)
    // Si on a les deux, on lance le chargement automatiquement
    if (props.initialPair) {
      logDebug('Init load()')
      load(props.initialPair, props.initialEventType)
    }
  }
})

watch(
  () => [props.initialPair, props.initialEventType],
  ([pair, eventType]) => {
    if (!pair || !eventType) return
    if (store.selectedPair !== pair) store.selectedPair = pair
    if (store.selectedEventType !== eventType) store.selectedEventType = eventType
    logDebug(`Watch load ${pair} / ${eventType}`)
    load(pair, eventType)
  }
)

const { peakDelayLoading, peakDelayError, peakDelayResults, analyzePeakDelay,
         decayLoading, decayError, decayResults, analyzeDecayProfile,
         eventTypes, eventTypesError, eventTypesLoading, loadEventTypes, getEventLabel } = useRetrospectiveAnalysis()
const { resolveEventType } = useEventTypeResolver(eventTypes, getEventLabel)
const { graphData, loading: graphLoading, chargerDonnéesGraph } = useRetroAnalysisGraphData()

const pairs = ref<string[]>([])
const pendingPair = ref<string | null>(null)
const pendingEvent = ref<string | null>(null)
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
  if (eventTypes.value.length > 0) {
    logDebug(`Event types sample: ${eventTypes.value.slice(0, 3).map(e => e.name).join(' | ')}`)
  }
  if (pendingPair.value && pendingEvent.value) {
    const resolved = resolveEventType(pendingEvent.value)
    if (resolved) {
      logDebug(`Resolve event -> ${resolved}`)
      triggerLoad(pendingPair.value, resolved)
      pendingPair.value = null
      pendingEvent.value = null
    } else {
      logDebug(`Resolve event failed: ${pendingEvent.value}`)
    }
  }
})

watch(eventTypes, () => {
  if (!pendingPair.value || !pendingEvent.value) return
  const resolved = resolveEventType(pendingEvent.value)
  if (!resolved) return
  logDebug(`Resolve event (watch) -> ${resolved}`)
  triggerLoad(pendingPair.value, resolved)
  pendingPair.value = null
  pendingEvent.value = null
})

async function load(pairOverride?: string, eventTypeOverride?: string) {
  const pair = pairOverride ?? store.selectedPair
  const eventType = eventTypeOverride ?? store.selectedEventType
  if (!pair || !eventType) return
  logDebug(`Load start -> ${pair} / ${eventType}`)
  store.loading = true
  store.error = null
  try {
    await analyzePeakDelay(pair, eventType)
    if (peakDelayResults.value) {
      logDebug(`Peak delay ok (n=${peakDelayResults.value.event_count})`)
    } else if (peakDelayError.value) {
      logDebug(`Peak delay error: ${peakDelayError.value}`)
    } else {
      logDebug('Peak delay empty')
    }
    await analyzeDecayProfile(pair, eventType)
    if (decayResults.value) {
      logDebug(`Decay profile ok (n=${decayResults.value.event_count})`)
    } else if (decayError.value) {
      logDebug(`Decay profile error: ${decayError.value}`)
    } else {
      logDebug('Decay profile empty')
    }
    await chargerDonnéesGraph(pair, eventType)
    if (graphData.value) {
      logDebug(`Volatility profile ok (n=${graphData.value.event_count})`)
    } else {
      logDebug('Volatility profile empty')
    }
    
    // Sync to store
    store.peakDelayResults = peakDelayResults.value
    store.decayResults = decayResults.value
    store.graphData = graphData.value
    logDebug(`Store sync ok (graphData=${store.graphData ? 'set' : 'null'})`)
  } catch (e) {
    store.error = String(e)
    logDebug(`Load error: ${store.error}`)
  } finally {
    store.loading = false
    logDebug('Load end')
  }
}

function triggerLoad(pair: string, eventType: string) {
  if (!pair || !eventType) return
  let resolvedEventType = resolveEventType(eventType)
  if (!resolvedEventType) {
    pendingPair.value = pair
    pendingEvent.value = eventType
    logDebug(`Trigger load deferred (event not resolved): ${eventType}`)
    return
  }
  if (store.selectedPair !== pair) store.selectedPair = pair
  if (store.selectedEventType !== resolvedEventType) store.selectedEventType = resolvedEventType
  logDebug(`Trigger load ${pair} / ${resolvedEventType}`)
  load(pair, resolvedEventType)
}

defineExpose({
  triggerLoad
})

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
    timeout: store.graphData.timeout,
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
