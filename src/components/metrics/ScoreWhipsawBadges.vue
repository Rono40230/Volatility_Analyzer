<template>
  <!-- Score & Whipsaw Blocs -->
  <div style="display: flex; gap: 12px; align-items: center;">
    <!-- Score Bloc -->
    <MetricTooltip
      title="Score"
      direction="bottom"
      style="max-width: 250px;"
    >
      <div style="flex: 0 0 auto; padding: 12px 16px; background: rgba(45, 90, 123, 0.15); border: 1px solid #2d5a7b; border-radius: 6px; font-size: 12px; min-width: 140px;">
        <div style="color: #64a5d8; margin-bottom: 6px; font-weight: bold;">
          ⭐ SCORE
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <div style="font-size: 18px; font-weight: bold;" :style="{ color: getScoreColor(score) }">
            {{ score.toFixed(0) }}/100
          </div>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Score de viabilité du Straddle ajusté par la fréquence whipsaw. Reflète la qualité réelle du setup en tenant compte des retournements.</div>
        </div>
      </template>
      <template #interpretation>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📊 Interprétation</div>
          <div class="interpretation-item"><strong>🟢 Excellent:</strong> ≥75/100 → Setup optimal</div>
          <div class="interpretation-item"><strong>🔵 Bon:</strong> 60-74/100 → Setup viable</div>
          <div class="interpretation-item"><strong>🟡 Acceptable:</strong> 45-59/100 → À considérer</div>
          <div class="interpretation-item"><strong>🔴 Faible:</strong> &lt;45/100 → À éviter</div>
        </div>
      </template>
    </MetricTooltip>

    <!-- Fréquence Whipsaw -->
    <MetricTooltip
      title="Fréquence Whipsaw"
      direction="bottom"
    >
      <div style="flex: 0 0 auto; padding: 12px 16px; background: rgba(45, 90, 123, 0.15); border: 1px solid #2d5a7b; border-radius: 6px; font-size: 12px; min-width: 160px;">
        <div style="color: #64a5d8; margin-bottom: 6px; font-weight: bold;">
          📊 WHIPSAW
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <div style="font-size: 18px; font-weight: bold;" :style="{ color: getWhipsawColor(whipsawFrequency) }">
            {{ whipsawFrequency.toFixed(1) }}%
          </div>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition</div>
          <div class="tooltip-section-text">Fréquence des retournements rapides (whipsaws) dans le quarter. Mesure la volatilité et l'instabilité du mouvement.</div>
        </div>
      </template>
      <template #interpretation>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📊 Interprétation</div>
          <div class="interpretation-item"><strong>🟢 Très Bas:</strong> &lt;5% → Excellent, très stable</div>
          <div class="interpretation-item"><strong>🔵 Bas:</strong> 5-10% → Bon, acceptable</div>
          <div class="interpretation-item"><strong>🟡 Modéré:</strong> 10-20% → Attention requise</div>
          <div class="interpretation-item"><strong>🟠 Élevé:</strong> 20-30% → Risqué</div>
          <div class="interpretation-item"><strong>🔴 Très Élevé:</strong> &gt;30% → À éviter</div>
        </div>
      </template>
    </MetricTooltip>
  </div>
</template>

<script setup lang="ts">
import MetricTooltip from '../MetricTooltip.vue'
import { getScoreColor, getWhipsawColor } from './BestSliceCard.helpers'

defineProps<{
  score: number
  whipsawFrequency: number
}>()
</script>

<style scoped>
.interpretation-item {
  padding: 4px 0;
  font-size: 12px;
}
</style>
