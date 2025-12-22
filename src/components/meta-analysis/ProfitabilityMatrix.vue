<template>
  <div class="tab-content">
    <h3>Matrice de Rentabilité (Score Volatilité/Bruit)</h3>
    <div class="heatmap-container">
      <table>
        <thead>
          <tr>
            <th>Événement \ Paire</th>
            <th v-for="pair in data.pairs" :key="pair">{{ pair }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="event in data.events" :key="event">
            <td class="row-header">{{ event }}</td>
            <td 
              v-for="pair in data.pairs" 
              :key="pair"
              :style="{ backgroundColor: getHeatmapColor(data.matrix[event][pair]) }"
              class="cell"
            >
              {{ data.matrix[event][pair] > 0 ? data.matrix[event][pair].toFixed(1) : '-' }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
interface MatrixData {
  pairs: string[]
  events: string[]
  matrix: Record<string, Record<string, number>>
}

defineProps<{
  data: MatrixData
}>()

function getHeatmapColor(score: number): string {
  if (score === 0) return '#333'
  // Echelle simple : 0 -> 50 (vert)
  const intensity = Math.min(score / 50, 1)
  return `rgba(76, 175, 80, ${intensity})`
}
</script>

<style scoped>
.tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  height: 100%;
}

.heatmap-container {
  overflow-x: auto;
  background: #1a1a1a;
  border-radius: 8px;
  padding: 10px;
}

.heatmap-container table {
  width: 100%;
  border-collapse: collapse;
}

.heatmap-container th, .heatmap-container td {
  padding: 10px;
  text-align: center;
  border: 1px solid #333;
}

.heatmap-container th {
  background: #252525;
  color: #aaa;
}

.heatmap-container .row-header {
  background: #252525;
  color: #e0e0e0;
  font-weight: 600;
  text-align: left;
}

.heatmap-container .cell {
  color: #fff;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
}
</style>
