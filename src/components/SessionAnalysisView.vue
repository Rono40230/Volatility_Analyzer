<template>
  <div class="main-container">
    <div class="header-section">
      <div class="header-left">
        <h2 class="main-title">
          <span class="icon">📍</span>
          Analyse Volatilité par Sessions Forex
        </h2>
        <p class="main-subtitle">
          Analyse rétrospective de la volatilité selon les sessions de trading (horaires de Paris)
        </p>
      </div>
      <div class="header-right">
        <select 
          id="session-symbol" 
          v-model="selectedSymbol" 
          @change="analyzeSymbol"
          class="inline-symbol-select"
        >
          <option value="" style="color: #000000;">Choisir une paire</option>
          <option v-for="symbolInfo in symbols" :key="symbolInfo.symbol" :value="symbolInfo.symbol">
            {{ symbolInfo.symbol }}
          </option>
        </select>
      </div>
    </div>

    <!-- Contenu -->
    <div class="content-area">
      <!-- Loading -->
      <div v-if="loading" class="loading">
        <div class="spinner"></div>
        <p>Analyse des sessions en cours...</p>
      </div>

      <!-- Erreur -->
      <div v-if="error" class="error">
        <h3>❌ Erreur</h3>
        <p>{{ error }}</p>
      </div>

      <!-- Message de bienvenue -->
      <div v-if="!selectedSymbol && !loading && !error" class="welcome">
        <div class="welcome-icon">🌍</div>
        <h3>Sélectionnez une paire pour analyser sa volatilité par session</h3>
        <p class="info-text">
          Cette analyse montre comment la volatilité historique de chaque paire 
          varie selon les sessions de trading mondiales (Sydney, Tokyo, Londres, New York).
        </p>
      </div>

      <!-- Résultats -->
      <div v-if="sessionData && !loading">
          <!-- Info générale -->
          <div class="general-info">
            <h3>📊 {{ selectedSymbol }} - Analyse complète</h3>
            <div class="info-grid">
              <div class="info-card">
                <div class="info-icon">📅</div>
                <div class="info-value">{{ sessionData.period }}</div>
                <div class="info-label">Période analysée</div>
              </div>
              <div class="info-card">
                <div class="info-icon">📊</div>
                <div class="info-value">{{ sessionData.total_candles.toLocaleString() }}</div>
                <div class="info-label">Total de bougies</div>
              </div>
              <div class="info-card">
                <div class="info-icon">📈</div>
                <div class="info-value">{{ sessionData.avg_daily_volatility }} pips</div>
                <div class="info-label">Volatilité moyenne quotidienne</div>
              </div>
            </div>
          </div>

          <!-- Tableau des sessions -->
          <div class="session-table-container">
            <h3>⏰ Distribution de la volatilité par session (Heure de Paris)</h3>
            <table class="session-table">
              <thead>
                <tr>
                  <th>Session</th>
                  <th>Horaire Paris</th>
                  <th>Volatilité moy.</th>
                  <th>% du total</th>
                  <th>Nb bougies</th>
                  <th>Rang</th>
                </tr>
              </thead>
              <tbody>
                <tr 
                  v-for="(session, index) in sessionData.sessions" 
                  :key="session.name"
                  :class="{ 'best-session': index === 0 }"
                >
                  <td class="session-name">
                    <span class="session-icon">{{ session.icon }}</span>
                    {{ session.name }}
                  </td>
                  <td>{{ session.paris_hours }}</td>
                  <td class="volatility">{{ session.avg_volatility }} pips</td>
                  <td>
                    <div class="percentage-bar">
                      <div 
                        class="bar-fill" 
                        :style="{ width: session.percentage + '%' }"
                      ></div>
                      <span class="percentage-text">{{ session.percentage }}%</span>
                    </div>
                  </td>
                  <td>{{ session.candle_count.toLocaleString() }}</td>
                  <td class="rank">
                    <span class="rank-badge" :class="`rank-${index + 1}`">
                      #{{ index + 1 }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Heatmap des chevauchements -->
          <div class="overlap-section" v-if="sessionData.overlaps && sessionData.overlaps.length > 0">
            <h3>🔥 Chevauchements de sessions (zones de forte volatilité)</h3>
            <div class="overlap-grid">
              <div 
                v-for="overlap in sessionData.overlaps" 
                :key="overlap.name"
                class="overlap-card"
                :class="{ 'high-volatility': overlap.volatility_multiplier > 2 }"
              >
                <div class="overlap-name">{{ overlap.name }}</div>
                <div class="overlap-hours">{{ overlap.paris_hours }}</div>
                <div class="overlap-stat">
                  <span class="stat-label">Volatilité :</span>
                  <span class="stat-value">{{ overlap.avg_volatility }} pips</span>
                </div>
                <div class="overlap-multiplier">
                  <span class="multiplier-badge">
                    {{ overlap.volatility_multiplier }}x la moyenne
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Corrélation avec événements économiques -->
          <div class="calendar-correlation" v-if="sessionData.calendar_correlation">
            <h3>📅 Corrélation avec événements économiques</h3>
            <div class="correlation-grid">
              <div 
                v-for="corr in sessionData.calendar_correlation" 
                :key="corr.session"
                class="correlation-card"
              >
                <div class="corr-session">{{ corr.session }}</div>
                <div class="corr-stat">
                  <span class="stat-label">Événements HIGH IMPACT :</span>
                  <span class="stat-value">{{ corr.high_impact_events }}</span>
                </div>
                <div class="corr-stat">
                  <span class="stat-label">Volatilité lors d'événements :</span>
                  <span class="stat-value">{{ corr.event_volatility }} pips</span>
                </div>
                <div class="corr-stat">
                  <span class="stat-label">Impact moyen :</span>
                  <span class="stat-value impact">+{{ corr.impact_percentage }}%</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Recommandations -->
          <div class="recommendations">
            <h3>💡 Recommandations basées sur l'historique</h3>
            <div class="recommendation-list">
              <div 
                v-for="(rec, index) in sessionData.recommendations" 
                :key="index"
                class="recommendation-card"
                :class="rec.type"
              >
                <div class="rec-icon">{{ rec.icon }}</div>
                <div class="rec-content">
                  <div class="rec-title">{{ rec.title }}</div>
                  <div class="rec-description">{{ rec.description }}</div>
                </div>
              </div>
            </div>
          </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDataRefresh } from '../composables/useDataRefresh'

