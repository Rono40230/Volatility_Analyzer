<template>
  <div class="main-container">
    <AnalysisGroupTabs v-model="analysisGroup" />
    
    <div v-if="analysisGroup === 'correlation'" class="analysis-group">
      <CorrelationViewModeTabs v-model="viewMode">
        <CalendarFileSelector 
          class="file-selector-right"
          @file-selected="handleCalendarSelected"
        />
      </CorrelationViewModeTabs>
      <div class="content-area">
        <EventCorrelationByEvent
          v-if="viewMode === 'by-event'"
          :past-events="pastEvents"
          :calendar-id="selectedCalendarId"
        />
        <EventCorrelationByPair
          v-if="viewMode === 'by-pair'"
          :available-pairs="availablePairs"
        />
        <EventCorrelationHeatmap
          v-if="viewMode === 'heatmap'"
          :calendar-id="selectedCalendarId"
          :available-pairs="availablePairs"
        />
      </div>
    </div>

    <div v-if="analysisGroup === 'retrospective'" class="analysis-group">
      <RetrospectiveViewModeTabs v-model="retrospectiveView" />
      <div class="content-area">
        <PeakDelayAnalysis v-if="retrospectiveView === 'peak-delay'" />
        <DecayProfileView v-if="retrospectiveView === 'decay'" />
        <EntryTimingProfitability v-if="retrospectiveView === 'entry-timing'" />
        <DirectionalBiasView v-if="retrospectiveView === 'bias'" />
        <WhipsawRootCauseView v-if="retrospectiveView === 'whipsaw'" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import AnalysisGroupTabs from './AnalysisGroupTabs.vue'
import CorrelationViewModeTabs from './CorrelationViewModeTabs.vue'
import RetrospectiveViewModeTabs from './RetrospectiveViewModeTabs.vue'
import EventCorrelationByEvent from './EventCorrelationByEvent.vue'
import EventCorrelationByPair from './EventCorrelationByPair.vue'
import EventCorrelationHeatmap from './EventCorrelationHeatmap.vue'
import CalendarFileSelector from './CalendarFileSelector.vue'
import PeakDelayAnalysis from './PeakDelayAnalysis.vue'
import DecayProfileView from './DecayProfileView.vue'
import EntryTimingProfitability from './EntryTimingProfitability.vue'
import DirectionalBiasView from './DirectionalBiasView.vue'
import WhipsawRootCauseView from './WhipsawRootCauseView.vue'

interface PastEvent { name: string; count: number }

const store = useVolatilityStore()
const analysisGroup = ref<'correlation' | 'retrospective'>('correlation')
const viewMode = ref<'by-event' | 'by-pair' | 'heatmap'>('by-event')
const retrospectiveView = ref<'peak-delay' | 'decay' | 'entry-timing' | 'bias' | 'whipsaw'>('peak-delay')
const pastEvents = ref<PastEvent[]>([])
const availablePairs = ref<string[]>([])
const selectedCalendarId = ref<number | null>(null)

const { onPairDataRefresh } = useDataRefresh()
const unsubscribe = onPairDataRefresh(loadAvailablePairs)
onBeforeUnmount(() => unsubscribe())

watch(() => store.dataRefreshTrigger, () => loadPastEvents())
onMounted(async () => {
  await loadAvailablePairs()
  await loadPastEvents()
})

async function handleCalendarSelected(filename: string) {
  const calendarId = await invoke<number | null>('get_calendar_id_by_filename', { filename })
  selectedCalendarId.value = calendarId
  await loadPastEvents()
}

async function loadPastEvents() {
  if (!selectedCalendarId.value) {
    pastEvents.value = []
    return
  }
  try {
    pastEvents.value = await invoke<PastEvent[]>('get_past_events', { 
      monthsBack: 6,
      calendarId: selectedCalendarId.value
    })
  } catch {
    pastEvents.value = []
  }
}

async function loadAvailablePairs() {
  try {
    const data = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
    availablePairs.value = data.map(item => item.symbol)
  } catch {
    availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
  }
}
</script>

<style scoped>
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  color: #e2e8f0;
}

.analysis-group { width: 100%; }

.content-area {
  padding: 30px;
  min-height: 400px;
}

:deep(.file-selector-right) {
  margin-left: auto;
}
</style>
