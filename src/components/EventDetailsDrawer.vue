<template>
  <div v-if="isOpen" class="drawer-overlay" @click="close">
    <div class="drawer-container" @click.stop>
      <div class="drawer-header">
        <h2>Événements {{ formatHour(selectedHour) }} (Heure de Paris)</h2>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <div class="drawer-content">
        <div v-if="eventSummary.length > 0" class="events-summary">
          <div v-for="(item, idx) in eventSummary" :key="idx" class="event-summary-card" :class="item.impact.toLowerCase()">
            <div class="summary-header">
              <span class="event-impact-badge" :class="item.impact.toLowerCase()">{{ item.impact }}</span>
              <span class="event-name">{{ item.eventName }} <span class="event-name-fr">({{ item.eventNameFr }})</span></span>
              <span class="event-count">{{ item.count }}x</span>
            </div>
            <div class="summary-stats">
              <span class="schedule-time">⏰ {{ item.schedule }}</span>
              <span class="country-flag">{{ item.flag }} {{ item.country }}</span>
            </div>
          </div>
          <div class="global-stats">
            <div class="stat-box">
              <span class="stat-label">Total événements HIGH</span>
              <span class="stat-value high-count">{{ totalEventCount }}</span>
            </div>
          </div>
        </div>
        <div v-else class="no-events">
          <p>Aucun événement HIGH pour cette heure</p>
        </div>
      </div>

      <div class="drawer-footer">
        <div class="legend">
          <span class="legend-item">
            <span class="badge high">HIGH</span>
            Impact élevé
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

const props = defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

function close() {
  emit('close')
}

function formatHour(hour: number | null): string {
  if (hour === null) return '—'
  return `${hour.toString().padStart(2, '0')}:00`
}

function normalizeImpact(impact: string): string {
  const i = impact.toUpperCase().trim()
  if (i === 'HIGH' || i === 'H') return 'HIGH'
  if (i === 'MEDIUM' || i === 'M' || i === 'MED') return 'MEDIUM'
  if (i === 'LOW' || i === 'L') return 'LOW'
  return 'UNKNOWN'
}

const eventSummary = computed(() => {
  if (!props.allEvents) return []
  const grouped = new Map<string, { count: number; impact: string; firstTime: string }>()
  for (const evt of props.allEvents) {
    const normImpact = normalizeImpact(evt.impact)
    const key = `${evt.event_name}|${normImpact}`
    const current = grouped.get(key) || { count: 0, impact: normImpact, firstTime: evt.datetime }
    current.count += 1
    grouped.set(key, current)
  }
  return Array.from(grouped.entries()).map(([key, data]) => {
    const [eventName] = key.split('|')
    const translation = getEventTranslation(eventName)
    const schedule = getEventSchedule(eventName)
    return { eventName, eventNameFr: translation.fr, flag: translation.flag, country: translation.country, impact: data.impact, count: data.count, firstTime: data.firstTime, schedule }
  }).filter(e => e.impact === 'HIGH').sort((a, b) => b.count - a.count)
})

const totalEventCount = computed(() => eventSummary.value.length)
</script>

<style scoped>
.drawer-overlay { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5); display: flex; justify-content: flex-end; z-index: 1000; animation: fadeIn 0.3s ease-in; }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
.drawer-container { background: #161b22; width: 100%; max-width: 500px; height: 100vh; display: flex; flex-direction: column; border-left: 1px solid #30363d; box-shadow: -4px 0 12px rgba(0, 0, 0, 0.5); animation: slideIn 0.3s ease-out; }
@keyframes slideIn { from { transform: translateX(100%); } to { transform: translateX(0); } }
.drawer-header { display: flex; justify-content: space-between; align-items: center; padding: 20px; border-bottom: 1px solid #30363d; background: #0d1117; }
.drawer-header h2 { margin: 0; font-size: 1.3em; color: #f0f6fc; }
.close-btn { background: none; border: none; color: #8b949e; font-size: 1.5em; cursor: pointer; padding: 0; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border-radius: 6px; transition: background 0.2s; }
.close-btn:hover { background: #21262d; color: #f0f6fc; }
.drawer-content { flex: 1; overflow-y: auto; padding: 20px; }
.events-summary { display: flex; flex-direction: column; gap: 16px; }
.event-summary-card { padding: 14px; border-radius: 8px; border-left: 4px solid; background: rgba(56, 139, 253, 0.1); border-color: #388bfd; transition: all 0.2s; }
.event-summary-card:hover { background: rgba(56, 139, 253, 0.15); transform: translateX(4px); }
.event-summary-card.high { background: rgba(248, 81, 73, 0.1); border-color: #f85149; }
.event-summary-card.high:hover { background: rgba(248, 81, 73, 0.15); }
.event-summary-card.medium { background: rgba(201, 209, 43, 0.1); border-color: #c9d12d; }
.event-summary-card.medium:hover { background: rgba(201, 209, 43, 0.15); }
.event-summary-card.low { background: rgba(56, 139, 253, 0.1); border-color: #388bfd; }
.event-summary-card.low:hover { background: rgba(56, 139, 253, 0.15); }
.summary-header { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
.event-impact-badge { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 0.75em; font-weight: bold; color: white; min-width: 50px; text-align: center; }
.event-impact-badge.high { background: #f85149; }
.event-impact-badge.medium { background: #c9d12d; }
.event-impact-badge.low { background: #388bfd; }
.event-name { color: #58a6ff; font-weight: 600; flex: 1; word-break: break-word; font-size: 0.95em; }
.event-name-fr { color: #8b949e; font-weight: 400; font-size: 0.9em; }
.event-count { background: rgba(88, 166, 255, 0.2); color: #58a6ff; padding: 2px 8px; border-radius: 4px; font-weight: bold; font-size: 0.85em; white-space: nowrap; }
.summary-stats { display: flex; align-items: center; gap: 12px; font-size: 0.85em; }
.schedule-time, .country-flag { color: #8b949e; display: flex; align-items: center; gap: 4px; }
.global-stats { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; margin-top: 20px; padding-top: 20px; border-top: 1px solid #30363d; }
.stat-box { display: flex; flex-direction: column; align-items: center; padding: 12px; background: rgba(56, 139, 253, 0.1); border-radius: 6px; border: 1px solid #30363d; }
.stat-label { color: #8b949e; font-size: 0.75em; margin-bottom: 4px; }
.stat-value { color: #58a6ff; font-size: 1.4em; font-weight: bold; }
.stat-value.high-count { color: #f85149; }
.stat-value.medium-count { color: #c9d12d; }
.stat-value.low-count { color: #388bfd; }
.no-events { padding: 40px 20px; text-align: center; color: #8b949e; }
.no-events p { margin: 0; font-size: 0.95em; }
.drawer-footer { padding: 20px; border-top: 1px solid #30363d; background: #0d1117; }
.legend { display: flex; gap: 20px; font-size: 0.85em; }
.legend-item { display: flex; align-items: center; gap: 8px; color: #8b949e; }
.badge { display: inline-block; padding: 2px 6px; border-radius: 3px; font-size: 0.75em; font-weight: bold; color: white; }
.badge.high { background: #f85149; }
.badge.medium { background: #c9d12d; }
.badge.low { background: #388bfd; }
</style>