interface SessionStats {
  name: string
  icon: string
  paris_hours: string
  avg_volatility: number
  percentage: number
  candle_count: number
}

interface OverlapStats {
  name: string
  paris_hours: string
  avg_volatility: number
  volatility_multiplier: number
}

interface CalendarCorrelation {
  session: string
  high_impact_events: number
  event_volatility: number
  impact_percentage: number
}

interface Recommendation {
  icon: string
  type: 'positive' | 'warning' | 'info'
  title: string
  description: string
}

interface SessionAnalysisResult {
  period: string
  total_candles: number
  avg_daily_volatility: number
  sessions: SessionStats[]
  overlaps?: OverlapStats[]
  calendar_correlation?: CalendarCorrelation[]
  recommendations: Recommendation[]
}

interface SymbolInfo {
  symbol: string
  file_path: string
}

const symbols = ref<SymbolInfo[]>([])
const selectedSymbol = ref<string>('')
const sessionData = ref<SessionAnalysisResult | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

const { onPairDataRefresh } = useDataRefresh()

async function loadSymbols() {
  try {
    symbols.value = await invoke<SymbolInfo[]>('load_symbols')
  } catch (e) {
    console.error('Erreur lors du chargement des symboles:', e)
    error.value = 'Impossible de charger la liste des paires'
  }
}

onMounted(async () => {
  await loadSymbols()
  
  // S'abonner aux événements de rafraîchissement
  const unsubscribe = onPairDataRefresh(loadSymbols)
  
  // Se désabonner au démontage
  onBeforeUnmount(unsubscribe)
})

