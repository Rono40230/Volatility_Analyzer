<template>
  <div class="table-wrapper">
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
          <td>
            <select v-if="allCountries && allCountries.length > 0" 
              :value="ev.symbol" 
              @change="$emit('update-symbol', ev, ($event.target as HTMLSelectElement).value)"
              class="symbol-select"
            >
              <option v-for="c in allCountries" :key="c.symbol" :value="c.symbol">
                {{ c.symbol }}
              </option>
              <option v-if="!allCountries.find(c => c.symbol === ev.symbol)" :value="ev.symbol">
                {{ ev.symbol }}
              </option>
            </select>
            <span v-else>{{ ev.symbol }}</span>
          </td>
          <td>
            <span class="impact-badge" :class="ev.impact.toLowerCase()">{{ ev.impact }}</span>
          </td>
          <td>{{ ev.label || ev.description }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { CalendarEvent, CurrencySummary } from '../../types/cleanup'

defineProps<{
  events: CalendarEvent[]
  allCountries?: CurrencySummary[]
}>()

defineEmits(['update-symbol'])

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

.symbol-select {
  background: #ffffff;
  color: #000000;
  border: 1px solid #475569;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 0.9rem;
  cursor: pointer;
}

.symbol-select:focus {
  outline: none;
  border-color: #3b82f6;
}
</style>
