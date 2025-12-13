<template>
  <div class="volatility-duration-section">
    <div class="section-header">
      <span class="icon">⏱️</span>
      <h5>Note Stratégique & Durée</h5>
    </div>
    
    <div class="duration-grid">
      <div class="duration-item">
        <span class="label">Pic de Volatilité</span>
        <span class="value">{{ formatNumber(volatilityDuration.peak_duration_minutes) }} min</span>
        <span class="desc">Durée de l'impulsion principale (>80% max)</span>
      </div>
      
      <div class="duration-item">
        <span class="label">Demi-Vie (Half-Life)</span>
        <span class="value">{{ formatNumber(volatilityDuration.half_life_minutes) }} min</span>
        <span class="desc">Temps pour revenir à 50% de volatilité</span>
      </div>
      
      <div class="duration-item highlight">
        <span class="label">Expiration Trade</span>
        <span class="value">{{ formatNumber(volatilityDuration.trade_expiration_minutes) }} min</span>
        <span class="desc">Fermeture recommandée (Max Profit)</span>
      </div>
    </div>

    <div v-if="tradingPlan && tradingPlan.risk_assessment" class="risk-note">
      <strong>Note de Risque:</strong> {{ tradingPlan.risk_assessment }}
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  volatilityDuration: Record<string, unknown>
  tradingPlan: Record<string, unknown>
}>()

function formatNumber(val: unknown): string {
  if (typeof val === 'number') return val.toFixed(0)
  return '—'
}
</script>

<style scoped>
.volatility-duration-section {
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 12px;
  margin-top: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  border-bottom: 1px solid #334155;
  padding-bottom: 8px;
}

.section-header h5 {
  margin: 0;
  font-size: 0.95rem;
  color: #94a3b8;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.duration-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.duration-item {
  display: flex;
  flex-direction: column;
  background: rgba(15, 23, 42, 0.4);
  padding: 8px;
  border-radius: 6px;
  border: 1px solid #1e293b;
}

.duration-item.highlight {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.3);
}

.label {
  font-size: 0.75rem;
  color: #94a3b8;
  margin-bottom: 4px;
}

.value {
  font-size: 1.1rem;
  font-weight: 700;
  color: #f1f5f9;
}

.duration-item.highlight .value {
  color: #60a5fa;
}

.desc {
  font-size: 0.65rem;
  color: #64748b;
  margin-top: 4px;
  line-height: 1.2;
}

.risk-note {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed #334155;
  font-size: 0.8rem;
  color: #cbd5e1;
  font-style: italic;
}
</style>
