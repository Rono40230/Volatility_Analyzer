<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useVolatilityStore } from './stores/volatility'
import SymbolSelector from './components/SymbolSelector.vue'
import AnalysisPanel from './components/AnalysisPanel.vue'
import HourlyTable from './components/HourlyTable.vue'
import CalendarView from './components/CalendarView.vue'
import EventCorrelationView from './components/EventCorrelationView.vue'
import SessionAnalysisView from './components/SessionAnalysisView.vue'

onMounted(() => {
  console.log('🎯 Vue App.vue mounted - Application Vue initialisée');
  console.log('📱 activeTab:', activeTab.value);
  console.log('🌐 window.location:', window.location.href);
  console.log('📦 Composants importés:', { SymbolSelector, AnalysisPanel, HourlyTable, CalendarView, EventCorrelationView, SessionAnalysisView });
  // Charger les symboles au démarrage
  store.loadSymbols()
});

const store = useVolatilityStore()
const { analysisResult, selectedSymbol, loading, error } = storeToRefs(store)

const activeTab = ref<'volatility' | 'calendar' | 'correlation' | 'sessions'>('volatility')
const selectedSymbolLocal = ref('')

async function handleSymbolSelected(symbol: string) {
  console.log('🔍 Symbol selected:', symbol);
  await store.analyzeSymbol(symbol)
}

async function handleSymbolChange() {
  if (selectedSymbolLocal.value) {
    console.log('🔍 Symbol selected:', selectedSymbolLocal.value);
    await store.analyzeSymbol(selectedSymbolLocal.value)
  }
}

function switchTab(tab: 'volatility' | 'calendar' | 'correlation' | 'sessions') {
  console.log('🔄 Changement d onglet:', tab);
  activeTab.value = tab
}
</script>

<template>
  <div class="app">
    <nav class="app-tabs">
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'volatility' }"
        @click="switchTab('volatility')"
      >
        📊 Analyse Volatilité
      </button>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'correlation' }"
        @click="switchTab('correlation')"
      >
        📈 Corrélation Événements
      </button>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'sessions' }"
        @click="switchTab('sessions')"
      >
        📍 Analyse par Sessions
      </button>
      <div class="tab-spacer"></div>
      <button 
        class="tab-button" 
        :class="{ active: activeTab === 'calendar' }"
        @click="switchTab('calendar')"
      >
        📥 Importer des données
      </button>
    </nav>

    <main class="app-main">
      <template v-if="activeTab === 'volatility'">
        <div class="main-container">
          <div class="header-section">
            <div class="header-left">
              <h2 class="main-title">
                <span class="icon">📊</span>
                Analyse de Volatilité
              </h2>
              <p class="main-subtitle">
                Analyse rétrospective de la volatilité historique par heure et période
              </p>
            </div>
            <div class="header-right">
              <select 
                id="volatility-symbol-select"
                v-model="selectedSymbolLocal" 
                @change="handleSymbolChange"
                :disabled="loading"
                class="inline-symbol-select"
              >
                <option value="" style="color: #000000;">Choisir un symbole</option>
                <option 
                  v-for="symbol in store.symbols" 
                  :key="symbol.symbol" 
                  :value="symbol.symbol"
                >
                  {{ symbol.symbol }}
                </option>
              </select>
            </div>
          </div>

          <div class="content-area">
            <div v-if="loading" class="loading">
              <div class="spinner"></div>
              <p>Analyse en cours...</p>
            </div>

            <div v-if="error" class="error">
              <h3>❌ Erreur</h3>
              <p>{{ error }}</p>
            </div>

            <div v-if="!loading && !analysisResult && !error" class="welcome">
              <div class="welcome-icon">📊</div>
              <h3>Sélectionnez un symbole pour commencer</h3>
              <p class="info-text">
                Choisissez un symbole forex ci-dessus pour analyser sa volatilité historique 
                par heure, jour et période.
              </p>
            </div>

            <template v-if="!loading && analysisResult">
              <AnalysisPanel :result="analysisResult" />
              <HourlyTable 
                :stats="analysisResult.hourly_stats" 
                :best-hours="analysisResult.best_hours"
              />
            </template>
          </div>
        </div>
      </template>

      <template v-if="activeTab === 'calendar'">
        <CalendarView />
      </template>

      <template v-if="activeTab === 'correlation'">
        <EventCorrelationView />
      </template>

      <template v-if="activeTab === 'sessions'">
        <div class="sessions-container">
          <SessionAnalysisView />
        </div>
      </template>
    </main>

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
  min-height: 100vh;
  background: linear-gradient(135deg, #0f1419 0%, #1c2128 100%);
  display: flex;
  flex-direction: column;
}

.app-tabs {
  display: flex;
  gap: 10px;
  padding: 15px 20px;
  background: #161b22;
  box-shadow: 0 2px 8px rgba(0,0,0,0.4);
  border-bottom: 1px solid #30363d;
}

.tab-spacer {
  flex: 1;
}

.tab-button {
  padding: 12px 24px;
  border: 2px solid #30363d;
  background: #0d1117;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.tab-button:hover {
  background: #161b22;
  border-color: #58a6ff;
  color: #58a6ff;
}

.tab-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 0 12px rgba(88, 166, 255, 0.3);
}

.app-main {
  flex: 1;
  padding: 20px;
  width: 100%;
}

/* Structure harmonisée : bloc principal */
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
}

/* En-tête du bloc */
.header-section {
  background: linear-gradient(135deg, #1c2128 0%, #161b22 100%);
  padding: 30px;
  border-bottom: 2px solid #30363d;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 30px;
}

.header-left {
  flex: 1;
}

.header-right {
  display: flex;
  align-items: center;
  min-width: 250px;
}

.inline-symbol-select {
  width: 100%;
  padding: 10px 14px;
  font-size: 1em;
  border: 2px solid #30363d;
  border-radius: 8px;
  background: #ffffff;
  color: #000000;
  cursor: pointer;
  transition: all 0.3s;
}

.inline-symbol-select:hover:not(:disabled) {
  border-color: #58a6ff;
  background: #f8f9fa;
}

.inline-symbol-select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.2);
}

.inline-symbol-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inline-symbol-select option {
  background: #ffffff;
  color: #000000;
}

.main-title {
  display: flex;
  align-items: center;
  gap: 15px;
  color: #e6edf3;
  font-size: 2em;
  margin: 0 0 10px 0;
  font-weight: 700;
}

.main-title .icon {
  font-size: 1.2em;
}

.main-subtitle {
  color: #8b949e;
  font-size: 1.1em;
  margin: 0;
  line-height: 1.5;
}

/* Blocs internes (boutons de mode) */
.view-modes {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 1px solid #30363d;
}

.mode-button {
  flex: 1;
  padding: 15px 20px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1.1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.mode-button:hover:not(:disabled) {
  background: #1c2128;
  border-color: #58a6ff;
  color: #58a6ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.3);
}

.mode-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 4px 12px rgba(88, 166, 255, 0.4);
}

.mode-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Zone de contenu */
.content-area {
  padding: 30px;
  min-height: 400px;
}

.selector-view,
.results-view {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.info-text {
  color: #8b949e;
  line-height: 1.6;
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

.welcome p {
  font-size: 1.2em;
  color: #8b949e;
  margin-bottom: 30px;
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
