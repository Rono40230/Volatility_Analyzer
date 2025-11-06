<template>
  <div class="main-container">
    <!-- En-tête avec titre et dropdown -->
    <div class="header-section">
      <div class="header-left">
        <h2 class="main-title">
          <span class="icon">🎯</span>
          Corrélation Événements × Paires
        </h2>
        <p class="main-subtitle">
          Analyse rétrospective : Quelle paire a le plus bougé lors de chaque événement passé ?
        </p>
      </div>
      <!-- Dropdown pour événement (mode Par Événement) -->
      <div class="header-right" v-if="viewMode === 'by-event'">
        <select 
          id="event-select" 
          v-model="selectedEventId" 
          class="inline-event-select"
          @change="loadEventImpact"
        >
          <option value="" style="color: #000000;">Choisir un événement</option>
          <option v-for="event in pastEvents" :key="event.id" :value="event.id">
            {{ event.name }} - {{ formatDate(event.datetime) }} ({{ event.impact }})
          </option>
        </select>
      </div>
      <!-- Dropdown pour paire (mode Par Paire) -->
      <div class="header-right" v-if="viewMode === 'by-pair'">
        <label for="pair-select" class="pair-label">💱 Paire :</label>
        <select 
          id="pair-select" 
          v-model="selectedPairSymbol" 
          class="inline-pair-select"
          @change="loadPairEventHistory"
        >
          <option value="">Choisir une paire</option>
          <option v-for="pair in availablePairs" :key="pair" :value="pair">
            {{ pair }}
          </option>
        </select>
      </div>
    </div>

    <!-- Mode de vue -->
    <div class="view-mode-selector">
      <button 
        class="mode-button" 
        :class="{ active: viewMode === 'by-event' }"
        @click="switchViewMode('by-event')"
      >
        📅 Par Événement
      </button>
      <button 
        class="mode-button" 
        :class="{ active: viewMode === 'by-pair' }"
        @click="switchViewMode('by-pair')"
      >
        💱 Par Paire
      </button>
      <button 
        class="mode-button" 
        :class="{ active: viewMode === 'heatmap' }"
        @click="switchViewMode('heatmap')"
      >
        🔥 Heatmap
      </button>
    </div>

    <!-- Contenu -->
    <div class="content-area">
      <!-- Vue 1: Par Événement -->
      <template v-if="viewMode === 'by-event'">
        <div v-if="loadingEvent" class="loading">
          <div class="spinner"></div>
          <p>Analyse de l'impact de l'événement...</p>
        </div>

      <div v-if="eventImpact && !loadingEvent" class="event-impact-results">
        <!-- Informations sur l'événement -->
        <div class="event-info-card">
          <div class="event-header">
            <h3>{{ eventImpact.event_name }}</h3>
            <span class="event-badge" :class="`impact-${eventImpact.impact.toLowerCase()}`">
              {{ eventImpact.impact }}
            </span>
          </div>
          <div class="event-details">
            <div class="detail-item">
              <span class="label">📅 Date :</span>
              <span class="value">{{ formatDateTime(eventImpact.datetime) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">🌍 Pays :</span>
              <span class="value">{{ eventImpact.country }}</span>
            </div>
            <div class="detail-item">
              <span class="label">💱 Devise :</span>
              <span class="value">{{ eventImpact.currency }}</span>
            </div>
          </div>
        </div>

        <!-- Classement des paires par impact -->
        <div class="impact-ranking">
          <h3>📊 Impact mesuré par paire ({{ eventImpact.window_start }} → {{ eventImpact.window_end }})</h3>
          
          <table class="impact-table">
            <thead>
              <tr>
                <th>Rang</th>
                <th>Paire</th>
                <th>Volatilité Event</th>
                <th>Vol. Baseline</th>
                <th>Multiplicateur</th>
                <th>Direction</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(pair, index) in eventImpact.pair_impacts" :key="pair.symbol" :class="{ 'top-pair': index === 0 }">
                <td>
                  <span v-if="index === 0" class="rank-badge rank-1">🥇 #1</span>
                  <span v-else-if="index === 1" class="rank-badge rank-2">🥈 #2</span>
                  <span v-else-if="index === 2" class="rank-badge rank-3">🥉 #3</span>
                  <span v-else class="rank-badge">#{{ index + 1 }}</span>
                </td>
                <td class="pair-name">{{ pair.symbol }}</td>
                <td class="volatility">{{ pair.event_volatility }} pips</td>
                <td class="baseline">{{ pair.baseline_volatility }} pips</td>
                <td class="multiplier">
                  <span class="multiplier-value" :class="getMultiplierClass(pair.multiplier)">
                    ×{{ pair.multiplier.toFixed(1) }}
                  </span>
                </td>
                <td>
                  <span class="direction-badge" :class="`dir-${pair.direction}`">
                    {{ getDirectionIcon(pair.direction) }} {{ pair.direction }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Observations -->
        <div class="observations-card">
          <h3>💡 Observations</h3>
          <ul>
            <li v-for="(obs, index) in eventImpact.observations" :key="index">{{ obs }}</li>
          </ul>
        </div>
      </div>
    </template>

    <!-- Vue 2: Par Paire -->
    <template v-if="viewMode === 'by-pair'">
      <div v-if="loadingPair" class="loading">
        <div class="spinner"></div>
        <p>Chargement de l'historique...</p>
      </div>

      <div v-if="pairHistory && !loadingPair" class="pair-history-results">
        <!-- En-tête paire -->
        <div class="pair-header-card">
          <h3>💱 {{ pairHistory.symbol }}</h3>
          <p>Historique des réactions aux événements ({{ pairHistory.period }})</p>
        </div>

        <!-- Statistiques agrégées -->
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-icon">📊</div>
            <div class="stat-value">{{ pairHistory.total_events }}</div>
            <div class="stat-label">Événements analysés</div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">📈</div>
            <div class="stat-value">{{ pairHistory.avg_volatility }} pips</div>
            <div class="stat-label">Volatilité moyenne</div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">🎯</div>
            <div class="stat-value">{{ pairHistory.max_volatility }} pips</div>
            <div class="stat-label">Impact maximum</div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">⚡</div>
            <div class="stat-value">×{{ pairHistory.avg_multiplier }}</div>
            <div class="stat-label">Multiplicateur moyen</div>
          </div>
        </div>

        <!-- Tableau historique -->
        <div class="history-table-container">
          <h3>📅 Historique détaillé</h3>
          
          <table class="history-table">
            <thead>
              <tr>
                <th>Date</th>
                <th>Événement</th>
                <th>Impact</th>
                <th>Volatilité</th>
                <th>vs Baseline</th>
                <th>Direction</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="event in pairHistory.events" :key="event.event_id">
                <td class="date-cell">{{ formatDate(event.datetime) }}</td>
                <td class="event-name">{{ event.event_name }}</td>
                <td>
                  <span class="impact-badge" :class="`impact-${event.impact.toLowerCase()}`">
                    {{ event.impact }}
                  </span>
                </td>
                <td class="volatility">{{ event.volatility }} pips</td>
                <td class="percentage-change" :class="getChangeClass(event.change_percent)">
                  +{{ event.change_percent }}%
                </td>
                <td>
                  <span class="direction-badge" :class="`dir-${event.direction}`">
                    {{ getDirectionIcon(event.direction) }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Top événements impactants -->
        <div class="top-events-card">
          <h3>🔥 Top 5 événements les plus impactants</h3>
          <div class="top-events-list">
            <div v-for="(event, index) in pairHistory.top_events" :key="index" class="top-event-item">
              <span class="rank">{{ index + 1 }}.</span>
              <span class="event-name">{{ event.name }}</span>
              <span class="event-date">({{ formatDate(event.datetime) }})</span>
              <span class="event-volatility">→ {{ event.volatility }} pips</span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Vue 3: Heatmap -->
    <template v-if="viewMode === 'heatmap'">
      <div class="heatmap-container">
        <h3>🔥 Heatmap Événements × Paires</h3>
        <p class="heatmap-subtitle">Volatilité moyenne mesurée (pips) - {{ heatmapData?.period || 'Chargement...' }}</p>

        <div v-if="loadingHeatmap" class="loading">
          <div class="spinner"></div>
          <p>Génération de la heatmap...</p>
        </div>

        <div v-if="heatmapData && !loadingHeatmap" class="heatmap-grid">
          <table class="heatmap-table">
            <thead>
              <tr>
                <th class="header-corner">Type d'événement</th>
                <th v-for="pair in heatmapData.pairs" :key="pair">{{ pair }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="eventType in heatmapData.event_types" :key="eventType.name">
                <td class="event-type-cell">
                  <div class="event-type-name">{{ eventType.name }}</div>
                  <div class="event-count">({{ eventType.count }} evt)</div>
                </td>
                <td 
                  v-for="pair in heatmapData.pairs" 
                  :key="`${eventType.name}-${pair}`"
                  class="heatmap-cell"
                  :class="getHeatmapClass(getHeatmapValue(eventType.name, pair))"
                  :title="`${eventType.name} → ${pair}: ${getHeatmapValue(eventType.name, pair)} pips`"
                >
                  <span class="cell-value">{{ getHeatmapValue(eventType.name, pair) }}</span>
                </td>
              </tr>
            </tbody>
          </table>

          <!-- Légende -->
          <div class="heatmap-legend">
            <div class="legend-title">Légende :</div>
            <div class="legend-items">
              <div class="legend-item">
                <div class="legend-color heat-very-high"></div>
                <span>>500 pips</span>
              </div>
              <div class="legend-item">
                <div class="legend-color heat-high"></div>
                <span>200-500 pips</span>
              </div>
              <div class="legend-item">
                <div class="legend-color heat-medium"></div>
                <span>100-200 pips</span>
              </div>
              <div class="legend-item">
                <div class="legend-color heat-low"></div>
                <span>50-100 pips</span>
              </div>
              <div class="legend-item">
                <div class="legend-color heat-very-low"></div>
                <span><50 pips</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Message de bienvenue -->
    <div v-if="!selectedEventId && !selectedPairSymbol && viewMode !== 'heatmap'" class="welcome">
      <div class="welcome-icon">🎯</div>
      <h3>Analyse Rétrospective</h3>
      <p class="info-text">
        Explorez l'impact historique des événements économiques sur les différentes paires.<br>
        Sélectionnez un mode de vue pour commencer l'analyse.
      </p>
    </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDataRefresh } from '../composables/useDataRefresh'

// Types
interface PastEvent {
  id: number
  name: string
  datetime: string
  country: string
  currency: string
  impact: string
}

interface PairImpact {
  symbol: string
  event_volatility: number
  baseline_volatility: number
  multiplier: number
  direction: string
}

interface EventImpactResult {
  event_id: number
  event_name: string
  datetime: string
  country: string
  currency: string
  impact: string
  window_start: string
  window_end: string
  pair_impacts: PairImpact[]
  observations: string[]
}

interface PairEventHistory {
  symbol: string
  period: string
  total_events: number
  avg_volatility: number
  max_volatility: number
  avg_multiplier: number
  events: any[]
  top_events: any[]
}

interface HeatmapData {
  period: string
  pairs: string[]
  event_types: { name: string; count: number }[]
  data: { [key: string]: { [key: string]: number } }
}

// État
const viewMode = ref<'by-event' | 'by-pair' | 'heatmap'>('by-event')
const selectedEventId = ref<string>('')
const selectedPairSymbol = ref<string>('')
const pastEvents = ref<PastEvent[]>([])
const availablePairs = ref<string[]>([])
const eventImpact = ref<EventImpactResult | null>(null)
const pairHistory = ref<PairEventHistory | null>(null)
const heatmapData = ref<HeatmapData | null>(null)
const loadingEvent = ref(false)
const loadingPair = ref(false)
const loadingHeatmap = ref(false)

// Watch pour charger automatiquement la heatmap quand on switch
watch(viewMode, (newMode) => {
  if (newMode === 'heatmap' && !heatmapData.value) {
    loadHeatmap()
  }
})

// Fonctions
const { onPairDataRefresh } = useDataRefresh()

onMounted(async () => {
  await loadPastEvents()
  await loadAvailablePairs()
  
  // S'abonner aux événements de rafraîchissement
  const unsubscribe = onPairDataRefresh(loadAvailablePairs)
  
  // Se désabonner au démontage
  onBeforeUnmount(unsubscribe)
})

function switchViewMode(mode: 'by-event' | 'by-pair' | 'heatmap') {
  viewMode.value = mode
  // Reset selections when switching modes
  if (mode !== 'by-event') {
    selectedEventId.value = ''
    eventImpact.value = null
  }
  if (mode !== 'by-pair') {
    selectedPairSymbol.value = ''
    pairHistory.value = null
  }
}

async function loadPastEvents() {
  try {
    // Charger les vrais événements depuis la base de données
    pastEvents.value = await invoke<PastEvent[]>('get_past_events', {
      monthsBack: 6
    })
  } catch (error) {
    console.error('Erreur chargement événements:', error)
    pastEvents.value = []
  }
}

async function loadAvailablePairs() {
  try {
    // Charger les paires depuis le backend (comme dans SymbolSelector)
    const symbolData = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
    // Extraire uniquement les noms des symboles
    availablePairs.value = symbolData.map(item => item.symbol)
  } catch (error) {
    console.error('Erreur chargement paires:', error)
    // Fallback sur des paires par défaut en cas d'erreur
    availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
  }
}

async function loadEventImpact() {
  if (!selectedEventId.value) return
  
  loadingEvent.value = true
  try {
    // Charger les vraies données depuis le backend
    eventImpact.value = await invoke<EventImpactResult>('get_event_impact_by_pair', { 
      eventId: parseInt(selectedEventId.value) 
    })
  } catch (error) {
    console.error('Erreur analyse événement:', error)
    eventImpact.value = null
  } finally {
    loadingEvent.value = false
  }
}

async function loadPairEventHistory() {
  if (!selectedPairSymbol.value) return
  
  loadingPair.value = true
  try {
    // Charger les vraies données depuis le backend
    pairHistory.value = await invoke<PairEventHistory>('get_pair_event_history', {
      pairSymbol: selectedPairSymbol.value,
      monthsBack: 6
    })
  } catch (error) {
    console.error('Erreur historique paire:', error)
    pairHistory.value = null
  } finally {
    loadingPair.value = false
  }
}

async function loadHeatmap() {
  loadingHeatmap.value = true
  try {
    // Charger les vraies données depuis le backend
    heatmapData.value = await invoke<HeatmapData>('get_correlation_heatmap', {
      monthsBack: 6
    })
  } catch (error) {
    console.error('Erreur heatmap:', error)
    // En cas d'erreur, afficher un message à l'utilisateur
    heatmapData.value = {
      period: 'Données non disponibles',
      pairs: [],
      event_types: [],
      data: {}
    }
  } finally {
    loadingHeatmap.value = false
  }
}

function formatDate(datetime: string): string {
  return new Date(datetime).toLocaleDateString('fr-FR', { 
    day: '2-digit', 
    month: '2-digit', 
    year: 'numeric' 
  })
}

function formatDateTime(datetime: string): string {
  return new Date(datetime).toLocaleString('fr-FR', { 
    day: '2-digit', 
    month: '2-digit', 
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function getDirectionIcon(direction: string): string {
  if (direction === 'HAUSSIER') return '📈'
  if (direction === 'BAISSIER') return '📉'
  return '➡️'
}

function getMultiplierClass(multiplier: number): string {
  if (multiplier >= 50) return 'mult-extreme'
  if (multiplier >= 20) return 'mult-very-high'
  if (multiplier >= 10) return 'mult-high'
  if (multiplier >= 5) return 'mult-medium'
  return 'mult-low'
}

function getChangeClass(percent: number): string {
  if (percent >= 1000) return 'change-extreme'
  if (percent >= 500) return 'change-very-high'
  if (percent >= 200) return 'change-high'
  return 'change-medium'
}

function getHeatmapValue(eventType: string, pair: string): number {
  return heatmapData.value?.data[eventType]?.[pair] || 0
}

function getHeatmapClass(value: number): string {
  if (value >= 500) return 'heat-very-high'
  if (value >= 200) return 'heat-high'
  if (value >= 100) return 'heat-medium'
  if (value >= 50) return 'heat-low'
  return 'heat-very-low'
}
</script>

<style scoped>
/* Bloc principal - IDENTIQUE À APP.VUE */
.main-container {
  background: #161b22;
  border-radius: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #30363d;
  overflow: hidden;
  color: #e2e8f0;
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
  gap: 12px;
  min-width: 300px;
}

.pair-label {
  font-weight: 600;
  color: #e6edf3;
  font-size: 0.95em;
  white-space: nowrap;
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

.inline-event-select {
  width: 100%;
  padding: 10px 14px;
  font-size: 0.95em;
  border: 2px solid #30363d;
  border-radius: 8px;
  background: #ffffff;
  color: #000000;
  cursor: pointer;
  transition: all 0.3s;
}

.inline-event-select:hover:not(:disabled) {
  border-color: #58a6ff;
  background: #f8f9fa;
}

.inline-event-select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.2);
}

.inline-event-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inline-event-select option {
  background: #ffffff;
  color: #000000;
}

.inline-pair-select {
  width: 100%;
  padding: 10px 14px;
  font-size: 0.95em;
  border: 2px solid #30363d;
  border-radius: 8px;
  background: #ffffff;
  color: #000000;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 250px;
}

.inline-pair-select:hover:not(:disabled) {
  border-color: #58a6ff;
  background: #f8f9fa;
}

.inline-pair-select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.2);
}

.inline-pair-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inline-pair-select option {
  background: #ffffff;
  color: #000000;
}

/* Mode selector */
.view-mode-selector {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 2px solid #30363d;
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

.mode-button:hover {
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

/* Zone de contenu */
.content-area {
  padding: 30px;
  min-height: 400px;
}

/* Selectors */
.event-selector-container,
.pair-selector-container {
  background: #1a202c;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 30px;
  border: 1px solid #2d3748;
}

.event-selector-container label,
.pair-selector-container label {
  display: block;
  color: #e2e8f0;
  font-weight: 600;
  margin-bottom: 10px;
}

.event-select,
.pair-select {
  width: 100%;
  padding: 12px 16px;
  font-size: 1.1em;
  border: 2px solid #4a5568;
  border-radius: 8px;
  background: #2d3748;
  color: #e2e8f0;
  cursor: pointer;
  transition: all 0.3s;
}

.event-select:hover,
.pair-select:hover {
  border-color: #667eea;
  background: #374151;
}

.event-select:focus,
.pair-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.event-select option,
.pair-select option {
  background: #2d3748;
  color: #e2e8f0;
}

/* Loading */
.loading {
  text-align: center;
  padding: 60px 20px;
  color: #e2e8f0;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #2d3748;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Event Impact Results */
.event-impact-results {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.event-info-card {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.event-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.event-header h3 {
  color: #e2e8f0;
  font-size: 1.5em;
}

.event-badge {
  padding: 6px 12px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.9em;
}

.impact-high {
  background: #dc2626;
  color: white;
}

.impact-medium {
  background: #f59e0b;
  color: white;
}

.impact-low {
  background: #3b82f6;
  color: white;
}

.event-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.detail-item {
  display: flex;
  gap: 10px;
}

.detail-item .label {
  color: #a0aec0;
  font-weight: 600;
}

.detail-item .value {
  color: #e2e8f0;
}

/* Impact Ranking */
.impact-ranking {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.impact-ranking h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.impact-table {
  width: 100%;
  border-collapse: collapse;
}

.impact-table thead {
  background: #2d3748;
}

.impact-table th {
  padding: 12px;
  text-align: left;
  font-weight: 600;
  color: #e2e8f0;
  border-bottom: 2px solid #4a5568;
}

.impact-table td {
  padding: 15px 12px;
  border-bottom: 1px solid #2d3748;
  color: #e2e8f0;
}

.impact-table tbody tr:hover {
  background: #2d3748;
}

.impact-table tbody tr.top-pair {
  background: #1e3a5f;
}

.pair-name {
  font-weight: 600;
  font-size: 1.05em;
}

.volatility {
  font-weight: 600;
  color: #818cf8;
}

.multiplier-value {
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
}

.mult-extreme {
  background: #7f1d1d;
  color: #fca5a5;
}

.mult-very-high {
  background: #dc2626;
  color: white;
}

.mult-high {
  background: #f59e0b;
  color: white;
}

.mult-medium {
  background: #3b82f6;
  color: white;
}

.mult-low {
  background: #4a5568;
  color: #e2e8f0;
}

.direction-badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.9em;
  font-weight: 600;
}

.dir-HAUSSIER {
  background: #065f46;
  color: #6ee7b7;
}

.dir-BAISSIER {
  background: #7f1d1d;
  color: #fca5a5;
}

.dir-NEUTRE {
  background: #4a5568;
  color: #e2e8f0;
}

.rank-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-weight: 700;
  font-size: 0.9em;
}

.rank-1 {
  background: #ffd700;
  color: #744210;
}

.rank-2 {
  background: #c0c0c0;
  color: #2d3748;
}

.rank-3 {
  background: #cd7f32;
  color: white;
}

/* Observations */
.observations-card {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.observations-card h3 {
  color: #e2e8f0;
  margin-bottom: 15px;
}

.observations-card ul {
  list-style: none;
  padding: 0;
}

.observations-card li {
  padding: 10px;
  margin-bottom: 8px;
  background: #2d3748;
  border-left: 3px solid #667eea;
  border-radius: 4px;
  color: #e2e8f0;
}

/* Pair History */
.pair-history-results {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.pair-header-card {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
  text-align: center;
}

.pair-header-card h3 {
  color: #e2e8f0;
  font-size: 2em;
  margin-bottom: 10px;
}

.pair-header-card p {
  color: #a0aec0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.stat-card {
  background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  padding: 20px;
  border-radius: 10px;
  text-align: center;
  border: 1px solid #4a5568;
  transition: transform 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.4);
}

.stat-icon {
  font-size: 2.5em;
  margin-bottom: 10px;
}

.stat-value {
  font-size: 2em;
  font-weight: bold;
  color: #e2e8f0;
  margin: 10px 0;
}

.stat-label {
  color: #a0aec0;
  font-size: 0.95em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.history-table-container {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.history-table-container h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
}

.history-table {
  width: 100%;
  border-collapse: collapse;
}

.history-table thead {
  background: #2d3748;
}

.history-table th {
  padding: 12px;
  text-align: left;
  font-weight: 600;
  color: #e2e8f0;
  border-bottom: 2px solid #4a5568;
}

.history-table td {
  padding: 12px;
  border-bottom: 1px solid #2d3748;
  color: #e2e8f0;
}

.history-table tbody tr:hover {
  background: #2d3748;
}

.date-cell {
  font-size: 0.95em;
}

.event-name {
  font-weight: 600;
}

.percentage-change {
  font-weight: 700;
}

.change-extreme {
  color: #dc2626;
}

.change-very-high {
  color: #f59e0b;
}

.change-high {
  color: #10b981;
}

.change-medium {
  color: #3b82f6;
}

.top-events-card {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.top-events-card h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
}

.top-events-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.top-event-item {
  background: #2d3748;
  padding: 15px;
  border-radius: 8px;
  display: flex;
  gap: 10px;
  align-items: center;
  color: #e2e8f0;
}

.top-event-item .rank {
  font-weight: 700;
  color: #667eea;
  font-size: 1.2em;
}

.top-event-item .event-name {
  flex: 1;
  font-weight: 600;
}

.top-event-item .event-date {
  color: #a0aec0;
  font-size: 0.9em;
}

.top-event-item .event-volatility {
  font-weight: 700;
  color: #818cf8;
}

/* Heatmap */
.heatmap-container {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.heatmap-container h3 {
  color: #e2e8f0;
  margin-bottom: 10px;
  font-size: 1.8em;
}

.heatmap-subtitle {
  color: #a0aec0;
  margin-bottom: 30px;
}

.heatmap-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 30px;
}

.header-corner {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 700;
  padding: 15px;
  border: 1px solid #4a5568;
}

.heatmap-table th {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 600;
  padding: 12px;
  border: 1px solid #4a5568;
  text-align: center;
}

.event-type-cell {
  background: #2d3748;
  padding: 12px;
  border: 1px solid #4a5568;
  text-align: left;
}

.event-type-name {
  font-weight: 700;
  color: #e2e8f0;
  margin-bottom: 4px;
}

.event-count {
  font-size: 0.85em;
  color: #a0aec0;
}

.heatmap-cell {
  padding: 15px;
  text-align: center;
  border: 1px solid #4a5568;
  cursor: help;
  transition: all 0.2s;
}

.heatmap-cell:hover {
  transform: scale(1.05);
  box-shadow: 0 0 10px rgba(102, 126, 234, 0.5);
}

.cell-value {
  font-weight: 700;
  font-size: 1.1em;
}

.heat-very-high {
  background: #7f1d1d;
  color: #fca5a5;
}

.heat-high {
  background: #dc2626;
  color: white;
}

.heat-medium {
  background: #f59e0b;
  color: white;
}

.heat-low {
  background: #3b82f6;
  color: white;
}

.heat-very-low {
  background: #4a5568;
  color: #e2e8f0;
}

.heatmap-legend {
  display: flex;
  gap: 20px;
  align-items: center;
  padding: 20px;
  background: #2d3748;
  border-radius: 8px;
}

.legend-title {
  font-weight: 700;
  color: #e2e8f0;
}

.legend-items {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e2e8f0;
}

.legend-color {
  width: 30px;
  height: 20px;
  border-radius: 4px;
  border: 1px solid #4a5568;
}

/* Welcome */
.welcome {
  text-align: center;
  padding: 60px 20px;
  background: #1a202c;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.welcome-icon {
  font-size: 4em;
  margin-bottom: 20px;
}

.welcome h3 {
  font-size: 1.8em;
  color: #e2e8f0;
  margin-bottom: 15px;
}

.info-text {
  font-size: 1.1em;
  color: #a0aec0;
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
}
</style>
