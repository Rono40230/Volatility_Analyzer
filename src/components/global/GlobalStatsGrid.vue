<template>
  <div class="global-stats-grid">
    <MetricTooltip title="Analyses Scannées">
      <div class="stat-card glass">
        <div class="stat-icon">
          📊
        </div>
        <div class="stat-info">
          <span class="stat-label">Analyses Scannées</span>
          <span class="stat-value">{{ result.total_analyses }}</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section-title">
          📖 Définition
        </div>
        <div class="tooltip-section-text">
          Nombre total d'archives de type "Volatilité brute" trouvées et analysées dans votre base de données.
        </div>
      </template>
      <template #usage>
        <div class="tooltip-section-title">
          💡 Interprétation
        </div>
        <div class="tooltip-section-text">
          Plus ce nombre est élevé, plus les statistiques de l'IA sont fiables. Avec moins de 5 analyses, les résultats peuvent être biaisés.
        </div>
      </template>
    </MetricTooltip>

    <MetricTooltip title="Confiance Moyenne">
      <div class="stat-card glass">
        <div class="stat-icon">
          🎯
        </div>
        <div class="stat-info">
          <span class="stat-label">Confiance Moyenne</span>
          <span class="stat-value highlight">{{ result.global_stats.average_confidence.toFixed(1) }}/100</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section-title">
          📖 Définition
        </div>
        <div class="tooltip-section-text">
          Moyenne des scores de confiance de toutes vos analyses archivées. Ce score combine volatilité, qualité des ticks et fiabilité des signaux.
        </div>
      </template>
      <template #scoring>
        <div class="tooltip-section-title">
          📊 Échelle de Confiance
        </div>
        <div class="tooltip-section-text">
          • <strong>80-100</strong> : Scalp Agressif ✅<br>
          • <strong>65-79</strong> : Scalp Normal 🟢<br>
          • <strong>50-64</strong> : Scalp Prudent 🟡<br>
          • <strong>35-49</strong> : Très Prudent 🟠<br>
          • <strong>0-34</strong> : Ne pas trader ❌
        </div>
      </template>
    </MetricTooltip>

    <MetricTooltip title="Volatilité Moyenne">
      <div class="stat-card glass">
        <div class="stat-icon">
          📈
        </div>
        <div class="stat-info">
          <span class="stat-label">Volatilité Moyenne</span>
          <span class="stat-value">{{ (result.global_stats.average_volatility * 100).toFixed(2) }}%</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section-title">
          📖 Définition
        </div>
        <div class="tooltip-section-text">
          Mesure l'amplitude moyenne des mouvements de prix sur toutes vos paires analysées. Calculée via l'ATR (Average True Range) normalisé.
        </div>
      </template>
      <template #usage>
        <div class="tooltip-section-title">
          💡 Interprétation
        </div>
        <div class="tooltip-section-text">
          • <strong>\u003c 10%</strong> : Marché calme, peu d'opportunités<br>
          • <strong>10-25%</strong> : Volatilité idéale pour le scalping<br>
          • <strong>\u003e 25%</strong> : Marché chaotique, risque élevé
        </div>
      </template>
    </MetricTooltip>
  </div>
</template>

<script setup lang="ts">
import MetricTooltip from '../MetricTooltip.vue'

interface GlobalStats {
  average_confidence: number
  average_volatility: number
}

interface GlobalStatsResult {
  total_analyses: number
  global_stats: GlobalStats
}

defineProps<{
  result: GlobalStatsResult
}>()
</script>

<style scoped>
.global-stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  padding: 20px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 16px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: transform 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  border-color: rgba(78, 205, 196, 0.3);
}

.stat-icon {
  font-size: 32px;
  background: rgba(255, 255, 255, 0.05);
  width: 60px;
  height: 60px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-label {
  color: #a0aec0;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #fff;
}

.stat-value.highlight {
  color: #4ecdc4;
}

.tooltip-section-title {
  font-size: 12px;
  font-weight: 700;
  color: #4ecdc4;
  margin-bottom: 4px;
  text-transform: uppercase;
}

.tooltip-section-text {
  font-size: 12px;
  color: #e2e8f0;
  line-height: 1.5;
  margin-bottom: 12px;
}

.tooltip-section-text:last-child {
  margin-bottom: 0;
}

.glass {
  background: rgba(30, 30, 45, 0.6);
  backdrop-filter: blur(10px);
}
</style>
