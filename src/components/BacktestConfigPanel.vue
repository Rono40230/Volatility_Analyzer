<script setup lang="ts">
import { useBacktestConfig } from '../composables/useBacktestConfig'
import { BacktestType } from '../stores/backtest'
import SymbolSelector from './SymbolSelector.vue'
import SearchableEventDropdown from './SearchableEventDropdown.vue'

const props = defineProps<{
  backtestType: BacktestType
}>()

const {
  config,
  loading,
  selectedSymbol,
  selectedEvent,
  selectedTime,
  startDate,
  endDate,
  availableEvents,
  lancerBacktest
} = useBacktestConfig(props)

import { computed } from 'vue'
import '../styles/backtest-config.css'

const isRunDisabled = computed(() => {
  if (loading.value) return true
  if (!selectedSymbol.value) return true
  if (props.backtestType === BacktestType.Event && !selectedEvent.value) return true
  return false
})

const disabledReason = computed(() => {
  if (loading.value) return 'Simulation en cours...'
  if (!selectedSymbol.value) return 'Veuillez sélectionner une paire'
  if (props.backtestType === BacktestType.Event && !selectedEvent.value) return 'Veuillez sélectionner un événement'
  return 'Prêt à lancer'
})
</script>

<template>
  <div class="config-panel">
    
    <div class="params-grid">
      <!-- Row 1: Selection -->
      <div class="param col-span-1">
        <label>Paire</label>
        <SymbolSelector v-model="selectedSymbol" />
      </div>

      <!-- Mode Événement -->
      <div v-if="backtestType === BacktestType.Event" class="param col-span-4">
        <label>Événement</label>
        <SearchableEventDropdown 
          v-model="selectedEvent" 
          :events="availableEvents"
        />
      </div>

      <!-- Mode Horaire -->
      <template v-else>
        <div class="param col-span-1">
          <label>Heure (UTC)</label>
          <input type="time" v-model="selectedTime" class="time-input" />
        </div>
        <div class="param col-span-1">
          <label>Début</label>
          <input type="date" v-model="startDate" class="date-input" />
        </div>
        <div class="param col-span-1">
          <label>Fin</label>
          <input type="date" v-model="endDate" class="date-input" />
        </div>
        <!-- Spacer -->
        <div class="param col-span-1"></div>
      </template>

      <div class="param col-span-6"></div>

      <!-- Row 2: Parameters -->
      <div class="param col-span-1">
        <label>Offset (pips)</label>
        <input type="number" v-model.number="config.offset_pips" step="0.1" />
      </div>
      <div class="param col-span-1">
        <label>Stop Loss (pips)</label>
        <input type="number" v-model.number="config.stop_loss_pips" step="0.1" />
      </div>
      <div class="param col-span-1">
        <label>Timeout (min)</label>
        <input type="number" v-model.number="config.timeout_minutes" />
      </div>
      <div class="param col-span-1">
        <label>Spread (pips)</label>
        <input type="number" v-model.number="config.spread_pips" step="0.1" />
      </div>
      <div class="param col-span-1">
        <label>Slippage (pips)</label>
        <input type="number" v-model.number="config.slippage_pips" step="0.1" title="Glissement estimé à l'exécution" />
      </div>
      
      <!-- Paramètres spécifiques Simultané (Aucun pour l'instant) -->
    </div>

    <div class="actions">
      <button 
        class="run-btn" 
        @click="lancerBacktest" 
        :disabled="isRunDisabled"
        :title="disabledReason"
      >
        <span v-if="loading">⏳ Simulation...</span>
        <span v-else>🚀 Lancer Backtest</span>
      </button>
    </div>
  </div>
</template>

