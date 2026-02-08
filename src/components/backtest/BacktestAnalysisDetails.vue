<template>
  <div class="analysis-details">
    <div class="analysis-section">
      <h3>Execution & Declenchement</h3>
      <div class="detail-row">
        <span class="label">Trades exécutés</span>
        <span class="value">{{ advanced.execution.executedCount }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Durée moyenne</span>
        <span class="value">{{ advanced.execution.avgDuration }} min</span>
      </div>
      <div class="detail-row">
        <span class="label">Durée médiane</span>
        <span class="value">{{ advanced.execution.medianDuration }} min</span>
      </div>
      <div class="detail-row">
        <span class="label">Wins rapides (≤ 1 min)</span>
        <span class="value" :class="rateColor(n(advanced.execution.quickWinRate), 20, 40)">{{ advanced.execution.quickWinRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Loss rapides (≤ 1 min)</span>
        <span class="value" :class="rateColorInv(n(advanced.execution.quickLossRate), 10, 25)">{{ advanced.execution.quickLossRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">TakeProfit</span>
        <span class="value" :class="rateColor(n(advanced.execution.tpRate), 20, 40)">{{ advanced.execution.tpRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Timeout</span>
        <span class="value" :class="rateColorInv(n(advanced.execution.timeoutRate), 15, 30)">{{ advanced.execution.timeoutRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>MFE / MAE Avancé</h3>
      <div class="detail-row">
        <span class="label">MFE moyen</span>
        <span class="value val-good">{{ advanced.mfeMae.avgMfe }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">MAE moyen</span>
        <span class="value val-bad">{{ advanced.mfeMae.avgMae }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Ratio MFE/MAE</span>
        <span class="value" :class="rateColor(n(advanced.mfeMae.mfeMaeRatio), 1.0, 1.5)">{{ advanced.mfeMae.mfeMaeRatio }}</span>
      </div>
      <div class="detail-row">
        <span class="label">TP cible</span>
        <span class="value">{{ advanced.mfeMae.tpPips.toFixed(1) }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">TP potentiel atteint</span>
        <span class="value" :class="rateColor(n(advanced.mfeMae.tpPotentialRate), 30, 50)">{{ advanced.mfeMae.tpPotentialRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">TP manqué</span>
        <span class="value" :class="rateColorInv(n(advanced.mfeMae.tpMissRate), 30, 50)">{{ advanced.mfeMae.tpMissRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>BE & Trailing</h3>
      <div class="detail-row">
        <span class="label">BE atteint</span>
        <span class="value" :class="rateColor(n(advanced.trailing.beHitRate), 30, 50)">{{ advanced.trailing.beHitRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Sorties Trailing</span>
        <span class="value" :class="rateColor(n(advanced.trailing.trailingExitRate), 15, 30)">{{ advanced.trailing.trailingExitRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>Stabilité temporelle</h3>
      <div class="detail-row">
        <span class="label">Meilleur mois</span>
        <span class="value val-good">{{ advanced.stability.bestMonth }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Pire mois</span>
        <span class="value val-bad">{{ advanced.stability.worstMonth }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Mois profitables</span>
        <span class="value" :class="rateColor(monthRatio, 40, 60)">{{ advanced.stability.profitableMonths }}/{{ advanced.stability.totalMonths }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Meilleur jour</span>
        <span class="value">{{ advanced.stability.bestWeekday }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Pire jour</span>
        <span class="value">{{ advanced.stability.worstWeekday }}</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>Frais & Sensibilité</h3>
      <div class="detail-row">
        <span class="label">Coût estimé / trade</span>
        <span class="value">{{ advanced.fees.costPerTrade }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Coût estimé total</span>
        <span class="value">{{ advanced.fees.costTotal }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Poids des frais</span>
        <span class="value" :class="rateColorInv(n(advanced.fees.costRatio), 10, 25)">{{ advanced.fees.costRatio }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  advanced: {
    execution: {
      avgDuration: string
      medianDuration: string
      quickWinRate: string
      quickLossRate: string
      tpRate: string
      timeoutRate: string
      executedCount: number
    }
    mfeMae: {
      avgMfe: string
      avgMae: string
      mfeMaeRatio: string
      tpPotentialRate: string
      tpMissRate: string
      tpPips: number
    }
    trailing: {
      beHitRate: string
      trailingExitRate: string
    }
    stability: {
      bestMonth: string
      worstMonth: string
      profitableMonths: number
      totalMonths: number
      bestWeekday: string
      worstWeekday: string
    }
    fees: {
      costPerTrade: string
      costTotal: string
      costRatio: string
    }
  }
}>()

/** Parse string to number */
function n(val: string | number): number {
  return typeof val === 'number' ? val : parseFloat(val) || 0
}

/** Higher = better. Green if >= good, orange if >= warn, red below. */
function rateColor(val: number, warn: number, good: number): string {
  if (val >= good) return 'val-good'
  if (val >= warn) return 'val-warn'
  return 'val-bad'
}

/** Higher = worse (inverted). Green if < warn, orange if < bad, red above. */
function rateColorInv(val: number, warn: number, bad: number): string {
  if (val < warn) return 'val-good'
  if (val < bad) return 'val-warn'
  return 'val-bad'
}

const monthRatio = computed(() => {
  const total = props.advanced.stability.totalMonths
  if (total === 0) return 0
  return (props.advanced.stability.profitableMonths / total) * 100
})
</script>

<style scoped>
.analysis-details {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.analysis-section {
  background: #2d3748;
  padding: 16px;
  border-radius: 8px;
}

.analysis-section h3 {
  margin: 0 0 12px 0;
  font-size: 0.95rem;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 6px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.detail-row:last-child {
  border-bottom: none;
}

.label {
  color: #cbd5e0;
  font-size: 0.9rem;
}

.value {
  color: #e2e8f0;
  font-weight: 600;
  font-size: 0.9rem;
  text-align: right;
}

/* Semantic color classes */
.val-good { color: #48bb78; }
.val-warn { color: #ed8936; }
.val-bad { color: #f56565; }
</style>
