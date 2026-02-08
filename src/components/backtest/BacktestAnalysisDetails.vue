<template>
  <div class="analysis-details">
    <div class="analysis-section">
      <h3>Execution & Declenchement</h3>
      <div class="detail-row">
        <span class="label">Trades executes</span>
        <span class="value">{{ advanced.execution.executedCount }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Duree moyenne</span>
        <span class="value">{{ advanced.execution.avgDuration }} min</span>
      </div>
      <div class="detail-row">
        <span class="label">Duree mediane</span>
        <span class="value">{{ advanced.execution.medianDuration }} min</span>
      </div>
      <div class="detail-row">
        <span class="label">Wins rapides (<= 1 min)</span>
        <span class="value">{{ advanced.execution.quickWinRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Loss rapides (<= 1 min)</span>
        <span class="value">{{ advanced.execution.quickLossRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">TakeProfit</span>
        <span class="value">{{ advanced.execution.tpRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Timeout</span>
        <span class="value">{{ advanced.execution.timeoutRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>MFE / MAE Avance</h3>
      <div class="detail-row">
        <span class="label">MFE moyen</span>
        <span class="value">{{ advanced.mfeMae.avgMfe }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">MAE moyen</span>
        <span class="value">{{ advanced.mfeMae.avgMae }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Ratio MFE/MAE</span>
        <span class="value">{{ advanced.mfeMae.mfeMaeRatio }}</span>
      </div>
      <div class="detail-row">
        <span class="label">TP cible</span>
        <span class="value">{{ advanced.mfeMae.tpPips.toFixed(1) }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">TP potentiel atteint</span>
        <span class="value">{{ advanced.mfeMae.tpPotentialRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">TP manque</span>
        <span class="value">{{ advanced.mfeMae.tpMissRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>BE & Trailing</h3>
      <div class="detail-row">
        <span class="label">BE atteint</span>
        <span class="value">{{ advanced.trailing.beHitRate }}%</span>
      </div>
      <div class="detail-row">
        <span class="label">Sorties Trailing</span>
        <span class="value">{{ advanced.trailing.trailingExitRate }}%</span>
      </div>
    </div>

    <div class="analysis-section">
      <h3>Stabilite temporelle</h3>
      <div class="detail-row">
        <span class="label">Meilleur mois</span>
        <span class="value">{{ advanced.stability.bestMonth }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Pire mois</span>
        <span class="value">{{ advanced.stability.worstMonth }}</span>
      </div>
      <div class="detail-row">
        <span class="label">Mois profitables</span>
        <span class="value">{{ advanced.stability.profitableMonths }}/{{ advanced.stability.totalMonths }}</span>
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
      <h3>Frais & Sensibilite</h3>
      <div class="detail-row">
        <span class="label">Cout estime / trade</span>
        <span class="value">{{ advanced.fees.costPerTrade }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Cout estime total</span>
        <span class="value">{{ advanced.fees.costTotal }} pips</span>
      </div>
      <div class="detail-row">
        <span class="label">Poids des frais</span>
        <span class="value">{{ advanced.fees.costRatio }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
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
</style>
