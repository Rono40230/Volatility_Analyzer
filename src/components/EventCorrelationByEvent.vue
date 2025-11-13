<template>
  <div v-if="loadingEvent" class="loading">
    <div class="spinner"></div>
    <p>Analyse de l'impact de l'événement...</p>
  </div>

  <!-- Message de bienvenue avec dropdown événement -->
  <div v-if="!selectedEventId && !loadingEvent" class="welcome">
    <div class="welcome-icon">🎯</div>
    <h3>Analyse Rétrospective par Événement</h3>
    <div class="welcome-select-container">
      <!-- Dropdown UNIQUE pour tous les événements -->
      <div class="dropdown-group">
        <label for="event-select">� Événements (HIGH & MEDIUM Impact) :</label>
        <select 
          id="event-select" 
          v-model="selectedEventId" 
          class="welcome-symbol-select"
          @change="loadEventImpact"
        >
          <option value="">Choisir un événement</option>
          <option v-for="event in props.pastEvents" :key="`event-${event.name}`" :value="event.name">
            {{ event.name }} ({{ event.count }} occurrences)
          </option>
        </select>
      </div>
    </div>
  </div>

  <div v-if="eventImpact && !loadingEvent" class="event-impact-results">
    <!-- Informations sur l'événement avec dropdown pour changer -->
    <div class="event-info-card">
      <div class="event-header">
        <div class="event-title-with-selector">
          <h3>
            {{ eventImpact.event_name }}
            <span class="inline-info">| 🌍 {{ eventImpact.country }} | 💱 {{ eventImpact.currency }}</span>
          </h3>
          <!-- Dropdown pour changer d'événement -->
          <select 
            v-model="selectedEventId" 
            class="inline-select"
            @change="loadEventImpact"
          >
            <option value="">Changer d'événement</option>
            <option v-for="event in props.pastEvents" :key="`event-${event.name}`" :value="event.name">
              {{ event.name }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <!-- Classement des paires par impact -->
    <div class="impact-ranking">
      <h3>📊 Impact mesuré par paire du {{ formatDateRange(eventImpact.datetime) }} au {{ formatDateRange(eventImpact.last_datetime) }} soit {{ eventImpact.event_count }} événements</h3>
      
      <table class="impact-table">
        <thead>
          <tr>
            <th>Rang</th>
            <th>Paire</th>
            <th style="cursor: pointer;" @click="sortEventVolatility">Volatilité Event</th>
            <th title="Variation moyenne en pips sur les 7 jours ouvrables précédant l'événement, à la même heure">Vol. Baseline</th>
            <th title="Ratio: volatilité event / volatilité baseline. Mesure l'amplification de la volatilité">Multiplicateur</th>
            <th title="Points de trading (1 point = 1/10 pip)">Points</th>
            <th title="Valeur monétaire approximative du mouvement de volatilité">Prix</th>
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
            <td class="volatility">{{ pair.event_volatility_formatted }} pips</td>
            <td class="baseline">{{ pair.baseline_volatility_formatted }} pips</td>
            <td class="multiplier">
              <span class="multiplier-value" :class="getMultiplierClass(pair.multiplier)">
                ×{{ pair.multiplier.toFixed(1) }}
              </span>
            </td>
            <td class="points">{{ pair.points_formatted }}</td>
            <td class="price">{{ pair.price_formatted }}</td>
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

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'

// Types
interface PastEvent {
  name: string
  count: number
}

interface PairImpact {
  symbol: string
  event_volatility: number
  baseline_volatility: number
  event_volatility_formatted: string
  baseline_volatility_formatted: string
  points: number
  points_formatted: string
  price: number
  price_formatted: string
  multiplier: number
  direction: string
}

interface EventImpactResult {
  event_id: number
  event_name: string
  datetime: string
  last_datetime: string
  country: string
  currency: string
  event_count: number
  window_start: string
  window_end: string
  pair_impacts: PairImpact[]
  observations: string[]
}

// Props et Emits
const props = defineProps<{
  pastEvents: PastEvent[]
  calendarId: number | null
}>()

const emit = defineEmits<{
  'event-loaded': [EventImpactResult]
}>()

// Store pour persistance
const store = useAnalysisStore()

// État
const selectedEventId = ref<string>('')
const eventImpact = ref<EventImpactResult | null>(null)
const loadingEvent = ref(false)

// État du tri pour "Par Événement"
const eventVolatilitySortOrder = ref<'asc' | 'desc'>('desc')

// Au montage, restaurer l'état du store
onMounted(() => {
  if (store.eventCorrelationData?.eventImpact) {
    eventImpact.value = store.eventCorrelationData.eventImpact
    selectedEventId.value = store.selectedEvent
  }
})

// Fonctions
async function loadEventImpact() {
  if (!selectedEventId.value) return
  
  loadingEvent.value = true
  try {
    // Trouver le count de l'événement sélectionné dans la liste
    const selectedEvent = props.pastEvents.find(e => e.name === selectedEventId.value)
    
    const eventCount = selectedEvent?.count || 0
    
    // Charger les vraies données depuis le backend
    eventImpact.value = await invoke<EventImpactResult>('get_event_impact_by_pair', { 
      eventType: selectedEventId.value,
      eventCount: eventCount,
      calendarId: props.calendarId
    })
    
    // Sauvegarder dans le store pour persistance
    store.setEventSelection(selectedEventId.value, props.calendarId)
    store.setEventCorrelationData({
      event: selectedEventId.value,
      eventImpact: eventImpact.value
    })
    
    emit('event-loaded', eventImpact.value)
  } catch (error) {
    console.error('Erreur analyse événement:', error)
    eventImpact.value = null
  } finally {
    loadingEvent.value = false
  }
}

function formatDateRange(datetime: string): string {
  const date = new Date(datetime)
  return date.toLocaleString('fr-FR', {
    day: 'numeric',
    month: 'long',
    year: 'numeric'
  })
}

function getMultiplierClass(multiplier: number): string {
  if (multiplier >= 50) return 'mult-extreme'
  if (multiplier >= 20) return 'mult-very-high'
  if (multiplier >= 10) return 'mult-high'
  if (multiplier >= 5) return 'mult-medium'
  return 'mult-low'
}

function sortEventVolatility() {
  // Inverser l'ordre de tri
  eventVolatilitySortOrder.value = eventVolatilitySortOrder.value === 'asc' ? 'desc' : 'asc'

  if (!eventImpact.value?.pair_impacts) return

  // Trier les pair_impacts par event_volatility
  const sorted = [...eventImpact.value.pair_impacts]
  
  sorted.sort((a, b) => {
    const aVal = a.event_volatility
    const bVal = b.event_volatility

    if (eventVolatilitySortOrder.value === 'asc') {
      return aVal - bVal
    } else {
      return bVal - aVal
    }
  })

  eventImpact.value.pair_impacts = sorted
}
</script>

<style scoped>
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

.welcome-select-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  margin-top: 30px;
  max-width: 600px;
  width: 100%;
  margin-left: auto;
  margin-right: auto;
}

