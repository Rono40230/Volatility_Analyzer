<template>
  <div v-if="isOpen" class="drawer-overlay" @click="close">
    <div class="drawer-container" @click.stop>
      <div class="drawer-header">
        <h2>Événements {{ formatHour(selectedHour) }} (Heure de Paris)</h2>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <div class="drawer-content">
        <div v-if="eventSummary.length > 0" class="events-summary">
          <!-- Résumé des événements par type -->
          <div
            v-for="(item, idx) in eventSummary"
            :key="idx"
            class="event-summary-card"
            :class="item.impact.toLowerCase()"
          >
            <div class="summary-header">
              <span class="event-impact-badge" :class="item.impact.toLowerCase()">
                {{ item.impact }}
              </span>
              <span class="event-name">{{ item.eventName }} <span class="event-name-fr">({{ item.eventNameFr }})</span></span>
              <span class="event-count">{{ item.count }}x</span>
            </div>
            <div class="summary-stats">
              <span class="schedule-time">⏰ {{ item.schedule }}</span>
              <span class="country-flag">{{ item.flag }} {{ item.country }}</span>
            </div>
          </div>

          <!-- Statistiques globales -->
          <div class="global-stats">
            <div class="stat-box">
              <span class="stat-label">Total événements</span>
              <span class="stat-value">{{ totalEventCount }}</span>
            </div>
            <div class="stat-box">
              <span class="stat-label">HIGH</span>
              <span class="stat-value high-count">{{ highCount }}</span>
            </div>
            <div class="stat-box">
              <span class="stat-label">MEDIUM</span>
              <span class="stat-value medium-count">{{ mediumCount }}</span>
            </div>
          </div>
        </div>

        <div v-else class="no-events">
          <p>Aucun événement HIGH/MEDIUM pour cette heure</p>
        </div>
      </div>

      <div class="drawer-footer">
        <div class="legend">
          <span class="legend-item">
            <span class="badge high">HIGH</span>
            Impact élevé
          </span>
          <span class="legend-item">
            <span class="badge medium">MEDIUM</span>
            Impact moyen
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { EventInHour } from '../stores/volatility'
import { getEventTranslation } from '../stores/eventTranslations'
import { getEventSchedule } from '../stores/eventSchedules'

