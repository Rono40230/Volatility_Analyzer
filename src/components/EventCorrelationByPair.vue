<template>
  <div v-if="loading" class="loading">
    <div class="spinner"></div>
    <p>Analyse de la corrélation par paire...</p>
  </div>

  <!-- Message de bienvenue avec sélecteur de paire -->
  <div v-if="!selectedPair && !loading" class="welcome">
    <div class="welcome-icon">💱</div>
    <h3>Analyse Rétrospective par Paire</h3>
    <div class="welcome-select-container">
      <label for="pair-select">Sélectionnez une paire :</label>
      <select 
        id="pair-select" 
        v-model="selectedPair" 
        class="welcome-symbol-select"
        @change="loadPairCorrelation"
      >
        <option value="">Choisir une paire</option>
        <option v-for="pair in availablePairs" :key="pair" :value="pair">
          {{ pair }}
        </option>
      </select>
    </div>
  </div>

  <!-- Résultats de corrélation par paire -->
  <div v-if="pairCorrelation && !loading" class="pair-correlation-results">
    <!-- En-tête avec sélecteur INLINE -->
    <div class="pair-info-card">
      <div class="pair-header">
        <h3>{{ selectedPair }}</h3>
        <select 
          v-model="selectedPair" 
          class="inline-select"
          @change="loadPairCorrelation"
        >
          <option value="">Changer de paire</option>
          <option v-for="pair in availablePairs" :key="pair" :value="pair">
            {{ pair }}
          </option>
        </select>
      </div>
    </div>

    <!-- Tableau des événements impactant cette paire -->
    <div class="correlation-table-container">
      <table class="correlation-table">
        <thead>
          <tr>
            <th>Rang</th>
            <th>Événement</th>
            <th colspan="3" style="text-align: center;">
              <MetricTooltip title="Volatilité observée">
                <span style="cursor: help; border-bottom: 1px dotted #58a6ff;">Volatilité observée (pips)</span>
                <template #definition>
                  <div class="tooltip-section-title">Définition</div>
                  <div class="tooltip-section-text">Amplitude en pips (mouvements de prix) observée avant et après l'événement économique.</div>
                </template>
                <template #usage>
                  <div class="tooltip-section-title">Mesure</div>
                  <div class="tooltip-section-text"><strong>-30mn:</strong> Volatilité durant les 30 minutes précédant l'événement | <strong>+30mn:</strong> Volatilité durant les 30 minutes suivant l'événement | <strong>1h total:</strong> Volatilité totale sur l'heure complète</div>
                </template>
                <template #scoring>
                  <div class="tooltip-section-title">Interprétation</div>
                  <div class="tooltip-section-text">Plus élevé = Plus de mouvement. Permet d'identifier si l'événement a réellement provoqué une augmentation de la volatilité par rapport à la baseline.</div>
                </template>
              </MetricTooltip>
            </th>
            <th>
              <MetricTooltip title="Score de Corrélation">
                <span style="cursor: help; border-bottom: 1px dotted #58a6ff;">Score</span>
                <template #definition>
                  <div class="tooltip-section-title">Définition</div>
                  <div class="tooltip-section-text">Score composite (0-100%) mesurant l'impact de l'événement sur la volatilité de la paire. Combine volatilité moyenne, changement d'impact et récurrence.</div>
                </template>
                <template #usage>
                  <div class="tooltip-section-title">Composants du Score</div>
                  <div class="tooltip-section-text"><strong>Volatilité (max 60%):</strong> Amplitude moyenne observée | <strong>Impact (max 25%):</strong> Changement avant/après l'événement | <strong>Récurrence (max 15%):</strong> Nombre d'occurrences historiques</div>
                </template>
                <template #scoring>
                  <div class="tooltip-section-title">Interprétation</div>
                  <div class="tooltip-section-text">🟢 <strong>75-100%:</strong> Impact TRÈS ÉLEVÉ - Événement extrêmement corrélé | 🟠 <strong>50-75%:</strong> Impact MOYEN - Corrélation notable | 🔴 <strong>&lt;50%:</strong> Impact FAIBLE - Corrélation mineure</div>
                </template>
              </MetricTooltip>
            </th>
          </tr>
          <tr>
            <th></th>
            <th></th>
            <th>-30mn</th>
            <th>+30mn</th>
            <th>1h total</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(event, index) in topEvents" :key="event.name" :class="{ 'top-event': index < 3 }">
            <td>
              <span v-if="index === 0" class="rank-badge rank-1">🥇 #1</span>
              <span v-else-if="index === 1" class="rank-badge rank-2">🥈 #2</span>
              <span v-else-if="index === 2" class="rank-badge rank-3">🥉 #3</span>
              <span v-else class="rank-badge">#{{ index + 1 }}</span>
            </td>
            <td class="event-name">
              {{ event.name }}
              <span v-if="event.has_data === false" class="no-data-badge">❌</span>
            </td>
            <td class="volatility">{{ event.volatility_before_fmt }}</td>
            <td class="volatility">{{ event.volatility_after_fmt }}</td>
            <td class="volatility-total">{{ event.volatility_total_fmt }}</td>
            <td class="correlation-score">
              <span class="score-value" :class="getScoreClass(event.correlation_score)">
                {{ event.correlation_score.toFixed(1) }}%
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
        <li v-if="observations.length > 0" v-for="(obs, index) in observations" :key="index">
          {{ obs }}
        </li>
        <li v-else>Données insuffisantes pour générer des observations.</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'
