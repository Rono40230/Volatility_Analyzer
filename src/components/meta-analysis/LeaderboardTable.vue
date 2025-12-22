<template>
  <div class="tab-content">
    <h3>Leaderboard des Événements</h3>
    <table class="data-table">
      <thead>
        <tr>
          <th>Rang</th>
          <th>Événement</th>
          <th>Analyses</th>
          <th>Volatilité Moy.</th>
          <th>Bruit Moy.</th>
          <th>Score</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(item, index) in data" :key="item.event">
          <td>#{{ index + 1 }}</td>
          <td>{{ item.event }}</td>
          <td>{{ item.count }}</td>
          <td class="positive">+{{ item.avgVol.toFixed(1) }}%</td>
          <td>{{ item.avgNoise.toFixed(2) }}</td>
          <td><strong>{{ item.score.toFixed(1) }}</strong></td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
interface LeaderboardItem {
  event: string
  count: number
  avgVol: number
  avgNoise: number
  score: number
}

defineProps<{
  data: LeaderboardItem[]
}>()
</script>

<style scoped>
.tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  height: 100%;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  background: #1a1a1a;
  border-radius: 8px;
  overflow: hidden;
}

.data-table th, .data-table td {
  padding: 12px 15px;
  text-align: left;
  border-bottom: 1px solid #333;
}

.data-table th {
  background: #252525;
  color: #aaa;
  font-weight: 600;
}

.data-table tr:hover {
  background: #2a2a2a;
}

.positive {
  color: #4caf50;
}
</style>
