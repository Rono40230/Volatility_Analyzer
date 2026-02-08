<script setup lang="ts">
import type { TradeResult } from '../../stores/backtest'
import UnitDisplay from '../UnitDisplay.vue'
import { computed, ref } from 'vue'

const props = defineProps<{
  trades: TradeResult[]
  unit: string
  symbol?: string
}>()

const showMfeHelp = ref(false)
const openLogIndex = ref<number | null>(null)

const activeLogs = computed(() => {
  if (openLogIndex.value == null) return []
  const trade = props.trades[openLogIndex.value]
  if (!trade) return []
  return formatLogs(trade.logs)
})

const activeHasLogs = computed(() => {
  if (openLogIndex.value == null) return false
  const trade = props.trades[openLogIndex.value]
  return !!trade && trade.logs.length > 0
})

function formatDate(iso: string) {
  return new Date(iso).toLocaleString('fr-FR')
}

function getOutcomeClass(outcome: string) {
  switch (outcome) {
    case 'TakeProfit': return 'outcome-win'
    case 'TrailingStop': return 'outcome-win'
    case 'StopLoss': return 'outcome-loss'
    case 'Timeout': return 'outcome-neutral'
    case 'NoEntry': return 'outcome-neutral'
    default: return ''
  }
}

function getSequenceIcons(trade: TradeResult): string {
  let icons = ''
  const logs = trade.logs.join(' ')
  
  if (logs.includes('SL Long')) icons += 'L:🛑 '
  if (logs.includes('Timeout Long')) icons += 'L:⏰ '
  if (logs.includes('TS Long')) icons += 'L:🧭 '
  
  if (logs.includes('SL Short')) icons += 'S:🛑 '
  if (logs.includes('Timeout Short')) icons += 'S:⏰ '
  if (logs.includes('TS Short')) icons += 'S:🧭 '
  
  if (trade.outcome === 'TakeProfit' && !icons.includes('⏰')) {
    // TP(R) réellement atteint
  }
  if (trade.outcome === 'TrailingStop' && !icons.includes('⏰')) {
    // Sortie trailing profitable (TP non atteint)
  }

  if (icons === '') {
     if (trade.outcome === 'TakeProfit') return '🎯'
     if (trade.outcome === 'TrailingStop') return '🧭'
     if (trade.outcome === 'StopLoss') return '🛑'
     if (trade.outcome === 'Timeout') return '⏰'
     if (trade.outcome === 'NoEntry') return '—'
  }

  return icons.trim()
}

function formatLogs(logs: string[]): string[] {
  return logs.map(log => {
      // Nettoyer les emojis existants pour éviter les doublons ou formater proprement
      let clean = log.replace('💥 ', '').replace('⏰ ', '')
      // Ajouter nos propres puces si besoin
      if (log.includes('SL')) return `🛑 ${clean}`
      if (log.includes('TS')) return `🧭 ${clean}`
      if (log.includes('Timeout')) return `⏱️ ${clean}`
      return `ℹ️ ${clean}`
  })
}
</script>

<template>
  <div class="trades-list">
    <h3>Journal des Trades</h3>
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Date Événement</th>
            <th>Entrée</th>
            <th>Sortie</th>
            <th>Durée</th>
            <th>Pips Net</th>
            <th>Résultat</th>
            <th>
              <div class="header-with-tooltip">
                MFE / MAE
                <button type="button" class="info-icon" @click="showMfeHelp = true">ℹ️</button>
              </div>
            </th>
            <th>Séquence</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(trade, index) in trades" :key="index">
            <td>{{ formatDate(trade.event_date) }}</td>
            <td>{{ trade.entry_time ? formatDate(trade.entry_time).split(' ')[1] : '-' }}</td>
            <td>{{ trade.exit_time ? formatDate(trade.exit_time).split(' ')[1] : '-' }}</td>
            <td>{{ trade.duration_minutes }}m</td>
            <td :class="trade.pips_net > 0 ? 'win' : (trade.pips_net < 0 ? 'loss' : 'neutral')">
              <UnitDisplay :value="trade.pips_net" :unit="unit" :decimals="1" :symbol="symbol" />
            </td>
            <td>
              <span :class="['outcome-badge', getOutcomeClass(trade.outcome)]">
                {{ trade.outcome }}
              </span>
            </td>
            <td class="excursion">
              <span class="mfe" title="Max Favorable Excursion">+<UnitDisplay :value="trade.max_favorable_excursion" :unit="unit" :decimals="1" :symbol="symbol" /></span> / 
              <span class="mae" title="Max Adverse Excursion">-<UnitDisplay :value="trade.max_adverse_excursion" :unit="unit" :decimals="1" :symbol="symbol" /></span>
            </td>
            <td class="sequence-cell">
              <span class="sequence-icons">{{ getSequenceIcons(trade) }}</span>
              <button type="button" class="log-icon" @click="openLogIndex = index">📝</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="showMfeHelp" class="mini-modal-overlay" @click.self="showMfeHelp = false">
      <div class="mini-modal">
        <div class="mini-modal-header">
          <div class="mini-modal-title">Excursions de Prix</div>
          <button class="mini-modal-close" type="button" @click="showMfeHelp = false">✕</button>
        </div>
        <div class="mini-modal-body">
          <div class="mini-modal-section">
            <div class="mini-modal-section-title">Definition</div>
            <p><strong>MFE (Maximum Favorable Excursion)</strong> : Le point le plus favorable atteint par le prix pendant le trade (Gain Max Latent).</p>
            <p><strong>MAE (Maximum Adverse Excursion)</strong> : Le point le plus defavorable atteint par le prix pendant le trade (Perte Max Latente).</p>
          </div>
          <div class="mini-modal-section">
            <div class="mini-modal-section-title">Interpretation</div>
            <p>Ces metriques mesurent la qualite de l'entree et de la sortie :</p>
            <ul class="mini-modal-list">
              <li>Un <strong>MFE eleve</strong> avec un gain final faible suggere une sortie tardive (TP rate ou Trailing Stop trop large).</li>
              <li>Un <strong>MAE eleve</strong> signifie que le trade a frole le Stop Loss (Entree risquee).</li>
            </ul>
          </div>
        </div>
      </div>
    </div>

    <div v-if="openLogIndex !== null" class="mini-modal-overlay" @click.self="openLogIndex = null">
      <div class="mini-modal">
        <div class="mini-modal-header">
          <div class="mini-modal-title">Journal d'execution</div>
          <button class="mini-modal-close" type="button" @click="openLogIndex = null">✕</button>
        </div>
        <div class="mini-modal-body">
          <ul class="log-list">
            <li v-for="(log, i) in activeLogs" :key="i">
              {{ log }}
            </li>
            <li v-if="!activeHasLogs" class="log-empty">Aucun detail disponible</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.trades-list {
  background: #1a202c;
  padding: 1.5rem;
  border-radius: 8px;
  border: 1px solid #2d3748;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.trades-list h3 {
  margin: 0 0 1rem 0;
  color: #e2e8f0;
}

