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
  configMode,
  loading,
  selectedSymbol,
  selectedEvent,
  selectedTime,
  startDate,
  endDate,
  availableEvents,
  lancerBacktest,
  appliquerParamsAuto
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
      <div v-if="backtestType === BacktestType.Event" class="param col-span-3">
        <label>Événement</label>
        <SearchableEventDropdown 
          v-model="selectedEvent" 
          :events="availableEvents"
        />
      </div>
      <div v-if="backtestType === BacktestType.Event" class="param col-span-1">
        <label>Écart (pips)</label>
        <input type="number" v-model.number="config.spread_pips" step="0.1" />
      </div>
      <div v-if="backtestType === BacktestType.Event" class="param col-span-1">
        <label>Glissement (pips)</label>
        <input type="number" v-model.number="config.slippage_pips" step="0.1" title="Glissement estimé à l'exécution" />
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
        <label>TP (R)</label>
        <input type="number" v-model.number="config.tp_rr" step="0.5" min="3" max="10" />
      </div>
      <div class="param col-span-1">
        <label>R (pips)</label>
        <input type="number" v-model.number="config.stop_loss_pips" step="0.1" />
      </div>
      <div class="param col-span-1">
        <label>Stop suiveur (ATR x)</label>
        <input type="number" v-model.number="config.trailing_atr_coef" step="0.1" min="0.5" max="5" />
      </div>
      <div class="param col-span-1">
        <label>Période ATR</label>
        <input type="number" v-model.number="config.atr_period" step="1" min="2" max="50" />
      </div>
      <div class="param col-span-1">
        <label>Rafraichissement TS (sec)</label>
        <input type="number" v-model.number="config.trailing_refresh_seconds" step="1" min="0" />
      </div>
      <div class="param col-span-1">
        <label>Délai max (min)</label>
        <input type="number" v-model.number="config.timeout_minutes" />
      </div>
      <div class="param col-span-1"></div>
      <div class="param col-span-1"></div>
      
      <!-- Paramètres spécifiques Simultané (Aucun pour l'instant) -->
    </div>

    <div class="actions">
      <button 
        class="auto-btn" 
        @click="appliquerParamsAuto" 
        :disabled="!selectedSymbol || loading"
        :title="!selectedSymbol ? 'Sélectionnez une paire d\'abord' : 'Appliquer les paramètres recommandés'"
      >
        📚 Params Auto
      </button>
      <span v-if="configMode === 'auto'" class="auto-badge">✅ Paramètres recommandés appliqués</span>
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

