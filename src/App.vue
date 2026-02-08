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

const appWindow = getCurrentWindow()
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
  console.log('[DIAG] App.vue onMounted START, activeTab =', activeTab.value)
  try {
    console.log('[DIAG] Calling init_candle_index...')
    await invoke('init_candle_index', {})
    console.log('[DIAG] init_candle_index OK')
  } catch (err) {
    console.log('[DIAG] init_candle_index failed (non-blocking):', err)
  }
  console.log('[DIAG] Calling loadSymbols...')
  volatilityStore.loadSymbols()
  console.log('[DIAG] Calling restoreHeatmapFromStorage...')
  analysisStore.restoreHeatmapFromStorage()
  console.log('[DIAG] App.vue onMounted END')
})

// Gestion des modales globales
const showFormulasModal = ref(false)
const showExportModal = ref(false)
const isQuarterAnalysisOpen = ref(false)
const quarterAnalysisHour = ref<number | undefined>(undefined)
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
  quarterAnalysisHour.value = hour
  quarterAnalysisQuarter.value = quarter
  isQuarterAnalysisOpen.value = true
}

// Window Actions
function minimize() { appWindow.minimize() }
function toggleMaximize() { appWindow.toggleMaximize() }
function closeApp() { appWindow.close() }
</script>

<template>
  <div class="app-shell">
    
    <!-- === SYSTEME WINDOW (Drag + Controls) === -->
    <!-- Masqué sur Linux : decorations: true utilise la titlebar native du WM -->
    <!-- <div class="titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
      <div class="titlebar-spacer"></div>
      <div class="window-controls">
        <button @click="minimize" class="win-btn" title="Réduire">
          <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
        </button>
        <button @click="toggleMaximize" class="win-btn" title="Agrandir">
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1,1 L9,9 L1,9 z" fill="none" stroke="currentColor"/></svg>
        </button>
        <button @click="closeApp" class="win-btn close-btn" title="Fermer">
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1,1 L9,9 M9,1 L1,9" stroke="currentColor"/></svg>
        </button>
      </div>
    </div> -->

    <!-- === MAIN NAVIGATION HEADER === -->
    <!-- Positionné en flux normal, mais doit être capable d'être cliqué si pas couvert totalement par Drag Region -->
    <!-- Drag Region fait 32px height fixed top. Ce header fait 48px. -->
    <header class="app-header">
      <div class="header-left">
        <button v-if="activeTab !== 'home'" @click="activeTab = 'home'" class="nav-home-btn" title="Retour Accueil">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-home"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
          <span class="home-text">Accueil</span>
        </button>
        <div v-else class="brand">
          <img src="./assets/logo.png" alt="Logo" width="24" height="24" /> Volatility Analyzer
        </div>
      </div>
      
      <div class="header-center">
        <!-- Optional: Toolbar items can go here -->
      </div>
    </header>

    <!-- === APP CONTENT AREA === -->
    <main class="app-viewport">
      
      <!-- HOME VIEW -->
      <HomeView v-show="activeTab === 'home'" @navigate="switchTab" @open-modal="handleOpenModal"/>

      <!-- WORKSPACE VIEWS (v-show pour préserver le cache KeepAlive lors du passage par Home) -->
      <div v-show="activeTab !== 'home'" class="workspace-container">
        <!-- Volatility Tab -->
        <template v-if="activeTab === 'volatility'">
           <div class="volatility-layout">
             <!-- Loading State -->
             <div v-if="loading" class="state-msg">
                <div class="spinner"></div>
                <p>Analyse en cours...</p>
             </div>

             <!-- Error State -->
             <div v-if="error" class="error-msg">
                <h3>❌ Erreur</h3>
                <p>{{ error }}</p>
             </div>

             <!-- Empty State -->
             <div v-if="!loading && !analysisResult && !error" class="empty-state">
                <div class="welcome-icon">💱</div>
                <h3>Sélectionnez un symbole pour commencer</h3>
                <select v-model="selectedSymbolLocal" :disabled="loading" class="symbol-select" @change="handleSymbolChange">
                  <option value="">Choisir un symbole</option>
                  <option v-for="symbol in volatilityStore.symbols" :key="symbol.symbol" :value="symbol.symbol">{{ symbol.symbol }}</option>
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
        <template v-if="activeTab === 'calendar'"><ImportHub /></template>

        <!-- KeepAlive : préserve l'état des onglets lourds lors des switches -->
        <KeepAlive>
          <EventCorrelationView v-if="activeTab === 'heatmap'" :view-mode="'heatmap'" key="heatmap" />
        </KeepAlive>
        <KeepAlive>
          <EventCorrelationView v-if="activeTab === 'retrospective'" :view-mode="'retrospective'" key="retrospective" />
        </KeepAlive>
        <KeepAlive>
          <BacktestView v-if="activeTab === 'backtest'" key="backtest" />
        </KeepAlive>

        <template v-if="activeTab === 'archives'"><ArchivesView /></template>
        <template v-if="activeTab === 'planning'"><PlanningView /></template>
      </div>

    </main>

    <!-- MODALS -->
    <FormulasModal :is-open="showFormulasModal" @close="showFormulasModal = false" />
    <ExportModal :is-open="showExportModal" :current-symbol="selectedSymbolLocal" @close="showExportModal = false" />
    <MetricsAnalysisModal
      :is-open="isQuarterAnalysisOpen"
      :analysis-result="analysisResult"
      :pre-selected-hour="quarterAnalysisHour"
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

/* --- TITLE BAR SYSTEM (Custom, safe drag region) --- */

.titlebar {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border-color);
  user-select: none;
}

.titlebar-spacer { flex: 1; }

.window-controls {
  display: flex;
  align-items: center;
  padding-right: 4px;
}
.win-btn {
  width: 46px;
  height: 32px;
  background: transparent;
  border: none;
  color: #8b949e;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}
.win-btn:hover { background: rgba(255,255,255,0.1); color: #fff; }
.close-btn:hover { background: #d32f2f; color: #fff; }

/* 3. Header Navigation (In Flow) */
.app-header {
  height: 48px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  padding-top: 10px; /* Compensate for visually hidden top part if needed, though 32px drag covers top 32px */
  border-bottom: 1px solid var(--border-color);
  background: var(--panel-bg);
  flex-shrink: 0;
}
/* NOTE: Top 32px of viewport is covered by Drag Region (z-40).
   The header is top-aligned in flex column.
   So header top 32px is unclickable.
   BUT header buttons must be clickable.
   SOLUTION: Give header buttons z-index 41+ or move header down.
   We'll use z-index relative on the buttons.
*/
.header-left {
  display: flex;
  align-items: center;
  position: relative;
  z-index: 45; /* Higher than Drag Region */
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
.brand { font-weight: 700; color: #e6edf3; font-size: 1rem; display: flex; align-items: center; gap: 8px; }

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
