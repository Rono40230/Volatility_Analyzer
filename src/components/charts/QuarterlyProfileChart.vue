<template>
  <div class="chart-container">
    <!-- Event Filter Controls -->
    <div class="event-controls">
      <button 
        @click="filterMode = 'none'" 
        :class="{ active: filterMode === 'none' }"
        title="Masquer les événements"
      >
        🚫
      </button>
      <button 
        @click="filterMode = 'peak'" 
        :class="{ active: filterMode === 'peak' }"
        title="Événements proches du pic"
      >
        🎯
      </button>
      <button 
        @click="filterMode = 'all'" 
        :class="{ active: filterMode === 'all' }"
        title="Tous les événements"
      >
        👁️
      </button>
    </div>

    <svg viewBox="0 0 400 200" class="chart-svg">
      <!-- Grid Lines -->
      <line x1="40" y1="180" x2="380" y2="180" stroke="#30363d" stroke-width="1" />
      <line x1="40" y1="20" x2="40" y2="180" stroke="#30363d" stroke-width="1" />

      <!-- Quarter Background (Pastel Red) -->
      <rect :x="getX(0)" y="20" :width="getX(15) - getX(0)" height="160" fill="#f87171" opacity="0.1" stroke="none" />

      <!-- Vertical Markers for Quarter Start/End -->
      <line v-if="isExtended" :x1="getX(0)" y1="20" :x2="getX(0)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />
      <line v-if="isExtended" :x1="getX(15)" y1="20" :x2="getX(15)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />

      <!-- X Axis Labels (Exact Time) -->
      <text v-for="m in xAxisLabels" :key="m" :x="getX(m)" y="195" font-size="7" fill="#8b949e" text-anchor="middle">{{ formatTime(m) }}</text>

      <!-- Y Axis Labels & Grid (Every 1 pip) -->
      <template v-for="tick in yAxisTicks" :key="tick">
        <line :x1="40" :y1="getY(tick)" :x2="380" :y2="getY(tick)" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" opacity="0.5" />
        <text x="35" :y="getY(tick) + 2" font-size="7" fill="#8b949e" text-anchor="end">{{ tick }}</text>
      </template>

      <!-- Profile Line -->
      <polyline :points="points" fill="none" stroke="#58a6ff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />

      <!-- Area under curve -->
      <path :d="areaPath" fill="url(#blueGradient)" stroke="none" opacity="0.3" />

      <!-- Volatility Zone (Pastel Green) -->
      <rect v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="getX(optimalEntry)"
            y="20"
            :width="volatilityZoneWidth"
            height="160"
            fill="#bbf7d0"
            opacity="0.2"
            stroke="none" />

      <!-- Volatility Zone Label -->
      <text v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="getX(optimalEntry) + volatilityZoneWidth / 2"
            y="100"
            :transform="`rotate(-90, ${getX(optimalEntry) + volatilityZoneWidth / 2}, 100)`"
            font-size="7"
            fill="#22c55e"
            text-anchor="middle"
            dominant-baseline="middle"
            font-weight="bold"
            style="pointer-events: none; letter-spacing: 0.5px; text-shadow: 0px 0px 2px rgba(0,0,0,0.5);">
        Durée de l'impulsion : {{ duration }}m
      </text>

      <!-- Optimal Entry Marker -->
      <line v-if="optimalEntry !== undefined" :x1="getX(optimalEntry)" y1="20" :x2="getX(optimalEntry)" y2="180" stroke="#10b981" stroke-width="1" stroke-dasharray="4,4" />
      <text v-if="optimalEntry !== undefined" :x="getX(optimalEntry)" y="15" font-size="7" fill="#10b981" text-anchor="middle" font-weight="bold">{{ entryLabel || `Entrée (${formatTime(optimalEntry)})` }}</text>

      <!-- Event Flags -->
      <g v-for="(event, index) in processedEvents" :key="index">
        <!-- Flag Pole -->
        <line 
          :x1="getX(getEventMinute(event.time))" 
          y1="20" 
          :x2="getX(getEventMinute(event.time))" 
          y2="180" 
          :stroke="getEventColor(event.impact)" 
          stroke-width="1" 
          stroke-dasharray="2,2" 
          opacity="0.7" 
        />
        
        <!-- Flag Icon (Triangle) -->
        <path 
          :d="`M ${getX(getEventMinute(event.time))} ${20 + event.stackIndex * 12} L ${getX(getEventMinute(event.time)) + 6} ${24 + event.stackIndex * 12} L ${getX(getEventMinute(event.time))} ${28 + event.stackIndex * 12} Z`" 
          :fill="getEventColor(event.impact)" 
        />

        <!-- Frequency Label -->
        <text 
          :x="getX(getEventMinute(event.time)) + 8" 
          :y="26 + event.stackIndex * 12" 
          font-size="6" 
          :fill="getEventColor(event.impact)"
          alignment-baseline="middle"
        >x{{ event.frequency }}</text>

        <!-- Event Label (on hover via title) -->
        <title>{{ event.tooltip }}</title>
      </g>

      <!-- Gradients -->
      <defs>
        <linearGradient id="blueGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" style="stop-color:#58a6ff;stop-opacity:0.5" />
          <stop offset="100%" style="stop-color:#58a6ff;stop-opacity:0" />
        </linearGradient>
      </defs>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, watch, ref } from 'vue'
