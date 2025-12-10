<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from './stores/volatility'
import { useAnalysisStore } from './stores/analysisStore'
import AnalysisPanel from './components/AnalysisPanel.vue'
import HourlyTable from './components/HourlyTable.vue'
import MetricsAnalysisModal from './components/MetricsAnalysisModal.vue'
import FormulasModal from './components/FormulasModal.vue'
import ImportHub from './components/ImportHub.vue'
import EventCorrelationView from './components/EventCorrelationView.vue'
import ArchivesView from './views/ArchivesView.vue'

const volatilityStore = useVolatilityStore()
const analysisStore = useAnalysisStore()
const { analysisResult, loading, error } = storeToRefs(volatilityStore)

const savedTab = localStorage.getItem('activeTab') as 'volatility' | 'heatmap' | 'retrospective' | 'archives' | 'calendar' | null
const activeTab = ref<'volatility' | 'heatmap' | 'retrospective' | 'archives' | 'calendar'>(savedTab || 'heatmap')
const selectedSymbolLocal = ref('')

watch(activeTab, (newTab) => {
  localStorage.setItem('activeTab', newTab)
})

onMounted(async () => {
  try {
    await invoke('init_candle_index', {})
  } catch (error) {
    // Non-bloquant
  }
  volatilityStore.loadSymbols()
  analysisStore.restoreHeatmapFromStorage()
})

// Gestion de la modal Bidi depuis le tableau
const showBidiModal = ref(false)
const bidiModalHour = ref(0)
const bidiModalQuarter = ref(0)
const showFormulasModal = ref(false)

async function handleSymbolSelected(symbol: string) {
  await volatilityStore.analyzeSymbol(symbol)
}

async function handleSymbolChange() {
  if (selectedSymbolLocal.value) {
    await volatilityStore.analyzeSymbol(selectedSymbolLocal.value)
  }
}

function handleOpenBidiParams(data: { hour: number; quarter: number }) {
  bidiModalHour.value = data.hour
  bidiModalQuarter.value = data.quarter
  showBidiModal.value = true
}

function switchTab(tab: 'volatility' | 'heatmap' | 'retrospective' | 'archives' | 'calendar') {
  activeTab.value = tab
}
</script>

<template>
  <div class="app">
    <nav class="app-tabs">
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'heatmap' }"
        @click="switchTab('heatmap')"
      >
        🔥 Heatmap de Corrélation
      </button>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'volatility' }"
        @click="switchTab('volatility')"
      >
        📊 Volatilité brute
      </button>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'retrospective' }"
        @click="switchTab('retrospective')"
      >
        📊 Métriques Rétrospectives
      </button>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'archives' }"
        @click="switchTab('archives')"
      >
        🗄️ Archives
      </button>
      <div class="tab-spacer" />
      <button 
        class="tab-button" 
        @click="switchTab('calendar')"
      >
        📥 Importer des données
      </button>
      <button 
        class="tab-button formulas-btn"
        @click="showFormulasModal = true"
        title="Voir toutes les formules et calculs"
      >
        🧮 Formules
      </button>
    </nav>

    <main class="app-main">
      <template v-if="activeTab === 'volatility'">
        <div class="main-container">
          <div class="content-area">
            <div
              v-if="loading"
              class="loading"
            >
              <div class="hourglass">⏳</div>
              <p>Analyse en cours...</p>
            </div>

            <div
              v-if="error"
              class="error"
            >
              <h3>❌ Erreur</h3>
              <p>{{ error }}</p>
            </div>

            <div
              v-if="!loading && !analysisResult && !error"
              class="welcome"
            >
              <div class="welcome-icon">
                💱
              </div>
              <h3>Sélectionnez un symbole pour commencer</h3>
              <div class="welcome-select-container">
                <select 
                  id="volatility-symbol-select"
                  v-model="selectedSymbolLocal" 
                  :disabled="loading"
                  class="welcome-symbol-select"
                  @change="handleSymbolChange"
                >
                  <option value="">
                    Choisir un symbole
                  </option>
                  <option 
                    v-for="symbol in volatilityStore.symbols" 
                    :key="symbol.symbol" 
                    :value="symbol.symbol"
                  >
                    {{ symbol.symbol }}
                  </option>
                </select>
              </div>
            </div>

            <template v-if="!loading && analysisResult">
              <AnalysisPanel 
                :result="analysisResult" 
                :symbols="volatilityStore.symbols"
                @symbol-selected="handleSymbolSelected"
              />
              <HourlyTable 
                :stats="analysisResult.hourly_stats" 
                :best-quarter="analysisResult.best_quarter"
                :stats15min="analysisResult.stats_15min"
                :global-metrics="analysisResult.global_metrics"
                @open-bidi-params="handleOpenBidiParams"
              />
            </template>
          </div>
        </div>
      </template>

      <template v-if="activeTab === 'calendar'">
        <ImportHub />
      </template>

      <template v-if="activeTab === 'heatmap'">
        <EventCorrelationView :view-mode="'heatmap'" />
      </template>

      <template v-if="activeTab === 'retrospective'">
        <EventCorrelationView :view-mode="'retrospective'" />
      </template>

      <template v-if="activeTab === 'archives'">
        <ArchivesView />
      </template>
    </main>

    <MetricsAnalysisModal 
      v-if="showBidiModal"
      :is-open="showBidiModal"
      :analysis-result="analysisResult"
      :selected-symbol="analysisResult?.symbol || ''"
      :pre-selected-hour="bidiModalHour"
      :pre-selected-quarter="bidiModalQuarter"
      @close="showBidiModal = false"
    />

    <FormulasModal 
      :is-open="showFormulasModal"
      @close="showFormulasModal = false"
    />

    <footer class="app-footer">
      <p>Powered by Rust + Tauri 2.0 + Vue 3</p>
    </footer>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #0f1419;
  color: #e6edf3;
}
</style>

