<script setup lang="ts">
import type { TradeResult } from '../../stores/backtest'
import UnitDisplay from '../UnitDisplay.vue'
import MetricTooltip from '../MetricTooltip.vue'

const props = defineProps<{
  trades: TradeResult[]
  unit: string
  symbol?: string
}>()

function formatDate(iso: string) {
  return new Date(iso).toLocaleString('fr-FR')
}

function getOutcomeClass(outcome: string) {
  switch (outcome) {
    case 'TakeProfit': return 'outcome-win'
    case 'RecoveryWin': return 'outcome-recovery'
    case 'StopLoss': return 'outcome-loss'
    case 'DoubleLoss': return 'outcome-loss'
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
  
  if (logs.includes('SL Short')) icons += 'S:🛑 '
  if (logs.includes('Timeout Short')) icons += 'S:⏰ '
  
  if (trade.outcome === 'TakeProfit' && !icons.includes('⏰')) {
    // Si gagnant sans timeout explicite, on suppose un TP/TS
    // Mais en simultané, c'est souvent un mix. 
    // On affiche juste ce qu'on sait. Si vide, on met un check
  }

  if (icons === '') {
     if (trade.outcome === 'TakeProfit') return '🎯'
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
                <MetricTooltip title="Excursions de Prix">
                  <span class="info-icon">ℹ️</span>
                  <template #definition>
                    <p><strong>MFE (Maximum Favorable Excursion)</strong> : Le point le plus favorable atteint par le prix pendant le trade (Gain Max Latent).</p>
                    <p><strong>MAE (Maximum Adverse Excursion)</strong> : Le point le plus défavorable atteint par le prix pendant le trade (Perte Max Latente).</p>
                  </template>
                  <template #interpretation>
                    <p>Ces métriques mesurent la qualité de l'entrée et de la sortie :</p>
                    <ul>
                      <li>Un <strong>MFE élevé</strong> avec un gain final faible suggère une sortie tardive (TP raté ou Trailing Stop trop large).</li>
                      <li>Un <strong>MAE élevé</strong> signifie que le trade à frôlé le Stop Loss (Entrée risquée).</li>
                    </ul>
                  </template>
                </MetricTooltip>
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
              <MetricTooltip title="Journal d'exécution">
                <span class="log-icon">📝</span>
                <template #definition>
                  <ul class="log-list">
                    <li v-for="(log, i) in formatLogs(trade.logs)" :key="i">
                      {{ log }}
                    </li>
                    <li v-if="trade.logs.length === 0" class="log-empty">Aucun détail disponible</li>
                  </ul>
                </template>
              </MetricTooltip>
            </td>
          </tr>
        </tbody>
      </table>
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
  cursor: help;
  opacity: 0.6;
  transition: opacity 0.2s;
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
  cursor: help;
  font-size: 1.1em;
  opacity: 0.7;
  transition: opacity 0.2s;
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
</style>