import MetricTooltip from './MetricTooltip.vue'

interface Props {
  availablePairs: string[]
}

interface EventCorrelation {
  name: string
  count: number
  volatility_before: number
  volatility_after: number
  volatility_total: number
  volatility_before_fmt: string
  volatility_after_fmt: string
  volatility_total_fmt: string
  correlation_score: number
  has_data?: boolean
}

interface PairCorrelationData {
  pair: string
  events: EventCorrelation[]
}

const props = withDefaults(defineProps<Props>(), {
  availablePairs: () => []
})

const store = useAnalysisStore()
const loading = ref(false)

// Utiliser l'état du store
const selectedPair = computed({
  get: () => store.selectedPair,
  set: (value) => store.setPairSelection(value, store.selectedCalendarId),
})

const pairCorrelation = computed({
  get: () => store.pairCorrelationData as PairCorrelationData | null,
  set: (value) => store.setPairCorrelationData(value),
})

const topEvents = computed(() => {
  if (!pairCorrelation.value) return []
  return pairCorrelation.value.events.slice(0, 10).sort((a, b) => b.correlation_score - a.correlation_score)
})

const observations = computed(() => {
  if (!topEvents.value.length) return []
  
  const obs: string[] = []
  const topEvent = topEvents.value[0]
  
  if (topEvent) {
    obs.push(`L'événement "${topEvent.name}" est le plus corrélé avec ${selectedPair.value} (score: ${topEvent.correlation_score.toFixed(1)}%).`)
    
    const avgScore = topEvents.value.reduce((sum, e) => sum + e.correlation_score, 0) / topEvents.value.length
    if (avgScore > 60) {
      obs.push(`Corrélation moyenne élevée (${avgScore.toFixed(1)}%) - ${selectedPair.value} est très réactive aux événements économiques.`)
    } else if (avgScore > 30) {
      obs.push(`Corrélation moyenne modérée (${avgScore.toFixed(1)}%) - Impact événementiel mesuré.`)
    } else {
      obs.push(`Corrélation moyenne faible (${avgScore.toFixed(1)}%) - ${selectedPair.value} peu affectée par les événements économiques.`)
    }
  }
  
  return obs
})

async function loadPairCorrelation() {
  if (!selectedPair.value) return
  
  loading.value = true
  try {
    const result = await invoke<PairCorrelationData>(
      'get_pair_event_correlation',
      { symbol: selectedPair.value, monthsBack: 12 }
    )
    
    pairCorrelation.value = result
  } catch (error) {
    console.error('Erreur lors du chargement de la corrélation:', error)
    pairCorrelation.value = {
      pair: selectedPair.value,
      events: []
    }
  } finally {
    loading.value = false
  }
}

function getScoreClass(score: number): string {
  if (score >= 75) return 'score-green'
  if (score >= 50) return 'score-orange'
  return 'score-red'
}
</script>

<style scoped>
.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 20px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #30363d;
  border-top: 4px solid #58a6ff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.welcome {
  text-align: center;
  padding: 60px 40px;
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border-radius: 12px;
  border: 1px solid #30363d;
}

.welcome-icon {
  font-size: 4em;
  margin-bottom: 20px;
}

.welcome h3 {
  font-size: 1.8em;
  color: #e2e8f0;
  margin-bottom: 30px;
}

.welcome-select-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
  max-width: 400px;
  margin: 0 auto;
}

.welcome-select-container label {
  font-size: 1.1em;
  color: #8b949e;
  font-weight: 600;
}

.welcome-symbol-select {
  width: 100%;
  padding: 12px 15px;
  background: #161b22;
  border: 2px solid #30363d;
  color: #000;
  border-radius: 8px;
  font-size: 1em;
  cursor: pointer;
  transition: all 0.3s;
}

