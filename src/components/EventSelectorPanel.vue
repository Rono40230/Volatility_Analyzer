<template>
  <div v-if="!eventImpact && !loading" class="welcome">
    <div class="welcome-icon">🎯</div>
    <h3>Analyse Rétrospective par Événement</h3>
    <div class="welcome-select-container">
      <div class="dropdown-group">
        <label for="event-select">📅 Événements (HIGH & MEDIUM Impact) :</label>
        <select id="event-select" :value="modelValue" class="welcome-symbol-select" @change="$emit('update:modelValue', $event.target.value); $emit('load')">
          <option value="">Choisir un événement</option>
          <option v-for="event in pastEvents" :key="`event-${event.name}`" :value="event.name">
            {{ getEventLabel(event.name) }} ({{ event.count }} occurrences)
          </option>
        </select>
      </div>
    </div>
  </div>

  <div v-else-if="eventImpact" class="event-info-card">
    <div class="event-header">
      <div class="event-title-with-selector">
        <h3>{{ getEventLabel(eventImpact.event_name) }} <span class="inline-info">| 🌍 {{ eventImpact.country }} | 💱 {{ eventImpact.currency }}</span></h3>
        <select :value="modelValue" class="inline-select" @change="$emit('update:modelValue', $event.target.value); $emit('load')">
          <option value="">Changer d'événement</option>
          <option v-for="event in pastEvents" :key="`event-${event.name}`" :value="event.name">
            {{ getEventLabel(event.name) }}
          </option>
        </select>
      </div>
      <button class="btn-archive" @click="$emit('archive')">💾 Archiver</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { EventImpactResult } from '../composables/useEventCorrelationByEvent'

interface PastEvent {
  name: string
  count: number
}

defineProps<{
  modelValue: string
  pastEvents: PastEvent[]
  eventImpact: EventImpactResult | null
  loading: boolean
  getEventLabel: (name: string) => string
}>()

defineEmits<{
  'update:modelValue': [value: string]
  load: []
  archive: []
}>()
</script>

<style scoped>
.welcome { text-align: center; padding: 40px 20px; }
.welcome-icon { font-size: 3em; margin-bottom: 20px; }
.welcome h3 { color: #e6edf3; margin: 20px 0; }
.welcome-select-container { display: flex; justify-content: center; margin: 20px 0; }
.dropdown-group { display: flex; flex-direction: column; gap: 10px; align-items: center; }
.dropdown-group label { color: #e6edf3; font-weight: 600; }
.welcome-symbol-select, .inline-select { padding: 8px 12px; border-radius: 6px; border: 1px solid #30363d; background: #0d1117; color: #e6edf3; font-size: 0.95em; }
.event-info-card { padding: 20px; background: rgba(56, 139, 253, 0.05); border-radius: 12px; border: 1px solid #30363d; margin-bottom: 20px; }
.event-header { display: flex; justify-content: space-between; align-items: center; }
.event-title-with-selector { display: flex; gap: 15px; align-items: center; flex: 1; }
.event-title-with-selector h3 { margin: 0; color: #f0f6fc; }
.inline-info { color: #8b949e; font-size: 0.9em; font-weight: 400; }
.inline-select { padding: 6px 10px; font-size: 0.9em; }
.btn-archive { padding: 8px 16px; background: #238636; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; }
.btn-archive:hover { background: #2ea043; }
</style>