import { eventTranslations } from '../../utils/eventTranslations'
import { 
  formatTime as formatTimeUtil, 
  getEventMinute as getEventMinuteUtil, 
  getEventColor, 
  getX as getXUtil, 
  getY as getYUtil 
} from './quarterlyProfileChartUtils'

const props = defineProps<{
  profile: number[]
  optimalEntry?: number
  duration?: number
  entryLabel?: string
  hour?: number
  quarter?: number
  events?: Array<{
    time: string
    name: string
    impact: string
    currency: string
    frequency: number
  }>
}>()

const filterMode = ref<'none' | 'all' | 'peak'>('none')

onMounted(() => {
  // Mounted
})

watch(() => props.events, (newEvents) => {
  // Events updated
}, { deep: true })

const isExtended = computed(() => props.profile.length > 15)
const minMinute = computed(() => isExtended.value ? -5 : 0)
const maxMinute = computed(() => isExtended.value ? 30 : 14) // 30 to include up to +15min after end (15+15)
const totalRange = computed(() => maxMinute.value - minMinute.value)

const formatTime = (minute: number) => formatTimeUtil(minute, props.hour, props.quarter)
const getEventMinute = (timeStr: string) => getEventMinuteUtil(timeStr, props.hour, props.quarter)

const xAxisLabels = computed(() => {
  if (isExtended.value) {
    // Échelle de 5 minutes : -5, 0, 5, 10, 15, 20, 25, 30, 35, 40, 45
    const labels = []
    for (let i = minMinute.value; i <= maxMinute.value; i += 5) {
      labels.push(i)
    }
    return labels
  }
  return [0, 5, 10, 14]
})

const maxValue = computed(() => {
  const max = Math.max(...props.profile)
  const ceiling = Math.ceil(max)
  return ceiling === 0 ? 1 : ceiling
})

const yAxisTicks = computed(() => {
  const max = maxValue.value
  const ticks = []
  let step = 1
  if (max > 20) step = 2
  if (max > 50) step = 5
  
  for (let i = 0; i <= max; i += step) {
    ticks.push(i)
  }
  return ticks
})

const getX = (minute: number) => getXUtil(minute, minMinute.value, totalRange.value)
const getY = (val: number) => getYUtil(val, maxValue.value)

const points = computed(() => {
  return props.profile.map((val, idx) => {
    const minute = minMinute.value + idx
    return `${getX(minute)},${getY(val)}`
  }).join(' ')
})

