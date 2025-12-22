<template>
  <div class="tab-content">
    <h3>Graphique de Divergence (Volatilité vs Propreté)</h3>
    <div class="chart-container scatter-plot">
      <!-- Axe Y -->
      <div class="axis-y-label">Propreté (Faible Bruit) ⬆️</div>
      
      <div class="scatter-area">
        <div 
          v-for="point in data" 
          :key="point.id"
          class="scatter-point"
          :style="{ left: point.x + '%', bottom: point.y + '%' }"
          :title="`${point.eventType} (${point.pair})\nVol: +${point.volatilityIncrease.toFixed(1)}%\nNoise: ${point.noiseRatio.toFixed(2)}`"
        ></div>
      </div>

      <!-- Axe X -->
      <div class="axis-x-label">Explosivité (Volatilité) ➡️</div>
    </div>
    <div class="legend">
      <span class="legend-item"><span class="dot"></span> Chaque point est une analyse archivée</span>
      <span class="legend-item">Haut-Droit = Pépites 💎</span>
      <span class="legend-item">Bas-Droit = Danger ⚠️</span>
    </div>
  </div>
</template>

<script setup lang="ts">
interface DivergencePoint {
  id: number
  x: number
  y: number
  eventType: string
  pair: string
  volatilityIncrease: number
  noiseRatio: number
}

defineProps<{
  data: DivergencePoint[]
}>()
</script>

<style scoped>
.tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  height: 100%;
}

.chart-container {
  flex: 1;
  background: #1a1a1a;
  border-radius: 8px;
  position: relative;
  min-height: 400px;
  border: 1px solid #333;
}

.scatter-plot {
  padding: 40px 40px 60px 60px;
}

.scatter-area {
  width: 100%;
  height: 100%;
  position: relative;
  border-left: 2px solid #444;
  border-bottom: 2px solid #444;
}

.scatter-point {
  position: absolute;
  width: 12px;
  height: 12px;
  background: #2196f3;
  border-radius: 50%;
  transform: translate(-50%, 50%);
  cursor: pointer;
  transition: transform 0.2s, background 0.2s;
  box-shadow: 0 0 5px rgba(33, 150, 243, 0.5);
}

.scatter-point:hover {
  transform: translate(-50%, 50%) scale(1.5);
  background: #64b5f6;
  z-index: 10;
}

.axis-y-label {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%) rotate(-90deg);
  color: #888;
  font-size: 0.9rem;
}

.axis-x-label {
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  color: #888;
  font-size: 0.9rem;
}

.legend {
  display: flex;
  gap: 20px;
  justify-content: center;
  color: #888;
  font-size: 0.9rem;
}

.dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  background: #2196f3;
  border-radius: 50%;
  margin-right: 5px;
}
</style>
