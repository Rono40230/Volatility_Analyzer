import { computed } from 'vue'
import type { EventInHour } from '../stores/volatility'
import { getEventTranslation } from '../stores/eventTranslations'
import { getEventSchedule } from '../stores/eventSchedules'

export interface EventSummaryItem {
  eventName: string
  eventNameFr: string
  flag: string
  country: string
  impact: string
  count: number
  firstTime: string
  schedule: string
}

function normalizeImpact(impact: string): string {
  const i = impact.toUpperCase().trim()
  if (i === 'HIGH' || i === 'H') return 'HIGH'
  if (i === 'MEDIUM' || i === 'M' || i === 'MED') return 'MEDIUM'
  if (i === 'LOW' || i === 'L') return 'LOW'
  return 'UNKNOWN'
}

export function useEventSummary(allEvents: any) {
  const eventSummary = computed(() => {
    if (!allEvents?.value) return []

    const grouped = new Map<string, { count: number; impact: string; firstTime: string }>()

    for (const evt of allEvents.value) {
      const normImpact = normalizeImpact(evt.impact)
      const key = `${evt.event_name}|${normImpact}`
      const current = grouped.get(key) || { count: 0, impact: normImpact, firstTime: evt.datetime }
      current.count += 1
      grouped.set(key, current)
    }

    return Array.from(grouped.entries())
      .map(([key, data]) => {
        const [eventName] = key.split('|')
        const translation = getEventTranslation(eventName)
        const schedule = getEventSchedule(eventName)
        return { eventName, eventNameFr: translation.fr, flag: translation.flag, country: translation.country, impact: data.impact, count: data.count, firstTime: data.firstTime, schedule }
      })
      .filter(e => e.impact === 'HIGH')
      .sort((a, b) => b.count - a.count)
  })

  const totalEventCount = computed(() => eventSummary.value.length)

  return { eventSummary, totalEventCount }
}