<style scoped>
.app {
  height: 100vh;
  background: linear-gradient(135deg, #0f1419 0%, #1c2128 100%);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-tabs {
  display: flex;
  gap: 10px;
  padding: 10px 20px;
  background: #161b22;
  box-shadow: 0 2px 8px rgba(0,0,0,0.4);
  border-bottom: 1px solid #30363d;
  flex-shrink: 0;
}

.tab-button {
  background: transparent;
  border: none;
  color: #8b949e;
  padding: 8px 16px;
  font-size: 0.95em;
  font-weight: 600;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
}

.tab-button:hover {
  background: rgba(177, 186, 196, 0.12);
  color: #c9d1d9;
}

.tab-button.active {
  background: rgba(56, 139, 253, 0.15);
  color: #58a6ff;
}

.tab-spacer {
  flex: 1;
}

.formulas-btn {
  color: #d29922;
}

.formulas-btn:hover {
  background: rgba(210, 153, 34, 0.15);
  color: #e3b341;
}

.app-main {
  flex: 1;
  padding: 10px;
  width: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.content-area {
  padding: 30px;
  min-height: 400px;
}

.loading {
  text-align: center;
  padding: 60px 20px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #30363d;
  border-top: 4px solid #58a6ff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

.hourglass {
  font-size: 60px;
  margin: 0 auto 20px;
  animation: hourglassFlip 1s ease-in-out infinite;
  display: inline-block;
}

@keyframes hourglassFlip {
  0% {
    transform: scaleX(1) rotateY(0deg);
  }
  50% {
    transform: scaleX(-1) rotateY(180deg);
  }
  100% {
    transform: scaleX(1) rotateY(360deg);
  }
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error {
  background: #2d1117;
  border: 2px solid #da3633;
  border-radius: 8px;
  padding: 20px;
  margin: 20px 0;
  color: #ff7b72;
}

.welcome {
  text-align: center;
  padding: 60px 20px;
  background: #161b22;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
}

.welcome-icon {
  font-size: 4em;
  margin-bottom: 20px;
}

.welcome h2 {
  font-size: 2em;
  color: #e6edf3;
  margin-bottom: 10px;
}

.welcome h3 {
  font-size: 1.8em;
  color: #e6edf3;
  margin-bottom: 15px;
}

.welcome p {
  font-size: 1.2em;
  color: #8b949e;
  margin-bottom: 30px;
}

.info-text {
  font-size: 1.1em;
  color: #a0aec0;
  max-width: 600px;
  margin: 0 auto 30px auto;
  line-height: 1.6;
}

.welcome-select-container {
  display: flex;
  justify-content: center;
  margin-top: 30px;
}

.welcome-symbol-select {
  padding: 12px 24px;
  font-size: 1.1em;
  border-radius: 8px;
  border: 2px solid #4a5568;
  background: #ffffff;
  color: #000000;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 250px;
}

.welcome-symbol-select option {
  background: #ffffff;
  color: #000000;
}

.welcome-symbol-select:hover {
  border-color: #667eea;
  background: #f7fafc;
}

.welcome-symbol-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.app-footer {
  background: #0d1117;
  color: #8b949e;
  text-align: center;
  padding: 20px;
  margin-top: 40px;
  border-top: 1px solid #30363d;
}
</style>
