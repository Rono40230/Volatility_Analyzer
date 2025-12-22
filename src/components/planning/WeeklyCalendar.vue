<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import EventActionCard from './EventActionCard.vue'

const props = defineProps<{
  weekStart: Date
}>()

interface ProjectedEvent {
  id: string
  time: string
  name: string
  currency: string
  impact: string
  pair: string
  offset: number
  tp: number
  sl: number
  confidence_score: number
  source: string
}

// Structure de données pour un jour
interface DayColumn {
  date: Date
  dayName: string
  events: ProjectedEvent[]
}

const events = ref<ProjectedEvent[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

async function loadEvents() {
  loading.value = true
  error.value = null
  try {
    const start = new Date(props.weekStart)
    start.setHours(0, 0, 0, 0)
    
    const end = new Date(start)
    end.setDate(end.getDate() + 5) // Samedi matin 00:00 pour inclure tout vendredi
    
    events.value = await invoke('project_stats_on_calendar', {
      startDate: start.toISOString(),
      endDate: end.toISOString()
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// Recharger quand la semaine change
watch(() => props.weekStart, () => {
  loadEvents()
}, { immediate: true })

const weekDays = computed<DayColumn[]>(() => {
  const days: DayColumn[] = []
  const start = new Date(props.weekStart)
  
  // Lundi (0) à Vendredi (4)
  for (let i = 0; i < 5; i++) {
    const d = new Date(start)
    d.setDate(start.getDate() + i)
    
    // Filtrer les événements pour ce jour
    const dayEvents = events.value.filter(e => {
      // Le format de time est "YYYY-MM-DD HH:MM:SS" (NaiveDateTime::to_string())
      // On peut le parser simplement
      const evtDate = new Date(e.time.replace(' ', 'T'))
      return evtDate.getDate() === d.getDate() &&
             evtDate.getMonth() === d.getMonth() &&
             evtDate.getFullYear() === d.getFullYear()
    }).map(e => ({
      ...e,
      // Formater l'heure pour l'affichage (HH:MM)
      time: e.time.split(' ')[1]?.substring(0, 5) || e.time,
      // Mapper confidence_score vers confidenceScore pour le composant enfant
      confidenceScore: e.confidence_score
    }))

    days.push({
      date: d,
      dayName: d.toLocaleDateString('fr-FR', { weekday: 'long' }),
      events: dayEvents
    })
  }
  return days
})

function handleEventUpdate(id: string, field: string, value: number) {
  // TODO: Mettre à jour le store / state local
}

function formatDayDate(date: Date): string {
  return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' })
}

function isToday(date: Date): boolean {
  const today = new Date()
  return date.getDate() === today.getDate() &&
         date.getMonth() === today.getMonth() &&
         date.getFullYear() === today.getFullYear()
}
</script>

<template>
  <div class="weekly-calendar">
    <div 
      v-for="day in weekDays" 
      :key="day.date.toISOString()" 
      class="day-column"
      :class="{ 'is-today': isToday(day.date) }"
    >
      <div class="day-header">
        <span class="day-name">{{ day.dayName }}</span>
        <span class="day-date">{{ formatDayDate(day.date) }}</span>
      </div>
      
      <div class="day-events">
        <div v-if="day.events.length === 0" class="no-events">
          Rien à signaler
        </div>
        <EventActionCard 
          v-for="event in day.events" 
          :key="event.id"
          :event="event"
          @update="handleEventUpdate"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.weekly-calendar {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 1rem;
  height: 100%;
  min-width: 1000px; /* Assurer une largeur min pour la lisibilité */
}

.day-column {
  background: var(--bg-tertiary);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  border: 1px solid transparent;
}

.day-column.is-today {
  border-color: var(--primary-color);
  background: rgba(var(--primary-rgb), 0.05);
}

.day-header {
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.day-name {
  text-transform: uppercase;
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--text-secondary);
}

.day-date {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--text-primary);
}

.is-today .day-name,
.is-today .day-date {
  color: var(--primary-color);
}

.day-events {
  flex: 1;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  overflow-y: auto;
}

.no-events {
  text-align: center;
  color: var(--text-disabled);
  font-size: 0.9rem;
  margin-top: 2rem;
  font-style: italic;
}
</style>
