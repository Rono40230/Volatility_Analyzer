<template>
  <div class="col-center">
    <div v-if="volatilityProfile && volatilityProfile.length > 0" class="graph-container">
      <QuarterlyProfileChart 
        :profile="volatilityProfile" 
        :optimal-entry="meilleurMoment"
        :duration="duration"
        :entry-label="placementTime ? `Entrée (${placementTime})` : undefined"
        :hour="hour"
        :quarter="quarter"
      />
    </div>
    <div v-else class="graph-placeholder">
      <div class="placeholder-content">
        <div class="icon">📊</div>
        <div class="message">Graphique de volatilité événementielle non disponible</div>
        <div class="sub-message">L'analyse brute est temporelle et non liée à un événement spécifique (T0).</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import QuarterlyProfileChart from '../charts/QuarterlyProfileChart.vue'

defineProps<{
  volatilityProfile: number[]
  meilleurMoment: number
  duration?: number
  placementTime?: string
  hour?: number
  quarter?: number
}>()
</script>

<style scoped>
.col-center {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.graph-container {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 4px;
  height: 100%;
  min-height: 150px;
  margin-bottom: 0;
}

.graph-placeholder {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  min-height: 200px;
  margin-bottom: 15px;
}

.placeholder-content {
  margin-bottom: 30px;
  opacity: 0.7;
}

.placeholder-content .icon {
  font-size: 40px;
  margin-bottom: 10px;
  opacity: 0.5;
}

.placeholder-content .message {
  font-size: 14px;
  font-weight: 600;
  color: #8b949e;
  margin-bottom: 5px;
}

.placeholder-content .sub-message {
  font-size: 12px;
  color: #6e7681;
}
</style>