interface Props {
  isOpen: boolean
  selectedHour: number | null
  allEvents: EventInHour[] | null
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

function close() {
  emit('close')
}

function formatHour(hour: number | null): string {
  if (hour === null) return '—'
  return `${hour.toString().padStart(2, '0')}:00`
}

// ============================================
// Résumé statistique des événements
// ============================================

interface EventSummaryItem {
  eventName: string
  eventNameFr: string
  flag: string
  country: string
  impact: string
  count: number
  firstTime: string
  schedule: string
}

const eventSummary = computed(() => {
  if (!props.allEvents) return []

  // Grouper les événements par (nom + impact)
  const grouped = new Map<string, { count: number; impact: string; firstTime: string }>()

  for (const evt of props.allEvents) {
    const key = `${evt.event_name}|${evt.impact}`
    const current = grouped.get(key) || { count: 0, impact: evt.impact, firstTime: evt.datetime }

    current.count += 1
    grouped.set(key, current)
  }

  // Convertir en array et trier : HIGH d'abord, puis par fréquence décroissante
  return Array.from(grouped.entries())
    .map(([key, data]) => {
      const [eventName] = key.split('|')
      const translation = getEventTranslation(eventName)
      const schedule = getEventSchedule(eventName)
      return {
        eventName,
        eventNameFr: translation.fr,
        flag: translation.flag,
        country: translation.country,
        impact: data.impact,
        count: data.count,
        firstTime: data.firstTime,
        schedule,
      }
    })
    .sort((a, b) => {
      // HIGH avant MEDIUM
      if (a.impact !== b.impact) {
        return a.impact === 'HIGH' ? -1 : 1
      }
      // Même impact : par count décroissant
      return b.count - a.count
    })
})

const totalEventCount = computed(() => {
  // Compte le nombre total de PAIRES (nom + impact) distinctes
  // = nombre de lignes dans la liste eventSummary
  return eventSummary.value.length
})

const highCount = computed(() => {
  // Compte les paires HIGH distinctes dans eventSummary
  return eventSummary.value.filter(e => e.impact === 'HIGH').length
})

const mediumCount = computed(() => {
  // Compte les paires MEDIUM distinctes dans eventSummary
  return eventSummary.value.filter(e => e.impact === 'MEDIUM').length
})
</script>

<style scoped>
.drawer-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: flex-end;
  z-index: 1000;
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.drawer-container {
  background: #161b22;
  width: 100%;
  max-width: 500px;
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-left: 1px solid #30363d;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.5);
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #30363d;
  background: #0d1117;
}

.drawer-header h2 {
  margin: 0;
  font-size: 1.3em;
  color: #f0f6fc;
}

.close-btn {
  background: none;
  border: none;
  color: #8b949e;
  font-size: 1.5em;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: #21262d;
  color: #f0f6fc;
}

.drawer-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.events-summary {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.event-summary-card {
  padding: 14px;
  border-radius: 8px;
  border-left: 4px solid;
  background: rgba(56, 139, 253, 0.1);
  border-color: #388bfd;
  transition: all 0.2s;
}

.event-summary-card:hover {
  background: rgba(56, 139, 253, 0.15);
  transform: translateX(4px);
}

.event-summary-card.high {
  background: rgba(248, 81, 73, 0.1);
  border-color: #f85149;
}

.event-summary-card.high:hover {
  background: rgba(248, 81, 73, 0.15);
}

.event-summary-card.medium {
  background: rgba(201, 209, 43, 0.1);
  border-color: #c9d12d;
}

.event-summary-card.medium:hover {
  background: rgba(201, 209, 43, 0.15);
}

.summary-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.event-impact-badge {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.75em;
  font-weight: bold;
  color: white;
  min-width: 50px;
  text-align: center;
}

.event-impact-badge.high {
  background: #f85149;
}

.event-impact-badge.medium {
  background: #c9d12d;
}

.event-name {
  color: #58a6ff;
  font-weight: 600;
  flex: 1;
  word-break: break-word;
  font-size: 0.95em;
}

.event-name-fr {
  color: #8b949e;
  font-weight: 400;
  font-size: 0.9em;
}

.event-count {
  background: rgba(88, 166, 255, 0.2);
  color: #58a6ff;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: bold;
  font-size: 0.85em;
  white-space: nowrap;
}

.summary-stats {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 0.85em;
}

.schedule-time {
  color: #8b949e;
  display: flex;
  align-items: center;
  gap: 4px;
}

.country-flag {
  color: #8b949e;
  display: flex;
  align-items: center;
  gap: 4px;
}

.global-stats {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 12px;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #30363d;
}

.stat-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background: rgba(56, 139, 253, 0.1);
  border-radius: 6px;
  border: 1px solid #30363d;
}

.stat-label {
  color: #8b949e;
  font-size: 0.75em;
  margin-bottom: 4px;
}

.stat-value {
  color: #58a6ff;
  font-size: 1.4em;
  font-weight: bold;
}

.stat-value.high-count {
  color: #f85149;
}

.stat-value.medium-count {
  color: #c9d12d;
}

.no-events {
  padding: 40px 20px;
  text-align: center;
  color: #8b949e;
}

.no-events p {
  margin: 0;
  font-size: 0.95em;
}

.drawer-footer {
  padding: 20px;
  border-top: 1px solid #30363d;
  background: #0d1117;
}

.legend {
  display: flex;
  gap: 20px;
  font-size: 0.85em;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #8b949e;
}

.badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.75em;
  font-weight: bold;
  color: white;
}

.badge.high {
  background: #f85149;
}

.badge.medium {
  background: #c9d12d;
}
</style>