async function analyzeSymbol() {
  if (!selectedSymbol.value) {
    sessionData.value = null
    return
  }

  loading.value = true
  error.value = null
  sessionData.value = null

  try {
    const result = await invoke<SessionAnalysisResult>('analyze_sessions', {
      pairSymbol: selectedSymbol.value
    })
    sessionData.value = result
  } catch (e: any) {
    console.error('Erreur lors de l\'analyse des sessions:', e)
    error.value = e.toString()
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* Structure harmonisée : bloc principal */
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

/* Zone de contenu */
.content-area {
  padding: 30px;
  min-height: 400px;
}/* Zone de contenu */
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

/* Ancien h2 et subtitle (legacy) */
.session-analysis h2 {
  color: #e2e8f0;
  font-size: 2em;
  margin-bottom: 10px;
}

.subtitle {
  color: #a0aec0;
  font-size: 1.1em;
  margin-bottom: 30px;
}

.symbol-selector-container {
  background: #1a202c;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  margin-bottom: 30px;
  border: 1px solid #2d3748;
}

.symbol-selector-container label {
  display: block;
  color: #e2e8f0;
  font-weight: 600;
  margin-bottom: 10px;
}

.symbol-select {
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

.symbol-select:hover {
  border-color: #667eea;
  background: #374151;
}

.symbol-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.symbol-select option {
  background: #2d3748;
  color: #e2e8f0;
}

.loading {
  text-align: center;
  padding: 60px 20px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error {
  background: #7f1d1d;
  border: 2px solid #dc2626;
  border-radius: 8px;
  padding: 20px;
  margin: 20px 0;
  color: #fca5a5;
}

.error h3 {
  color: #fca5a5;
}

.welcome {
  text-align: center;
  padding: 60px 20px;
  background: #1a202c;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0,0,0,0.3);
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

.results {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.general-info {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.general-info h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.info-card {
  background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  padding: 20px;
  border-radius: 10px;
  text-align: center;
  border: 1px solid #4a5568;
  transition: transform 0.2s;
}

.info-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.4);
}

.info-icon {
  font-size: 2.5em;
  margin-bottom: 10px;
}

.info-value {
  font-size: 2em;
  font-weight: bold;
  color: #e2e8f0;
  margin: 10px 0;
}

.info-label {
  color: #a0aec0;
  font-size: 0.95em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.session-table-container {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.session-table-container h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}


.session-table-container {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.session-table-container h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.session-table {
  width: 100%;
  border-collapse: collapse;
}

.session-table thead {
  background: #2d3748;
}

.session-table th {
  padding: 12px;
  text-align: left;
  font-weight: 600;
  color: #e2e8f0;
  border-bottom: 2px solid #4a5568;
}

.session-table td {
  padding: 15px 12px;
  border-bottom: 1px solid #2d3748;
  color: #e2e8f0;
}

.session-table tbody tr:hover {
  background: #2d3748;
}

.session-table tbody tr.best-session {
  background: #1e3a5f;
  font-weight: 600;
}

.session-name {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
}

.session-icon {
  font-size: 1.3em;
}

.volatility {
  font-weight: 600;
  color: #667eea;
}

.percentage-bar {
  position: relative;
  width: 100%;
  height: 30px;
  background: #2d3748;
  border-radius: 6px;
  overflow: hidden;
}

.bar-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  transition: width 0.5s;
}

.percentage-text {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-weight: 600;
  color: #e2e8f0;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
  z-index: 1;
}

.rank-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 12px;
  font-weight: 700;
  font-size: 0.9em;
}

.rank-badge.rank-1 {
  background: #ffd700;
  color: #744210;
}

.rank-badge.rank-2 {
  background: #c0c0c0;
  color: #2d3748;
}

.rank-badge.rank-3 {
  background: #cd7f32;
  color: white;
}

.rank-badge.rank-4 {
  background: #4a5568;
  color: #e2e8f0;
}

.overlap-section {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.overlap-section h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.overlap-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.overlap-card {
  background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  padding: 20px;
  border-radius: 10px;
  border: 2px solid #4a5568;
  transition: all 0.3s;
}

.overlap-card.high-volatility {
  background: linear-gradient(135deg, #7f1d1d 0%, #991b1b 100%);
  border-color: #dc2626;
}

.overlap-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
}

.overlap-name {
  font-weight: 700;
  font-size: 1.2em;
  color: #e2e8f0;
  margin-bottom: 8px;
}

.overlap-hours {
  color: #a0aec0;
  margin-bottom: 15px;
  font-size: 0.95em;
}

.overlap-stat {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.stat-label {
  color: #a0aec0;
}

.stat-value {
  font-weight: 600;
  color: #818cf8;
}

.overlap-multiplier {
  margin-top: 15px;
}

.multiplier-badge {
  display: inline-block;
  background: #667eea;
  color: white;
  padding: 6px 12px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.9em;
}

.calendar-correlation {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.calendar-correlation h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.correlation-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
}

.correlation-card {
  background: #2d3748;
  padding: 20px;
  border-radius: 10px;
  border-left: 4px solid #667eea;
}

.corr-session {
  font-weight: 700;
  font-size: 1.2em;
  color: #e2e8f0;
  margin-bottom: 15px;
}

.corr-stat {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
  padding: 8px 0;
  border-bottom: 1px solid #4a5568;
}

.corr-stat:last-child {
  border-bottom: none;
}

.stat-value.impact {
  color: #48bb78;
  font-weight: 700;
}

.recommendations {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid #2d3748;
}

.recommendations h3 {
  color: #e2e8f0;
  margin-bottom: 20px;
  font-size: 1.5em;
}

.recommendation-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.recommendation-card {
  display: flex;
  align-items: flex-start;
  gap: 15px;
  padding: 20px;
  border-radius: 10px;
  border-left: 4px solid;
  background: #2d3748;
}

.recommendation-card.positive {
  border-color: #48bb78;
  background: linear-gradient(135deg, #1a4d2e 0%, #2d3748 100%);
}

.recommendation-card.warning {
  border-color: #f59e0b;
  background: linear-gradient(135deg, #78350f 0%, #2d3748 100%);
}

.recommendation-card.info {
  border-color: #4299e1;
  background: linear-gradient(135deg, #1e3a8a 0%, #2d3748 100%);
}

.rec-icon {
  font-size: 2em;
  flex-shrink: 0;
}

.rec-content {
  flex: 1;
}

.rec-title {
  font-weight: 700;
  font-size: 1.1em;
  color: #e2e8f0;
  margin-bottom: 8px;
}

.rec-description {
  color: #a0aec0;
  line-height: 1.5;
}
</style>
