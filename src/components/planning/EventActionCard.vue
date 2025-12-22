<script setup lang="ts">
import { ref, computed } from 'vue'

interface EventAction {
  id: string
  time: string
  name: string
  currency: string
  impact: 'High' | 'Medium' | 'Low'
  offset: number
  tp: number
  sl: number
  confidenceScore: number
}

const props = defineProps<{
  event: EventAction
}>()

const emit = defineEmits<{
  (e: 'update', id: string, field: string, value: number): void
}>()

// État local pour l'édition (si on veut valider avant d'émettre)
// Pour l'instant on émet directement

const impactColor = computed(() => {
  switch (props.event.impact) {
    case 'High': return 'var(--danger-color)'
    case 'Medium': return 'var(--warning-color)'
    default: return 'var(--text-secondary)'
  }
})

const confidenceColor = computed(() => {
  if (props.event.confidenceScore >= 80) return 'var(--success-color)'
  if (props.event.confidenceScore >= 50) return 'var(--warning-color)'
  return 'var(--danger-color)'
})
</script>

<template>
  <div class="event-card">
    <div class="event-header">
      <div class="time-badge">{{ event.time }}</div>
      <div class="event-info">
        <div class="event-title">
          <span class="currency">{{ event.currency }}</span>
          <span class="name">{{ event.name }}</span>
        </div>
        <div class="indicators">
          <span class="impact-dot" :style="{ backgroundColor: impactColor }" :title="'Impact ' + event.impact"></span>
          <span class="confidence-score" :style="{ color: confidenceColor }">
            {{ event.confidenceScore }}% Confiance
          </span>
        </div>
      </div>
    </div>

    <div class="action-params">
      <div class="param-group">
        <label>Offset</label>
        <div class="input-wrapper">
          <input 
            type="number" 
            :value="event.offset"
            @input="emit('update', event.id, 'offset', Number(($event.target as HTMLInputElement).value))"
            step="0.1"
          >
          <span class="unit">pips</span>
        </div>
      </div>

      <div class="param-group">
        <label>TP</label>
        <div class="input-wrapper">
          <input 
            type="number" 
            :value="event.tp"
            @input="emit('update', event.id, 'tp', Number(($event.target as HTMLInputElement).value))"
            step="1"
          >
          <span class="unit">pts</span>
        </div>
      </div>

      <div class="param-group">
        <label>SL</label>
        <div class="input-wrapper">
          <input 
            type="number" 
            :value="event.sl"
            @input="emit('update', event.id, 'sl', Number(($event.target as HTMLInputElement).value))"
            step="1"
          >
          <span class="unit">pts</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.event-card {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  transition: transform 0.2s, box-shadow 0.2s;
}

.event-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-color: var(--primary-color);
}

.event-header {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
}

.time-badge {
  background: var(--bg-secondary);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.event-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.event-title {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.currency {
  font-weight: 700;
  color: var(--primary-color);
  font-size: 0.9rem;
}

.name {
  font-weight: 500;
  font-size: 0.95rem;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 180px;
}

.indicators {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
}

.impact-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.confidence-score {
  font-weight: 600;
}

.action-params {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 0.5rem;
  background: var(--bg-secondary);
  padding: 0.5rem;
  border-radius: 6px;
}

.param-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.param-group label {
  font-size: 0.7rem;
  color: var(--text-secondary);
  text-transform: uppercase;
}

.input-wrapper {
  display: flex;
  align-items: baseline;
  gap: 0.25rem;
}

.input-wrapper input {
  width: 100%;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-weight: 600;
  font-size: 0.9rem;
  padding: 0 0 2px 0;
  text-align: right;
}

.input-wrapper input:focus {
  outline: none;
  border-bottom-color: var(--primary-color);
}

.unit {
  font-size: 0.7rem;
  color: var(--text-secondary);
}
</style>