.dropdown-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.dropdown-group label {
  font-weight: 600;
  color: #cbd5e0;
  font-size: 0.95em;
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
  min-width: 300px;
  max-width: 600px;
  width: 100%;
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
  align-items: flex-start;
  margin-bottom: 20px;
  gap: 15px;
}

.event-title-with-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.event-title-with-selector h3 {
  margin: 0;
  white-space: nowrap;
}

.event-header h3 {
  color: #e2e8f0;
  font-size: 1.5em;
  margin: 0;
}

.inline-info {
  font-size: 0.7em;
  color: #cbd5e0;
  font-weight: 400;
  margin-left: 15px;
}

.inline-select {
  padding: 8px 12px;
  border: 1px solid #4a5568;
  background: #1c2128;
  color: #000000;
  border-radius: 6px;
  font-size: 0.95em;
  cursor: pointer;
  transition: all 0.2s;
}

.inline-select:hover {
  border-color: #58a6ff;
  background: #262d38;
  color: #000000;
}

.inline-select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 8px rgba(88, 166, 255, 0.3);
  color: #000000;
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

.baseline {
  font-weight: 600;
  color: #a78bfa;
}

.points {
  font-weight: 600;
  color: #6ee7b7;
}

.price {
  font-weight: 600;
  color: #fbbf24;
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
</style>
