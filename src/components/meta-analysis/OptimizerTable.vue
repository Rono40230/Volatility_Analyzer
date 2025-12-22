<template>
  <div class="tab-content">
    <h3>Optimiseur de Paramètres (Moyennes Historiques)</h3>
    <table class="data-table">
      <thead>
        <tr>
          <th>Événement</th>
          <th>Offset Moy.</th>
          <th>SL Moy.</th>
          <th>Timeout Moy.</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in data" :key="item.event">
          <td>{{ item.event }}</td>
          <td>{{ item.avgOffset.toFixed(1) }} pts</td>
          <td>{{ item.avgSL.toFixed(1) }} pts</td>
          <td>{{ Math.round(item.avgTimeout) }} min</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
interface OptimizerItem {
  event: string
  avgOffset: number
  avgSL: number
  avgTimeout: number
}

defineProps<{
  data: OptimizerItem[]
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
</style>
