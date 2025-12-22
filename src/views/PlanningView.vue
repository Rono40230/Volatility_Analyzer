<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import WeeklyCalendar from '../components/planning/WeeklyCalendar.vue'

// État pour la semaine sélectionnée
// Par défaut : semaine courante
const currentDate = new Date()
const selectedDate = ref(currentDate)
const calendarKey = ref(0)
const syncing = ref(false)
const error = ref<string | null>(null)

// Calcul du début et fin de semaine pour l'affichage
const weekStart = computed(() => {
  const d = new Date(selectedDate.value)
  const day = d.getDay()
  const diff = d.getDate() - day + (day === 0 ? -6 : 1) // ajuster pour lundi
  return new Date(d.setDate(diff))
})

const weekEnd = computed(() => {
  const d = new Date(weekStart.value)
  d.setDate(d.getDate() + 4) // Vendredi
  return d
})

function previousWeek() {
  const d = new Date(selectedDate.value)
  d.setDate(d.getDate() - 7)
  selectedDate.value = d
}

function nextWeek() {
  const d = new Date(selectedDate.value)
  d.setDate(d.getDate() + 7)
  selectedDate.value = d
}

function formatDate(date: Date): string {
  return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' })
}

async function syncCalendar() {
  syncing.value = true
  error.value = null
  try {
    await invoke('sync_forex_factory_week')
    // Force reload of calendar
    calendarKey.value++
  } catch (e) {
    error.value = 'Erreur de synchronisation: ' + e
  } finally {
    syncing.value = false
  }
}
</script>

<template>
  <div class="planning-view">
    <header class="planning-header">
      <div class="header-left">
        <h2>📅 Planning & Feuille de Route</h2>
        <p class="subtitle">Projection des analyses historiques sur le calendrier futur</p>
      </div>
      
      <div class="week-selector">
        <button class="nav-btn" @click="previousWeek">◀</button>
        <div class="current-week">
          <span class="week-label">Semaine du</span>
          <span class="week-dates">{{ formatDate(weekStart) }} au {{ formatDate(weekEnd) }}</span>
        </div>
        <button class="nav-btn" @click="nextWeek">▶</button>
      </div>

      <div class="header-actions">
        <div v-if="error" class="error-msg">{{ error }}</div>
        <button class="btn-secondary" @click="syncCalendar" :disabled="syncing">
          <span v-if="syncing" class="spinner">⏳</span>
          <span v-else>🔄</span>
          Synchronisation des événements de la semaine
        </button>
        <button class="btn-primary">
          🖨️ Exporter la Feuille
        </button>
      </div>
    </header>

    <main class="planning-content">
      <WeeklyCalendar :week-start="weekStart" :key="calendarKey" />
    </main>
  </div>
</template>

<style scoped>
.planning-view {
  /* Theme Variables */
  --bg-primary: #0f1419;
  --bg-secondary: #161b22;
  --bg-tertiary: #21262d;
  --bg-hover: rgba(177, 186, 196, 0.12);
  
  --text-primary: #e6edf3;
  --text-secondary: #8b949e;
  --text-disabled: #484f58;
  
  --primary-color: #58a6ff;
  --primary-rgb: 88, 166, 255;
  
  --border-color: #30363d;
  
  --success-color: #238636;
  --warning-color: #d29922;
  --danger-color: #da3633;

  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  padding: 1.5rem;
  gap: 1.5rem;
}

.planning-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.header-left h2 {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  color: var(--text-primary);
}

.subtitle {
  font-size: 0.9rem;
  color: var(--text-secondary);
  margin: 0.25rem 0 0 0;
}

.week-selector {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: var(--bg-secondary);
  padding: 0.5rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.nav-btn {
  background: none;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 1.2rem;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  transition: background 0.2s;
}

.nav-btn:hover {
  background: var(--bg-hover);
}

.current-week {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 200px;
}

.week-label {
  font-size: 0.8rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.week-dates {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--primary-color);
}

.header-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.header-actions .btn-primary {
  background: var(--primary-color);
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: opacity 0.2s;
}

.header-actions .btn-primary:hover {
  opacity: 0.9;
}

.btn-secondary {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  transition: all 0.2s;
}

.btn-secondary:hover:not(:disabled) {
  background-color: var(--bg-hover);
  border-color: var(--text-secondary);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
  display: inline-block;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.planning-content {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-secondary);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  padding: 1rem;
}

.error-msg {
  color: #ff6b6b;
  font-size: 0.9rem;
  margin-right: 10px;
}
</style>
