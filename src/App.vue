<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useVolatilityStore } from './stores/volatility'
import { useAnalysisStore } from './stores/analysisStore'
import AnalysisPanel from './components/AnalysisPanel.vue'
import HourlyTable from './components/HourlyTable.vue'
import MetricsAnalysisModal from './components/MetricsAnalysisModal.vue'
import FormulasModal from './components/FormulasModal.vue'
import ExportModal from './components/ExportModal.vue'
import ImportHub from './components/ImportHub.vue'
import EventCorrelationView from './components/EventCorrelationView.vue'
import ArchivesView from './views/ArchivesView.vue'
import BacktestView from './views/BacktestView.vue'
import PlanningView from './views/PlanningView.vue'
import HomeView from './views/HomeView.vue'

const volatilityStore = useVolatilityStore()
const analysisStore = useAnalysisStore()
const { analysisResult, loading, error } = storeToRefs(volatilityStore)

const savedTab = localStorage.getItem('activeTab') as string | null
const initialTab = (savedTab && ['heatmap', 'volatility', 'retrospective', 'backtest', 'planning', 'archives', 'calendar', 'home'].includes(savedTab)) 
  ? savedTab 
  : 'home'

const activeTab = ref<string>(initialTab)
const selectedSymbolLocal = ref('')

watch(activeTab, (newTab) => {
  localStorage.setItem('activeTab', newTab)
})

onMounted(async () => {
  try {
    await invoke('init_candle_index', {})
  } catch (_err) {
    // Non-bloquant
  }
  volatilityStore.loadSymbols()
  analysisStore.restoreHeatmapFromStorage()
})

// Gestion des modales globales
const showFormulasModal = ref(false)
const showExportModal = ref(false)
const isQuarterAnalysisOpen = ref(false)
const quarterAnalysisQuarter = ref<number | undefined>(undefined)

async function handleSymbolSelected(symbol: string) {
  await volatilityStore.analyzeSymbol(symbol)
}

async function handleSymbolChange() {
  if (selectedSymbolLocal.value) {
    await volatilityStore.analyzeSymbol(selectedSymbolLocal.value)
  }
}


function switchTab(tab: string) {
  activeTab.value = tab
}

function handleOpenModal(modal: 'formulas' | 'export') {
  if (modal === 'formulas') showFormulasModal.value = true
  if (modal === 'export') showExportModal.value = true
}

function handleQuarterAnalyze(hour: number, quarter: number) {
  quarterAnalysisQuarter.value = quarter
  isQuarterAnalysisOpen.value = true
}

// Contrôles de fenêtre (titlebar custom)
const appWindow = getCurrentWindow()
function minimizeWindow() { appWindow.minimize() }
function toggleMaximize() { appWindow.toggleMaximize() }
function closeWindow() { appWindow.close() }
</script>