const areaPath = computed(() => {
  const pts = points.value
  const firstX = getX(minMinute.value)
  const lastX = getX(minMinute.value + props.profile.length - 1)
  const bottomY = 180
  return `M ${firstX},${bottomY} L ${pts} L ${lastX},${bottomY} Z`
})

const volatilityZoneWidth = computed(() => {
  if (props.optimalEntry === undefined || props.duration === undefined) return 0
  const startX = getX(props.optimalEntry)
  const endX = getX(props.optimalEntry + props.duration)
  // Clamp width to chart area
  const maxX = 380
  return Math.min(endX, maxX) - startX
})

const processedEvents = computed(() => {
  if (!props.events || filterMode.value === 'none') return []

  let eventsToProcess = props.events

  // Filter for peak mode
  if (filterMode.value === 'peak') {
    if (props.optimalEntry !== undefined) {
      eventsToProcess = props.events.filter(e => {
        const eventMinute = getEventMinute(e.time)
        // Show events within +/- 5 minutes of optimal entry
        return Math.abs(eventMinute - props.optimalEntry!) <= 5
      })
    } else {
      // No optimal entry = no events near peak
      return []
    }
  }

  // 1. Group by time + frequency
  const groups = new Map<string, {
    time: string,
    frequency: number,
    impacts: string[],
    names: string[],
    currencies: string[]
  }>()

  for (const event of eventsToProcess) {
    const key = `${event.time}-${event.frequency}`
    if (!groups.has(key)) {
      groups.set(key, {
        time: event.time,
        frequency: event.frequency,
        impacts: [event.impact],
        names: [event.name],
        currencies: [event.currency]
      })
    } else {
      const g = groups.get(key)!
      g.impacts.push(event.impact)
      g.names.push(event.name)
      g.currencies.push(event.currency)
    }
  }

  // 2. Convert groups to displayable events
  const displayEvents = Array.from(groups.values()).map(g => {
    // Determine max impact for color
    let maxImpact = 'Low'
    if (g.impacts.some(i => i.toUpperCase() === 'HIGH')) maxImpact = 'High'
    else if (g.impacts.some(i => i.toUpperCase() === 'MEDIUM')) maxImpact = 'Medium'
    
    // Format tooltip info
    const eventList = g.names.map((n, i) => {
      const translation = eventTranslations[n]
      if (translation) {
        return `• ${n} (${translation.fr}) ${translation.flag}`
      }
      return `• ${n} (${g.currencies[i]})`
    }).join('\n')
    const tooltip = `${g.time} [${maxImpact}]\n${eventList}`
    
    return {
      time: g.time,
      frequency: g.frequency,
      impact: maxImpact,
      tooltip: tooltip,
      stackIndex: 0 // To be calculated
    }
  })

  // 3. Sort by time, then by frequency (descending)
  displayEvents.sort((a, b) => {
    const timeDiff = a.time.localeCompare(b.time)
    if (timeDiff !== 0) return timeDiff
    return b.frequency - a.frequency
  })

  // 4. Assign stack index for same time
  const timeStack = new Map<string, number>()
  for (const event of displayEvents) {
    const count = timeStack.get(event.time) || 0
    event.stackIndex = count
    timeStack.set(event.time, count + 1)
  }

  return displayEvents
})
</script>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  min-height: 150px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  position: relative;
}

.event-controls {
  position: absolute;
  top: 5px;
  right: 5px;
  display: flex;
  gap: 4px;
  z-index: 10;
}

.event-controls button {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
  color: rgba(255, 255, 255, 0.5);
}

.event-controls button:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.event-controls button.active {
  background: rgba(88, 166, 255, 0.2);
  border-color: rgba(88, 166, 255, 0.5);
  color: #58a6ff;
}

.chart-svg {
  width: 100%;
  height: 100%;
}
</style>