.table-container {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #2d3748;
  border-radius: 6px;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

th, td {
  padding: 0.75rem 1rem;
  text-align: left;
  border-bottom: 1px solid #2d3748;
}

th {
  background: #2d3748;
  color: #a0aec0;
  font-weight: 600;
  position: sticky;
  top: 0;
}

tr:hover {
  background: #2d3748;
}

.win { color: #48bb78; }
.loss { color: #f56565; }
.neutral { color: #a0aec0; }

.outcome-badge {
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

.outcome-win { background: rgba(72, 187, 120, 0.2); color: #48bb78; }
.outcome-recovery { background: rgba(66, 153, 225, 0.2); color: #4299e1; }
.outcome-loss { background: rgba(245, 101, 101, 0.2); color: #f56565; }
.outcome-neutral { background: rgba(160, 174, 192, 0.2); color: #a0aec0; }

.excursion {
  font-family: monospace;
  font-size: 0.85rem;
}

.mfe { color: #48bb78; }
.mae { color: #f56565; }

.header-with-tooltip {
  display: flex;
  align-items: center;
  gap: 8px;
}

.info-icon {
  font-size: 1.1em;
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 0.2s;
  background: transparent;
  border: none;
  color: inherit;
  padding: 0;
}

.info-icon:hover {
  opacity: 1;
}

.sequence-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  justify-content: space-between;
}

.sequence-icons {
  font-size: 0.85em;
  font-family: 'Segoe UI Emoji', 'Noto Color Emoji', sans-serif;
}

.log-icon {
  cursor: pointer;
  font-size: 1.1em;
  opacity: 0.7;
  transition: opacity 0.2s;
  background: transparent;
  border: none;
  color: inherit;
  padding: 0;
}

.log-icon:hover {
  opacity: 1;
  transform: scale(1.1);
}

.log-list {
  list-style: none;
  padding: 0;
  margin: 0;
  text-align: left;
}

.log-list li {
  padding: 4px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  font-size: 0.9em;
}

.log-list li:last-child {
  border-bottom: none;
}

.log-empty {
  color: #a0aec0;
  font-style: italic;
}

.mini-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
}

.mini-modal {
  width: 420px;
  max-width: 90vw;
  background: #1a202c;
  border: 1px solid #2d3748;
  border-radius: 10px;
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.4);
}

.mini-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #2d3748;
}

.mini-modal-title {
  color: #e2e8f0;
  font-weight: 600;
  font-size: 0.95rem;
}

.mini-modal-close {
  background: transparent;
  border: none;
  color: #a0aec0;
  font-size: 1.1rem;
  cursor: pointer;
}

.mini-modal-body {
  padding: 14px 16px 18px;
  color: #e2e8f0;
  font-size: 0.92rem;
  line-height: 1.4;
}

.mini-modal-section {
  margin-bottom: 12px;
}

.mini-modal-section-title {
  text-transform: uppercase;
  font-size: 0.75rem;
  color: #90cdf4;
  letter-spacing: 0.4px;
  margin-bottom: 6px;
}

.mini-modal-list {
  margin: 6px 0 0 16px;
}
</style>
