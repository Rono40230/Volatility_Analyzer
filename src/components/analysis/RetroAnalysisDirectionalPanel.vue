<template>
  <div class="panel-container directional-panel">
    <div class="panel-header">
      <h4>🔵 Directionnel</h4>
    </div>

    <div class="bidi-grid">
      <div class="bidi-param">
        <div class="bidi-label">Moment de placement</div>
        <div class="bidi-value">{{ meilleurMoment > 0 ? Math.round(meilleurMoment) : '—' }} <span class="bidi-unit">min avant</span></div>
        <div class="bidi-description">Quand placer les ordres en attente</div>
      </div>
      
      <div class="bidi-param buy-param">
        <div class="bidi-label">BUY STOP</div>
        <div class="bidi-value">+{{ offset > 0 ? formatPointsWithPips(pair, offset, 0) : '—' }}</div>
        <div class="bidi-description">Entrée Achat (au-dessus du prix)</div>
      </div>

      <div class="bidi-param sell-param">
        <div class="bidi-label">SELL STOP</div>
        <div class="bidi-value">-{{ offset > 0 ? formatPointsWithPips(pair, offset, 0) : '—' }}</div>
        <div class="bidi-description">Entrée Vente (en-dessous du prix)</div>
      </div>

      <div class="bidi-param">
        <div class="bidi-label">Stop Loss</div>
        <div class="bidi-value">{{ stopLoss > 0 ? formatPointsWithPips(pair, stopLoss, 0) : '—' }}</div>
        <div class="bidi-description">Distance d'arrêt (Risque)</div>
      </div>
      <div class="bidi-param">
        <div class="bidi-label">Trailing Stop</div>
        <div class="bidi-value">{{ trailingStop > 0 ? formatPointsWithPips(pair, trailingStop, 1) : '—' }}</div>
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

            <!-- Buy Stop -->
            <g v-if="offset > 0">
              <line x1="40" :y1="150 - scaleY(offset)" x2="160" :y2="150 - scaleY(offset)" stroke="#4ade80" stroke-width="2" />
              <text x="165" :y="150 - scaleY(offset) + 3" font-size="10" fill="#4ade80">BUY</text>
              
              <!-- SL Buy -->
              <line x1="60" :y1="150 - scaleY(offset) + scaleY(stopLoss)" x2="140" :y2="150 - scaleY(offset) + scaleY(stopLoss)" stroke="#f87171" stroke-width="1" stroke-dasharray="3,3" />
              <text x="30" :y="150 - scaleY(offset) + scaleY(stopLoss) + 3" font-size="9" fill="#f87171" text-anchor="end">SL</text>
            </g>

            <!-- Sell Stop -->
            <g v-if="offset > 0">
              <line x1="40" :y1="150 + scaleY(offset)" x2="160" :y2="150 + scaleY(offset)" stroke="#f87171" stroke-width="2" />
              <text x="165" :y="150 + scaleY(offset) + 3" font-size="10" fill="#f87171">SELL</text>

              <!-- SL Sell -->
              <line x1="60" :y1="150 + scaleY(offset) - scaleY(stopLoss)" x2="140" :y2="150 + scaleY(offset) - scaleY(stopLoss)" stroke="#f87171" stroke-width="1" stroke-dasharray="3,3" />
              <text x="30" :y="150 + scaleY(offset) - scaleY(stopLoss) + 3" font-size="9" fill="#f87171" text-anchor="end">SL</text>
            </g>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { formatPointsWithPips } from '../../utils/pipConverter'

interface Props {
  meilleurMoment?: number
  offset?: number
  stopLoss?: number
  trailingStop?: number
  timeout?: number
  pair: string
}

const props = withDefaults(defineProps<Props>(), {
  meilleurMoment: 0,
  offset: 0,
  stopLoss: 0,
  trailingStop: 0,
  timeout: 60,
  pair: 'EURUSD'
})

// Simple scaling function for visualization
// Max range to display is roughly SL * 1.5
const scaleY = (val: number) => {
  const maxRange = (props.stopLoss || 100) * 1.5
  const pxRange = 120 // half height available
  return (val / maxRange) * pxRange
}
</script>

<style scoped>
.panel-container {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 15px;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

.directional-panel {
  border-top: 3px solid #58a6ff;
}

.panel-header h4 {
  margin: 0 0 15px 0;
  color: #58a6ff;
  font-size: 0.9em;
  text-transform: uppercase;
  letter-spacing: 1px;
  text-align: center;
}

.bidi-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 10px;
  margin-bottom: 20px;
}

.bidi-param {
  background: rgba(255, 255, 255, 0.03);
  padding: 8px;
  border: 1px solid #3a5a78;
  border-radius: 6px;
  transition: all 0.2s;
}

.buy-param {
  border-color: rgba(74, 222, 128, 0.3);
  background: rgba(74, 222, 128, 0.05);
}

.buy-param .bidi-label { color: #4ade80; }

.sell-param {
  border-color: rgba(248, 113, 113, 0.3);
  background: rgba(248, 113, 113, 0.05);
}

.sell-param .bidi-label { color: #f87171; }

.bidi-param:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateY(-2px);
}

.bidi-label {
  font-size: 0.7em;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  margin-bottom: 4px;
}

.bidi-value {
  font-size: 1em;
  font-weight: 600;
  color: #58a6ff;
  margin-bottom: 2px;
  line-height: 1.2;
}

.bidi-unit {
  font-size: 0.7em;
  color: #6e8a99;
  margin-left: 2px;
}

.bidi-description {
  font-size: 0.65em;
  color: #6e8a99;
  margin-top: 2px;
  font-style: italic;
}

.graph-param {
  display: flex;
  flex-direction: column;
}

.visualizer-wrapper {
  width: 100%;
  height: 250px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  margin-top: 5px;
  padding: 0;
  box-sizing: border-box;
}

.visualizer-svg {
  width: 100%;
  height: 100%;
}
</style>
