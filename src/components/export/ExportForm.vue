<template>
  <div class="export-options">
    <h3>1. Sélectionner les rapports</h3>
    <div class="checkbox-group">
      <label class="checkbox-label">
        <input type="checkbox" :checked="selectedReports.includes('backtest')" @change="toggleReport('backtest')">
        <span class="label-text">Rapport de Backtest</span>
        <span class="label-desc">Performance (Win Rate, Drawdown, Equity Curve)</span>
      </label>

      <label class="checkbox-label">
        <input type="checkbox" :checked="selectedReports.includes('bidi_period')" @change="toggleReport('bidi_period')">
        <span class="label-text">Fiche Bidi : Paire/Période</span>
        <span class="label-desc">Volatilité Brute (Session Trading)</span>
      </label>

      <label class="checkbox-label">
        <input type="checkbox" :checked="selectedReports.includes('bidi_event')" @change="toggleReport('bidi_event')">
        <span class="label-text">Fiche Bidi : Paire/Événements</span>
        <span class="label-desc">Corrélation (News Trading)</span>
      </label>
      
      <label class="checkbox-label">
        <input type="checkbox" :checked="selectedReports.includes('ranking')" @change="toggleReport('ranking')">
        <span class="label-text">Classement des Opportunités</span>
        <span class="label-desc">Top des meilleures configurations par score</span>
      </label>
    </div>

    <h3>2. Configuration</h3>
    <div class="filters-group">
      <div class="filter-item">
        <label>Calendrier Économique</label>
        <select :value="selectedCalendarId" @change="$emit('update:selectedCalendarId', Number(($event.target as HTMLSelectElement).value))" class="select-input">
          <option v-for="cal in calendars" :key="cal.id" :value="cal.id">
            {{ cal.name }} ({{ formatPeriod(cal) }})
          </option>
        </select>
      </div>
      <div class="filter-item">
        <label>Paires à analyser</label>
        <select :value="selectedPairMode" @change="$emit('update:selectedPairMode', ($event.target as HTMLSelectElement).value)" class="select-input">
          <option value="all">Toutes les paires ({{ totalPairs }})</option>
          <option disabled>──────────</option>
          <option v-for="symbol in symbols" :key="symbol.symbol" :value="symbol.symbol">
            {{ symbol.symbol }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SymbolInfo } from '../../stores/volatilityTypes'

export interface CalendarMetadata {
  id: number
  name: string
  event_count: number
  start_date?: string
  end_date?: string
}

const props = defineProps<{
  selectedReports: string[]
  selectedCalendarId: number | null
  selectedPairMode: string
  calendars: CalendarMetadata[]
  symbols: SymbolInfo[]
}>()

const emit = defineEmits<{
  (e: 'update:selectedReports', value: string[]): void
  (e: 'update:selectedCalendarId', value: number): void
  (e: 'update:selectedPairMode', value: string): void
}>()

const totalPairs = props.symbols.length

function toggleReport(report: string) {
  const newReports = [...props.selectedReports]
  const index = newReports.indexOf(report)
  if (index === -1) {
    newReports.push(report)
  } else {
    newReports.splice(index, 1)
  }
  emit('update:selectedReports', newReports)
}

function formatPeriod(cal: CalendarMetadata): string {
  if (!cal.start_date || !cal.end_date) return '?'
  const start = new Date(cal.start_date).toLocaleDateString('fr-FR', { month: 'short', year: '2-digit' })
  const end = new Date(cal.end_date).toLocaleDateString('fr-FR', { month: 'short', year: '2-digit' })
  return `${start} - ${end}`
}
</script>

<style scoped>
.export-options h3 {
  color: #4a9eff;
  font-size: 1rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.export-options h3:not(:first-child) {
  margin-top: 1.5rem;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.checkbox-label {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 0.75rem;
  background: #252525;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.checkbox-label:hover {
  background: #2a2a2a;
}

.checkbox-label input {
  margin-top: 0.25rem;
}

.label-text {
  font-weight: 500;
  color: #e0e0e0;
  display: block;
}

.label-desc {
  font-size: 0.85rem;
  color: #888;
  display: block;
  margin-top: 0.25rem;
}

.filters-group {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
}

.filter-item label {
  display: block;
  color: #aaa;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}

.select-input {
  background: #fff;
  border: 1px solid #333;
  color: #000;
  padding: 0.5rem;
  border-radius: 4px;
  width: 100%;
}

select option {
  background: #fff;
  color: #000;
}
</style>
