<template>
  <div class="panel-container directional-panel">
    <div class="panel-header">
      <h4>🔵 Directionnel</h4>
    </div>

    <div class="bidi-grid">
      <div class="bidi-param">
        <div class="bidi-label">Moment de placement</div>
        <div class="bidi-value">{{ placementTime || (meilleurMoment !== undefined ? (meilleurMoment === 0 ? 'T0' : Math.round(meilleurMoment)) : '—') }} <span class="bidi-unit">{{ placementTime ? '' : (meilleurMoment === 0 ? '(Début)' : 'min avant') }}</span></div>
        <div class="bidi-description">Quand placer les ordres en attente</div>
      </div>
      
      <div class="bidi-param buy-param">
        <div class="bidi-label">BUY STOP</div>
        <div class="bidi-value">
          +<UnitDisplay v-if="offset > 0" :value="offset" :unit="getUnit(pair)" :decimals="0" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Entrée Achat (au-dessus du prix)</div>
      </div>

      <div class="bidi-param sell-param">
        <div class="bidi-label">SELL STOP</div>
        <div class="bidi-value">
          -<UnitDisplay v-if="offset > 0" :value="offset" :unit="getUnit(pair)" :decimals="0" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Entrée Vente (en-dessous du prix)</div>
      </div>

      <div class="bidi-param">
        <div class="bidi-label">Stop Loss</div>
        <div class="bidi-value">
          <UnitDisplay v-if="stopLoss > 0" :value="stopLoss" unit="pts" :decimals="1" :symbol="pair" />
          <span v-else>—</span>
        </div>
        <div class="bidi-description">Distance d'arrêt (Risque)</div>
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

      <div class="bidi-param graph-param">
        <div class="bidi-label">Visualisation</div>
        <div class="visualizer-wrapper">
          <svg viewBox="0 75 200 150" class="visualizer-svg">
            <!-- Center Line (Price) -->
            <line x1="20" y1="150" x2="180" y2="150" stroke="#4a5568" stroke-width="1" stroke-dasharray="4,4" />
            <text x="185" y="153" font-size="10" fill="#8b949e">T0</text>

            <!-- Spread Zone -->
            <rect v-if="spread > 0" x="20" :y="150 - scaleY(spread/2)" width="160" :height="scaleY(spread)" fill="#4a5568" opacity="0.3" />
            <text v-if="spread > 0" x="15" y="153" font-size="9" fill="#6b7280" text-anchor="end">Spread</text>

            <!-- Buy Stop -->
            <g v-if="offset > 0">
              <line x1="40" :y1="150 - scaleY(offset)" x2="160" :y2="150 - scaleY(offset)" stroke="#4ade80" stroke-width="2" />
              <text x="165" :y="150 - scaleY(offset) + 3" font-size="10" fill="#4ade80">BUY</text>
              
              <!-- SL Buy -->
              <line x1="60" :y1="150 - scaleY(offset) + scaleY(stopLoss)" x2="140" :y2="150 - scaleY(offset) + scaleY(stopLoss)" stroke="#f87171" stroke-width="1" stroke-dasharray="3,3" />
              <text x="30" :y="150 - scaleY(offset) + scaleY(stopLoss) + 3" font-size="9" fill="#f87171" text-anchor="end">SL</text>

              <!-- Hard TP Buy -->
              <line v-if="hardTp > 0" x1="60" :y1="150 - scaleY(offset) - scaleY(hardTp)" x2="140" :y2="150 - scaleY(offset) - scaleY(hardTp)" stroke="#60a5fa" stroke-width="1" stroke-dasharray="3,3" />
              <text v-if="hardTp > 0" x="30" :y="150 - scaleY(offset) - scaleY(hardTp) + 3" font-size="9" fill="#60a5fa" text-anchor="end">TP</text>
            </g>

            <!-- Sell Stop -->
            <g v-if="offset > 0">
              <line x1="40" :y1="150 + scaleY(offset)" x2="160" :y2="150 + scaleY(offset)" stroke="#f87171" stroke-width="2" />
              <text x="165" :y="150 + scaleY(offset) + 3" font-size="10" fill="#f87171">SELL</text>

              <!-- SL Sell -->
              <line x1="60" :y1="150 + scaleY(offset) - scaleY(stopLoss)" x2="140" :y2="150 + scaleY(offset) - scaleY(stopLoss)" stroke="#f87171" stroke-width="1" stroke-dasharray="3,3" />
              <text x="30" :y="150 + scaleY(offset) - scaleY(stopLoss) + 3" font-size="9" fill="#f87171" text-anchor="end">SL</text>

              <!-- Hard TP Sell -->
              <line v-if="hardTp > 0" x1="60" :y1="150 + scaleY(offset) + scaleY(hardTp)" x2="140" :y2="150 + scaleY(offset) + scaleY(hardTp)" stroke="#60a5fa" stroke-width="1" stroke-dasharray="3,3" />
              <text v-if="hardTp > 0" x="30" :y="150 + scaleY(offset) + scaleY(hardTp) + 3" font-size="9" fill="#60a5fa" text-anchor="end">TP</text>
            </g>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import UnitDisplay from '../UnitDisplay.vue'
import { obtenirPointsParPip } from '../../utils/pipConverter'

interface Props {
  meilleurMoment?: number
  offset?: number
  stopLoss?: number
  hardTp?: number
  trailingStop?: number
  timeout?: number
  pair: string
  pointValue?: number
  placementTime?: string
  spread?: number
}

const props = withDefaults(defineProps<Props>(), {
  meilleurMoment: 0,
  offset: 0,
  stopLoss: 0,
  hardTp: 0,
  trailingStop: 0,
  timeout: 60,
  pair: 'EURUSD',
  spread: 0
})

function getUnit(pair: string): string {
  return obtenirPointsParPip(pair) === 1 ? 'pts' : 'pips'
}

// Simple scaling function for visualization
const scaleY = (val: number) => {
  const maxVal = Math.max(props.stopLoss || 0, props.hardTp || 0)
  const maxRange = (maxVal || 100) * 1.2 + (props.offset || 50)
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

.directional-panel {
  border-left: 4px solid #3b82f6;
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

.buy-param {
  border-left: 3px solid #4ade80;
}

.sell-param {
  border-left: 3px solid #f87171;
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

.graph-param {
  grid-row: span 2;
  display: flex;
  flex-direction: column;
}

.visualizer-wrapper {
  flex: 1;
  min-height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.visualizer-svg {
  width: 100%;
  height: 100%;
  max-height: 200px;
}
</style>
