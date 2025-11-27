<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header"><div class="header-title"><span class="icon">🎯</span><h2>Métriques du meilleur moment pour trader</h2></div><button class="close-btn" @click="close">✕</button></div>
      <div class="modal-section">
        <div v-if="sliceAnalyses && sliceAnalyses.length > 0" class="slices-container">
          <BestSliceCard v-for="analysis in sliceAnalyses.filter(a => a.rank === 1)" :key="`slice-${analysis.rank}`" :analysis="analysis" :volatility-duration="volatilityDuration" :movement-qualities="movementQualities">
            <MetricsGrid :analysis="analysis" :analysis-data="analysisData" />
            <VolatilityDurationSection :volatility-duration="volatilityDuration" :trading-plan="tradingPlan" />
            <BidiParametersSection :slice-analyses="sliceAnalyses" :entry-window-analysis="entryWindowAnalysis" :analysis="analysis" />
            <ObservationsSection :analysis="analysis" :analysis-data="analysisData" :movement-qualities="movementQualities" :volatility-duration="volatilityDuration" />
            <StraddlePerformanceSection :win-rate="winRate" :whipsaw-analysis="whipsawAnalysis" :offset-optimal="offsetOptimal" :win-rate-color="winRateColor" />
            <VolatilityDecayChart 
              v-if="tradingPlan && volatilityDuration"
              :peak-volatility="(tradingPlan.atrPercentage ?? 2.5) / 100"
              :half-life-minutes="volatilityDuration.volatility_half_life_minutes ?? 120"
              :recommended-duration="tradingPlan.tradeDurationMinutes ?? 180"
              :start-hour="bestSliceHour"
              :start-minute="0"
            />
          </BestSliceCard>
        </div>
        <div v-if="!sliceAnalyses || sliceAnalyses.length === 0" class="no-data"><p>Aucune donnée disponible pour l'analyse</p></div>
      </div>
      <div class="modal-footer"><button v-if="!isArchiveMode" class="btn-archive" @click="openArchiveModal">💾 Archiver</button><button class="btn-primary" @click="close">Fermer l'analyse</button></div>
    </div>
  </div>
  <ArchiveModal :show="showArchiveModal" archive-type="Volatilité brute" :period-start="archivePeriodStart" :period-end="archivePeriodEnd" :symbol="analysisData?.symbol" :timeframe="'M1'" :data-json="archiveDataJson" @close="showArchiveModal = false" @saved="handleArchiveSaved" />
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import ArchiveModal from './ArchiveModal.vue'
import { useStraddleAnalysis } from '../composables/useStraddleAnalysis'
import { useMetricsAnalysisData } from '../composables/useMetricsAnalysisData'
import BestSliceCard from './metrics/BestSliceCard.vue'
import MetricsGrid from './metrics/MetricsGrid.vue'
import MovementQualitySection from './metrics/MovementQualitySection.vue'
import VolatilityDurationSection from './metrics/VolatilityDurationSection.vue'
import BidiParametersSection from './metrics/BidiParametersSection.vue'
import ObservationsSection from './metrics/ObservationsSection.vue'
import StraddlePerformanceSection from './metrics/StraddlePerformanceSection.vue'
import VolatilityDecayChart from './VolatilityDecayChart.vue'

interface Props {
  isOpen: boolean
  analysisResult: AnalysisResult | null
  isArchiveMode?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const { analysisData, sliceAnalyses, movementQualities, volatilityDuration, tradingPlan, entryWindowAnalysis, updateAnalysis } = useMetricsAnalysisData()
const { isLoading, offsetOptimal, winRate, whipsawAnalysis, analyzeStraddleMetrics, winRateColor } = useStraddleAnalysis()

const bestSliceHour = computed(() => {
  if (!sliceAnalyses.value || sliceAnalyses.value.length === 0) return 13
  const bestSlice = sliceAnalyses.value.find(a => a.rank === 1)
  return bestSlice?.slice?.hour ?? 13
})

watch(() => props.analysisResult, (result) => { if (result) updateAnalysis(result) })
watch(() => props.isOpen, (isOpen) => { if (isOpen && props.analysisResult) updateAnalysis(props.analysisResult) })
onMounted(() => { if (props.isOpen && props.analysisResult) updateAnalysis(props.analysisResult) })

watch(() => sliceAnalyses.value, async (newSlices) => {
  if (newSlices?.length > 0 && props.analysisResult) {
    const bestSlice = newSlices[0]
    if (bestSlice?.slice?.stats) {
      try {
        const symbol = props.analysisResult.symbol || 'EURUSD'
        const hour = bestSlice.slice?.hour || 0
        const quarter = bestSlice.slice?.quarter || 0
        await analyzeStraddleMetrics(symbol, hour, quarter)
      } catch (error) {
        // Straddle analysis error
      }
    }
  }
}, { deep: true })

const close = () => { emit('close') }

const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')

function openArchiveModal() {
  if (!props.analysisResult) return
  const result = props.analysisResult
  if (result.period_start && result.period_end) {
    archivePeriodStart.value = result.period_start
    archivePeriodEnd.value = result.period_end
  } else {
    const now = new Date()
    const oneYearAgo = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
    archivePeriodStart.value = oneYearAgo.toISOString()
    archivePeriodEnd.value = now.toISOString()
  }
  
  // Sérialiser les données d'analyse
  archiveDataJson.value = JSON.stringify({
    analysisResult: result,
    sliceAnalyses: sliceAnalyses.value,
    movementQualities: movementQualities.value,
    volatilityDuration: volatilityDuration.value,
    tradingPlan: tradingPlan.value,
    entryWindowAnalysis: entryWindowAnalysis
  })
  
  showArchiveModal.value = true
}

function handleArchiveSaved() {
  showArchiveModal.value = false
}

</script>

<style scoped lang="css">
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: #1a1f2e;
  border: 2px solid #2d3748;
  border-radius: 12px;
  width: 100%;
  max-width: 1600px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
  color: #e2e8f0;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border-bottom: 2px solid #2d3748;
  background: linear-gradient(135deg, #1a1f2e 0%, #2d3748 100%);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title .icon {
  font-size: 24px;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #cbd5e0;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.modal-section {
  padding: 24px;
  border-bottom: 1px solid #2d3748;
}

/* Modal Footer */
.modal-footer {
  padding: 20px 24px;
  border-top: 1px solid #2d3748;
  background: rgba(45, 55, 72, 0.3);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-primary {
  padding: 10px 20px;
  background: #3b82f6;
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-archive {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  margin-right: 10px;
}

.btn-archive:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

/* No Data */
.no-data {
  text-align: center;
  padding: 40px;
  color: #a0aec0;
}

/* Scrollbar */
.modal-content::-webkit-scrollbar {
  width: 8px;
}

.modal-content::-webkit-scrollbar-track {
  background: rgba(45, 55, 72, 0.3);
}

.modal-content::-webkit-scrollbar-thumb {
  background: #4a5568;
  border-radius: 4px;
}

.modal-content::-webkit-scrollbar-thumb:hover {
  background: #718096;
}
</style>