<template>
  <div class="app-shell">
    <!-- === TITLEBAR / NAVIGATION HEADER === -->
    <header
      class="app-header"
      data-tauri-drag-region
    >
      <div class="header-left">
        <button
          v-if="activeTab !== 'home'"
          class="nav-home-btn"
          title="Retour Accueil"
          @click="activeTab = 'home'"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="feather feather-home"
          >
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
            <polyline points="9 22 9 12 15 12 15 22" />
          </svg>
          <span class="home-text">Accueil</span>
        </button>
        <div class="brand">
          <img
            src="./assets/logo.png"
            alt="Logo"
            width="20"
            height="20"
          >
          <span>Volatility Analyzer</span>
        </div>
      </div>

      <div class="header-center">
        <!-- Optional: Toolbar items can go here -->
      </div>

      <div class="window-controls">
        <button
          class="win-btn"
          title="Minimiser"
          @click="minimizeWindow"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 12 12"
          >
            <rect
              x="1"
              y="5.5"
              width="10"
              height="1"
              fill="currentColor"
            />
          </svg>
        </button>
        <button
          class="win-btn"
          title="Maximiser"
          @click="toggleMaximize"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 12 12"
          >
            <rect
              x="1.5"
              y="1.5"
              width="9"
              height="9"
              rx="1"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
          </svg>
        </button>
        <button
          class="win-btn win-close"
          title="Fermer"
          @click="closeWindow"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 12 12"
          >
            <line
              x1="2"
              y1="2"
              x2="10"
              y2="10"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
            <line
              x1="10"
              y1="2"
              x2="2"
              y2="10"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </header>

    <!-- === APP CONTENT AREA === -->
    <main class="app-viewport">
      <!-- HOME VIEW -->
      <HomeView
        v-show="activeTab === 'home'"
        @navigate="switchTab"
        @open-modal="handleOpenModal"
      />

      <!-- WORKSPACE VIEWS (v-show pour préserver le cache KeepAlive lors du passage par Home) -->
      <div
        v-show="activeTab !== 'home'"
        class="workspace-container"
      >
        <!-- Volatility Tab -->
        <template v-if="activeTab === 'volatility'">
          <div class="volatility-layout">
            <!-- Loading State -->
            <div
              v-if="loading"
              class="state-msg"
            >
              <div class="spinner" />
              <p>Analyse en cours...</p>
            </div>

            <!-- Error State -->
            <div
              v-if="error"
              class="error-msg"
            >
              <h3>❌ Erreur</h3>
              <p>{{ error }}</p>
            </div>

            <!-- Empty State -->
            <div
              v-if="!loading && !analysisResult && !error"
              class="empty-state"
            >
              <div class="welcome-icon">
                💱
              </div>
              <h3>Sélectionnez un symbole pour commencer</h3>
              <select
                v-model="selectedSymbolLocal"
                :disabled="loading"
                class="symbol-select"
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

            <!-- Results -->
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
                :point-value="analysisResult.point_value"
                :unit="analysisResult.unit"
                :symbol="analysisResult.symbol"
                @analyze-quarter="handleQuarterAnalyze"
              />
            </template>
          </div>
        </template>

        <!-- Other Tabs -->
        <template v-if="activeTab === 'calendar'">
          <ImportHub />
        </template>

        <!-- KeepAlive : préserve l'état des onglets lourds lors des switches -->
        <KeepAlive>
          <EventCorrelationView
            v-if="activeTab === 'heatmap'"
            key="heatmap"
            :view-mode="'heatmap'"
          />
        </KeepAlive>
        <KeepAlive>
          <EventCorrelationView
            v-if="activeTab === 'retrospective'"
            key="retrospective"
            :view-mode="'retrospective'"
          />
        </KeepAlive>
        <KeepAlive>
          <BacktestView
            v-if="activeTab === 'backtest'"
            key="backtest"
          />
        </KeepAlive>

        <template v-if="activeTab === 'archives'">
          <ArchivesView />
        </template>
        <template v-if="activeTab === 'planning'">
          <PlanningView />
        </template>
      </div>
    </main>

    <!-- MODALS -->
    <FormulasModal
      :is-open="showFormulasModal"
      @close="showFormulasModal = false"
    />
    <ExportModal
      :is-open="showExportModal"
      :current-symbol="selectedSymbolLocal"
      @close="showExportModal = false"
    />
    <MetricsAnalysisModal
      :is-open="isQuarterAnalysisOpen"
      :analysis-result="analysisResult"
      :pre-selected-quarter="quarterAnalysisQuarter"
      @close="isQuarterAnalysisOpen = false"
    />
  </div>
</template>

<style>
/* Global Resets */
:root {
  --app-bg: #0f1419;
  --panel-bg: #161b22;
  --border-color: #30363d;
  --text-primary: #e6edf3;
  --text-secondary: #8b949e;
  --accent: #58a6ff;
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--app-bg);
  color: var(--text-primary);
  overflow: hidden; /* Prevent body scroll */
}

select {
  background: #ffffff;
  color: #000000;
}

select option {
  background: #ffffff;
  color: #000000;
}
</style>

<style scoped>
.app-shell {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  background: var(--app-bg);
  overflow: hidden;
  position: relative;
}

/* --- Header Navigation --- */
.app-header {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px 0 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--app-bg);
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
}

.nav-home-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255,255,255,0.05);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}
.nav-home-btn:hover { background: rgba(255,255,255,0.1); border-color: #8b949e; }
.brand { font-weight: 600; color: var(--text-secondary); font-size: 0.85rem; display: flex; align-items: center; gap: 6px; pointer-events: none; }
.brand span { opacity: 0.8; }

/* Window Controls */
.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  -webkit-app-region: no-drag;
}
.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 28px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.15s, color 0.15s;
}
.win-btn:hover { background: rgba(255,255,255,0.08); color: var(--text-primary); }
.win-close:hover { background: #e81123; color: #fff; }

/* --- VIEWPORT --- */
.app-viewport {
  flex: 1;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

.workspace-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 0 0 0 0; /* Full width */
}

.volatility-layout {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow-y: auto;
  gap: 16px;
}

.tab-panel {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* States */
.state-msg { text-align: center; padding: 40px; color: var(--text-secondary); }
.spinner { width: 30px; height: 30px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 1s infinite linear; margin: 0 auto 10px; }
@keyframes spin { to { transform: rotate(360deg); } }

.error-msg { background: rgba(211, 47, 47, 0.1); border: 1px solid rgba(211, 47, 47, 0.3); color: #ff7b72; padding: 16px; border-radius: 8px; }

.empty-state {
  text-align: center;
  padding: 60px;
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  max-width: 600px;
  margin: 40px auto;
}
.welcome-icon { font-size: 4rem; margin-bottom: 20px; }
.symbol-select {
  margin-top: 20px;
  padding: 10px 20px;
  font-size: 1rem;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: #ffffff;
  color: #000000;
  min-width: 200px;
}
.symbol-select option {
  color: #000000;
}
</style>