.welcome-symbol-select:hover {
  border-color: #58a6ff;
  box-shadow: 0 0 10px rgba(88, 166, 255, 0.2);
}

.welcome-symbol-select:focus {
  outline: none;
  border-color: #58a6ff;
  background: #0d1117;
  box-shadow: 0 0 15px rgba(88, 166, 255, 0.3);
}

.welcome-symbol-select option {
  background: #161b22;
  color: #000;
}

.welcome-symbol-select option:checked {
  background: #58a6ff;
  color: #000;
}

.pair-correlation-results {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.pair-info-card {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 25px;
}

.pair-header {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 20px;
}

.pair-header h3 {
  color: #e2e8f0;
  font-size: 1.4em;
  margin: 0;
}

.inline-select {
  padding: 10px 15px;
  background: #161b22;
  border: 2px solid #30363d;
  color: #8b949e;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 250px;
}

.inline-select:hover {
  border-color: #58a6ff;
  color: #58a6ff;
}

.inline-select:focus {
  outline: none;
  border-color: #58a6ff;
  background: #0d1117;
  color: #e2e8f0;
  box-shadow: 0 0 10px rgba(88, 166, 255, 0.3);
}

.inline-select option {
  background: #161b22;
  color: #000;
}

.inline-select option:checked {
  background: #58a6ff;
  color: #000;
}

.correlation-table-container {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 25px;
}

.correlation-table-container h3 {
  color: #e2e8f0;
  margin-top: 0;
  margin-bottom: 20px;
}

.correlation-table {
  width: 100%;
  border-collapse: collapse;
  color: #e2e8f0;
}

.correlation-table thead {
  background: #0d1117;
  border-bottom: 2px solid #30363d;
}

.correlation-table th {
  padding: 15px;
  text-align: left;
  font-weight: 600;
  color: #8b949e;
  cursor: pointer;
  user-select: none;
}

.correlation-table tbody tr {
  border-bottom: 1px solid #30363d;
  transition: background 0.2s;
}

.correlation-table tbody tr:hover {
  background: #161b22;
}

.correlation-table tbody tr.top-event {
  background: rgba(31, 111, 235, 0.1);
  border-left: 3px solid #1f6feb;
}

.correlation-table td {
  padding: 12px 15px;
}

.rank-badge {
  display: inline-block;
  background: #30363d;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.85em;
  font-weight: 600;
  color: #8b949e;
}

.rank-1 { background: #ffd700; color: #000; }
.rank-2 { background: #c0c0c0; color: #000; }
.rank-3 { background: #cd7f32; color: #fff; }

.event-name {
  font-weight: 500;
  color: #58a6ff;
}

.no-data-badge {
  display: inline-block;
  margin-left: 8px;
  font-size: 0.9em;
  padding: 2px 6px;
  background: #7f1d1d;
  color: #fca5a5;
  border-radius: 4px;
  font-weight: bold;
}

.has-data-badge {
  display: none;
}

.impact-badge {
  display: inline-block;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.9em;
  font-weight: 600;
  text-align: center;
  min-width: 80px;
}

.impact-high {
  background: rgba(248, 81, 73, 0.2);
  color: #f85149;
  border: 1px solid #f85149;
}

.impact-medium {
  background: rgba(217, 119, 6, 0.2);
  color: #d97706;
  border: 1px solid #d97706;
}

.impact-low {
  background: rgba(3, 102, 214, 0.2);
  color: #0366d6;
  border: 1px solid #0366d6;
}

.volatility {
  color: #79c0ff;
  font-weight: 500;
}

.volatility-total {
  color: #58a6ff;
  font-weight: 600;
  background: rgba(88, 166, 255, 0.1);
  padding: 4px 8px;
  border-radius: 4px;
}

.correlation-score {
  text-align: center;
}

.score-value {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 1.05em;
}

.score-green {
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
  border: 1px solid #22c55e;
}

.score-orange {
  background: rgba(249, 115, 22, 0.2);
  color: #f97316;
  border: 1px solid #f97316;
}

.score-red {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border: 1px solid #ef4444;
}

.observations-card {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 25px;
}

.observations-card h3 {
  color: #e2e8f0;
  margin-top: 0;
  margin-bottom: 15px;
}

.observations-card ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.observations-card li {
  padding: 10px 0;
  color: #8b949e;
  border-bottom: 1px solid #30363d;
  line-height: 1.6;
}

.observations-card li:last-child {
  border-bottom: none;
}

.observations-card li:before {
  content: "• ";
  color: #58a6ff;
  font-weight: bold;
  margin-right: 10px;
}
</style>
