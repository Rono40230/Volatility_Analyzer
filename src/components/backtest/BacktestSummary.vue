<script setup lang="ts">
import { computed } from 'vue'
import MetricTooltip from '../MetricTooltip.vue'
import UnitDisplay from '../UnitDisplay.vue'
import type { BacktestResult } from '../../stores/backtest'

const props = defineProps<{
  result: BacktestResult
}>()

const winRateClass = computed(() => {
  return props.result.win_rate_percent > 50 ? 'metric-excellent' : 'metric-poor'
})

const pfClass = computed(() => {
  const pf = props.result.profit_factor
  if (pf > 2.0) return 'metric-excellent'
  if (pf > 1.5) return 'metric-good'
  if (pf > 1.0) return 'metric-acceptable'
  return 'metric-poor'
})

const plClass = computed(() => {
  return props.result.total_pips >= 0 ? 'metric-excellent' : 'metric-poor'
})

const avgPlClass = computed(() => {
  return props.result.average_pips_per_trade >= 0 ? 'metric-excellent' : 'metric-poor'
})

const ddClass = computed(() => {
  return 'metric-poor' 
})
</script>

<template>
  <div class="metrics-grid">
    <MetricTooltip title="Profit Factor">
      <div class="metric-card">
        <h4>Profit Factor</h4>
        <div :class="['metric-value', pfClass]">{{ result.profit_factor.toFixed(2) }}</div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Ratio : (Somme des Gains Bruts) / (Somme des Pertes Brutes).</div>
        </div>
      </template>
      <template #usage>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📊 Interprétation</div>
          <div class="tooltip-section-text">
            <strong>< 1.0</strong> : Stratégie perdante.<br>
            <strong>1.0 - 1.5</strong> : Rentable mais fragile.<br>
            <strong>1.5 - 2.0</strong> : Bonne stratégie.<br>
            <strong>> 2.0</strong> : Excellente stratégie.
          </div>
        </div>
      </template>
    </MetricTooltip>

    <div class="metric-card">
      <h4>Trades</h4>
      <div class="metric-value metric-neutral">
        {{ result.total_trades }}
        <span class="metric-breakdown">
          <span class="win">{{ result.winning_trades }}W</span>
          <span class="divider">/</span>
          <span class="loss">{{ result.losing_trades }}L</span>
          <span class="divider">/</span>
          <span class="neutral">{{ result.no_entries }}N</span>
        </span>
      </div>
    </div>

    <MetricTooltip title="Win Rate">
      <div class="metric-card">
        <h4>Win Rate</h4>
        <div :class="['metric-value', winRateClass]">{{ result.win_rate_percent.toFixed(1) }}%</div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Pourcentage de trades gagnants par rapport au nombre total de trades.</div>
        </div>
      </template>
      <template #usage>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📊 Interprétation</div>
          <div class="tooltip-section-text">
            Un Win Rate faible n'est pas grave si le Profit Factor est élevé (stratégie de type "Sniper").
            <br><br>
            <strong>> 50%</strong> : Majorité de gains.<br>
            <strong>< 40%</strong> : Typique des stratégies de suivi de tendance (beaucoup de petites pertes, quelques gros gains).
          </div>
        </div>
      </template>
    </MetricTooltip>

    <MetricTooltip title="Total P/L">
      <div class="metric-card">
        <h4>Total P/L</h4>
        <div :class="['metric-value', plClass]">
          {{ result.total_pips > 0 ? '+' : '' }}{{ result.total_pips.toFixed(1) }} <span style="font-size: 0.6em">pips</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Résultat net cumulé en pips (Gains - Pertes).</div>
        </div>
      </template>
    </MetricTooltip>

    <MetricTooltip title="P/L par Trade">
      <div class="metric-card">
        <h4>P/L par Trade</h4>
        <div :class="['metric-value', avgPlClass]">
          {{ result.average_pips_per_trade > 0 ? '+' : '' }}{{ result.average_pips_per_trade.toFixed(1) }} <span style="font-size: 0.6em">pips</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Moyenne des gains/pertes par trade (Espérance mathématique).</div>
        </div>
      </template>
    </MetricTooltip>

    <MetricTooltip title="Max Drawdown">
      <div class="metric-card">
        <h4>Max Drawdown</h4>
        <div :class="['metric-value', ddClass]">-{{ result.max_drawdown_pips.toFixed(1) }}</div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">La plus grande baisse cumulée du capital (du plus haut sommet au plus bas creux).</div>
        </div>
      </template>
    </MetricTooltip>
  </div>
</template>

<style scoped>
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.metric-card {
  background: #2d3748;
  padding: 1rem;
  border-radius: 8px;
  text-align: center;
  border: 1px solid #4a5568;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 100px; /* Ensure consistent height */
}

.metric-card h4 {
  margin: 0 0 0.5rem 0;
  color: #a0aec0;
  font-size: 0.9rem;
}

.metric-value {
  font-size: 1.5rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.metric-breakdown {
  font-size: 0.85rem;
  font-weight: 500;
  color: #a0aec0;
  display: flex;
  align-items: center;
  margin-top: 4px; /* Slight adjustment for visual alignment */
}

.divider {
  margin: 0 3px;
  color: #4a5568;
}

.metric-sub {
  font-size: 0.8rem;
  margin-top: 0.2rem;
  color: #a0aec0;
}

.metric-excellent { color: #48bb78; }
.metric-good { color: #68d391; }
.metric-acceptable { color: #ecc94b; }
.metric-neutral { color: #e2e8f0; }
.metric-poor { color: #f56565; }

.win { color: #48bb78; }
.loss { color: #f56565; }
.neutral { color: #a0aec0; }

.tooltip-section {
  margin-bottom: 12px;
}

.tooltip-section:last-child {
  margin-bottom: 0;
}

.tooltip-section-title {
  font-weight: 600;
  color: #90cdf4;
  margin-bottom: 4px;
  font-size: 0.9em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.tooltip-section-text {
  color: #e2e8f0;
  line-height: 1.4;
  font-size: 0.95em;
}
</style>
