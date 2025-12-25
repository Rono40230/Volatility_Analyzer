<template>
  <div class="tab-content preview-mode">
    <div class="controls-bar">
      <div class="control-group">
        <button class="btn-back" @click="$emit('close')">← Retour</button>
        <span class="preview-title">Aperçu : <strong>{{ title }}</strong></span>
      </div>
      <div class="stats-badge">
        {{ events.length }} événements (max 100)
      </div>
    </div>

    <div v-if="loading" class="state-container">
      <div class="spinner"></div>
      <p>Chargement de l'aperçu...</p>
    </div>

    <div v-else class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Symbole</th>
            <th>Impact</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="ev in events" :key="ev.id">
            <td class="text-nowrap">{{ formatDateTime(ev.event_time) }}</td>
            <td>{{ ev.symbol }}</td>
            <td>
              <span class="impact-badge" :class="ev.impact.toLowerCase()">{{ ev.impact }}</span>
            </td>
            <td>{{ ev.description }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'

interface CalendarEvent {
  id: number
  symbol: string
  event_time: string
  impact: string
  description: string
  actual: number | null
  forecast: number | null
  previous: number | null
}

defineProps<{
  events: CalendarEvent[]
  title: string
  loading: boolean
}>()

defineEmits(['close'])

function formatDateTime(dt: string) {
  try {
    return new Date(dt).toLocaleString('fr-FR', { 
      year: 'numeric', month: '2-digit', day: '2-digit', 
      hour: '2-digit', minute: '2-digit' 
    })
  } catch { return dt }
}
</script>

<style scoped>
.preview-mode .controls-bar {
  background: #1e293b;
  border-bottom: 1px solid #334155;
  border-radius: 0;
  margin: -24px -24px 20px -24px;
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-back {
  background: transparent;
  border: 1px solid #475569;
  color: #cbd5e1;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.btn-back:hover {
  background: #334155;
  color: #fff;
}

.preview-title {
  color: #e2e8f0;
  font-size: 0.95rem;
}

.stats-badge {
  background: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.875rem;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.state-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  gap: 16px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  border-top-color: #f59e0b;
  animation: spin 1s linear infinite;
}

.table-wrapper {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #334155;
  border-radius: 8px;
  background: #0f172a;
  min-height: 0;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th {
  background: #1e293b;
  color: #94a3b8;
  font-weight: 600;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 12px 16px;
  position: sticky;
  top: 0;
  border-bottom: 1px solid #334155;
}

td {
  padding: 12px 16px;
  border-bottom: 1px solid #1e293b;
  color: #e2e8f0;
  font-size: 0.9rem;
}

tr:last-child td {
  border-bottom: none;
}

tr:hover td {
  background: rgba(255, 255, 255, 0.02);
}

.text-nowrap {
  white-space: nowrap;
}

.impact-badge {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.impact-badge.high { background: rgba(239, 68, 68, 0.2); color: #fca5a5; }
.impact-badge.medium { background: rgba(245, 158, 11, 0.2); color: #fcd34d; }
.impact-badge.low { background: rgba(59, 130, 246, 0.2); color: #93c5fd; }

@keyframes spin { to { transform: rotate(360deg); } }
</style>
