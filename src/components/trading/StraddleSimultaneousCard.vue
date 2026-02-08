<template>
  <div class="panel-container simultaneous-panel">
    <div class="panel-header">
      <h4>🟣 Parametres BIDI</h4>
    </div>

    <div class="bidi-grid">
      <div class="bidi-param">
        <div class="bidi-label">Moment de placement</div>
        <div class="bidi-value">{{ placementTime || (meilleurMoment !== undefined ? (meilleurMoment === 0 ? 'T0' : Math.round(meilleurMoment)) : '—') }} <span class="bidi-unit">{{ placementTime ? '' : (meilleurMoment === 0 ? '(Début)' : 'min avant') }}</span></div>
        <div class="bidi-description">Moment optimal basé sur la volatilité</div>
      </div>
      
      <div class="bidi-param recovery-param">
        <div class="bidi-label">SL RECOVERY</div>
        <div class="bidi-value">
          <UnitDisplay v-if="stopLossRecovery > 0" :value="stopLossRecovery" unit="pts" :decimals="1" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Pour couvrir le retournement</div>
      </div>

      <div class="bidi-param">
        <div class="bidi-label">Hard TP</div>
        <div class="bidi-value">
          <UnitDisplay v-if="hardTp > 0" :value="hardTp" unit="pts" :decimals="1" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Take Profit de sécurité (SL x 2)</div>
      </div>

      <div class="bidi-param">
        <div class="bidi-label">Trailing Stop</div>
        <div class="bidi-value">
          <UnitDisplay v-if="trailingStop > 0" :value="trailingStop" unit="pts" :decimals="1" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Stop dynamique adapté au noise</div>
      </div>

      <div class="bidi-param">
        <div class="bidi-label">Timeout</div>
        <div class="bidi-value">{{ timeout || '60' }} <span class="bidi-unit">min</span></div>
        <div class="bidi-description">Durée maximale du trade</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import UnitDisplay from '../UnitDisplay.vue'

interface Props {
  meilleurMoment?: number
  offset?: number
  stopLossRecovery?: number
  hardTp?: number
  trailingStop?: number
  timeout?: number
  pair: string
  pointValue?: number
  placementTime?: string
}

const props = withDefaults(defineProps<Props>(), {
  meilleurMoment: 0,
  offset: 0,
  stopLossRecovery: 0,
  hardTp: 0,
  trailingStop: 0,
  timeout: 60,
  pair: 'EURUSD'
})

// Simple scaling function for visualization
const scaleY = (val: number) => {
  const maxVal = Math.max(props.stopLossRecovery || 0, props.hardTp || 0)
  const maxRange = (maxVal || 150) * 1.2
  const pxRange = 60 // adjusted for viewBox height 150 (half 75)
  return (val / maxRange) * pxRange
}
</script>

<style scoped>
.panel-container {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 8px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.simultaneous-panel {
  border-left: 4px solid #a855f7;
}

.panel-header {
  margin-bottom: 8px;
  border-bottom: 1px solid #30363d;
  padding-bottom: 4px;
}

.panel-header h4 {
  margin: 0;
  color: #e6edf3;
  font-size: 1.1em;
  display: flex;
  align-items: center;
  gap: 8px;
}

.bidi-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 6px;
  flex: 1;
}

.bidi-param {
  background: #0d1117;
  padding: 6px;
  border-radius: 6px;
  border: 1px solid #21262d;
}

.recovery-param {
  border-left: 3px solid #facc15;
}

.bidi-label {
  font-size: 0.7em;
  text-transform: uppercase;
  color: #8b949e;
  letter-spacing: 0.3px;
  margin-bottom: 4px;
}

.bidi-value {
  font-size: 1em;
  font-weight: 600;
  color: #e6edf3;
  font-family: 'Roboto Mono', monospace;
  line-height: 1.2;
}

.bidi-unit {
  font-size: 0.7em;
  color: #6e8a99;
  font-weight: 400;
  margin-left: 2px;
}

.bidi-description {
  font-size: 0.65em;
  color: #6e8a99;
  margin-top: 2px;
  font-style: italic;
}

</style>
