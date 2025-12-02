<template>
  <div v-if="result.optimal_time_windows?.length > 0" class="timing-section">
    <h3>⏱️ Fenêtres Temporelles Optimales Post-Événement</h3>
    <div class="timing-grid">
      <div v-for="(window, index) in result.optimal_time_windows.slice(0, 6)" :key="window.event_type" class="timing-card glass" :class="'timing-rank-' + (index + 1)">
        <div class="timing-header">
          <div class="timing-rank">#{{ index + 1 }}</div>
          <div class="timing-event">
            <div class="event-name-original">{{ window.event_type }}</div>
            <div v-if="translateEventName(window.event_type) !== window.event_type" class="event-name-translation">({{ translateEventName(window.event_type) }})</div>
          </div>
          <div class="timing-consistency" :class="getScoreClass(window.consistency_score)">{{ window.consistency_score.toFixed(0) }}%</div>
        </div>
        <div class="timing-metrics">
          <div class="timing-metric"><div class="metric-icon">🎯</div><div class="metric-content"><span class="metric-label">Peak Time</span><span class="metric-value">{{ window.avg_peak_time_minutes.toFixed(0) }} min</span><span class="metric-hint">Temps pour atteindre le pic</span></div></div>
          <div class="timing-metric"><div class="metric-icon">🚪</div><div class="metric-content"><span class="metric-label">Entry Window</span><span class="metric-value">{{ window.avg_entry_window_minutes.toFixed(0) }} min avant</span><span class="metric-hint">Fenêtre d'entrée optimale</span></div></div>
          <div class="timing-metric"><div class="metric-icon">⏳</div><div class="metric-content"><span class="metric-label">Return to Normal</span><span class="metric-value">{{ window.avg_return_to_normal_minutes.toFixed(0) }} min</span><span class="metric-hint">Temps de retour au calme</span></div></div>
        </div>
        <div class="timing-footer"><span class="footer-label">{{ window.occurrence_count }} occurrences</span><span class="footer-separator">•</span><span class="footer-label">{{ window.affected_pairs.length }} paires</span></div>
      </div>
    </div>
    <div class="insight-box glass">
      <h4>💡 Guide d'Utilisation</h4>
      <p><strong>Peak Time</strong> : Temps moyen pour atteindre le maximum de volatilité après l'événement. <strong>Entry Window</strong> : Placez votre straddle dans cette fenêtre avant l'événement pour maximiser vos chances. <strong>Return to Normal</strong> : Durée pendant laquelle le marché reste volatil. Fermez vos positions avant ce délai. Le <strong>Score de Consistance</strong> indique la fiabilité de ces timings (basé sur le nombre d'occurrences).</p>
    </div>
  </div>
  <div v-else class="timing-placeholder glass">
    <div class="placeholder-icon">⏱️</div>
    <h4>Fenêtres Temporelles Optimales</h4>
    <p>Cette analyse nécessite des archives de type "Corrélation paire/événement".</p>
    <p class="hint">Créez des analyses de corrélation pour débloquer cette fonctionnalité.</p>
  </div>
</template>

<script setup lang="ts">
import { useEventTranslation } from '../../composables/useEventTranslation'

interface OptimalTimeWindow {
  event_type: string
  consistency_score: number
  avg_peak_time_minutes: number
  avg_entry_window_minutes: number
  avg_return_to_normal_minutes: number
  occurrence_count: number
  affected_pairs: string[]
}

interface OptimalTimingResult {
  optimal_time_windows?: OptimalTimeWindow[]
}

defineProps<{
  result: OptimalTimingResult
}>()

const { translateEventName } = useEventTranslation()

function getScoreClass(score: number): string {
  if (score >= 75) return 'score-excellent'
  if (score >= 50) return 'score-good'
  if (score >= 25) return 'score-average'
  return 'score-poor'
}
</script>

<style scoped>
.timing-section { margin-bottom: 20px; }
.timing-section h3 { font-size: 18px; margin-bottom: 20px; color: #e2e8f0; }
.timing-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 20px; margin-bottom: 20px; }
.timing-card { padding: 20px; border-radius: 12px; border: 1px solid rgba(255, 255, 255, 0.05); transition: all 0.2s; }
.timing-card:hover { transform: translateY(-2px); border-color: rgba(78, 205, 196, 0.3); }
.timing-header { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 20px; padding-bottom: 16px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); }
.timing-rank { font-size: 14px; font-weight: 700; color: #718096; background: rgba(255, 255, 255, 0.05); padding: 2px 8px; border-radius: 4px; margin-top: 2px; }
.timing-rank-1 .timing-rank { color: #fbbf24; background: rgba(251, 191, 36, 0.1); }
.timing-event { flex: 1; }
.event-name-original { font-weight: 600; color: #fff; font-size: 15px; margin-bottom: 2px; }
.event-name-translation { font-size: 12px; color: #a0aec0; font-style: italic; }
.timing-consistency { font-size: 16px; font-weight: 700; }
.score-excellent { color: #4ecdc4; } .score-good { color: #3b82f6; } .score-average { color: #f59e0b; } .score-poor { color: #ef4444; }
.timing-metrics { display: flex; flex-direction: column; gap: 16px; margin-bottom: 16px; }
.timing-metric { display: flex; align-items: flex-start; gap: 12px; }
.metric-icon { font-size: 18px; background: rgba(255, 255, 255, 0.05); width: 32px; height: 32px; border-radius: 50%; display: flex; align-items: center; justify-content: center; }
.metric-content { display: flex; flex-direction: column; }
.metric-label { font-size: 11px; color: #a0aec0; text-transform: uppercase; margin-bottom: 2px; }
.metric-value { font-size: 15px; font-weight: 600; color: #fff; }
.metric-hint { font-size: 11px; color: #718096; }
.timing-footer { font-size: 12px; color: #a0aec0; display: flex; align-items: center; gap: 8px; padding-top: 12px; border-top: 1px solid rgba(255, 255, 255, 0.05); }
.footer-separator { color: #4a5568; }
.insight-box { padding: 16px; border-radius: 8px; border-left: 4px solid #fbbf24; background: rgba(251, 191, 36, 0.1); }
.insight-box h4 { color: #fbbf24; margin: 0 0 8px 0; font-size: 14px; }
.insight-box p { margin: 0; font-size: 13px; color: #e2e8f0; line-height: 1.5; }
.timing-placeholder { padding: 40px; text-align: center; border-radius: 12px; border: 1px dashed rgba(255, 255, 255, 0.1); margin-bottom: 20px; }
.placeholder-icon { font-size: 40px; margin-bottom: 16px; opacity: 0.5; }
.timing-placeholder h4 { color: #e2e8f0; margin-bottom: 8px; }
.timing-placeholder p { color: #a0aec0; font-size: 13px; margin: 0; }
.timing-placeholder .hint { color: #4ecdc4; margin-top: 8px; font-size: 12px; }
.glass { background: rgba(30, 30, 45, 0.6); backdrop-filter: blur(10px); }
</style>
