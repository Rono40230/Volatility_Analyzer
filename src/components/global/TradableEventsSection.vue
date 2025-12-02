<template>
  <div
    v-if="result.tradable_events && result.tradable_events.length > 0"
    class="events-section"
  >
    <h3>🎯 Types d'Événements les Plus Tradables (Stratégie Straddle)</h3>
    <div class="events-grid">
      <div
        v-for="(event, index) in result.tradable_events.slice(0, 5)"
        :key="event.event_name"
        class="event-card glass"
        :class="'event-rank-' + (index + 1)"
      >
        <div class="event-header">
          <div class="event-rank">
            #{{ index + 1 }}
          </div>
          <div class="event-name">
            {{ event.event_name }}
          </div>
        </div>
        <div class="event-metrics">
          <div class="event-metric">
            <span class="metric-label">Score Tradabilité</span>
            <span
              class="metric-value score"
              :class="getScoreClass(event.tradability_score)"
            >
              {{ event.tradability_score.toFixed(0) }}/100
            </span>
          </div>
          <div class="event-metric">
            <span class="metric-label">Augmentation Volatilité</span>
            <span class="metric-value">×{{ event.avg_volatility_increase.toFixed(2) }}</span>
          </div>
          <div class="event-metric">
            <span class="metric-label">Occurrences</span>
            <span class="metric-value">{{ event.occurrence_count }}</span>
          </div>
        </div>
        <div class="event-pairs">
          <span class="pairs-label">Paires affectées:</span>
          <span class="pairs-list">{{ event.affected_pairs.join(', ') }}</span>
        </div>
      </div>
    </div>
    <div class="insight-box glass">
      <h4>💡 Interprétation</h4>
      <p>
        Le <strong>Score de Tradabilité</strong> mesure l'augmentation de volatilité générée par l'événement. 
        Un score de 100 signifie que la volatilité double pendant l'événement (idéal pour straddle). 
        Privilégiez les événements avec un score > 50 et plusieurs occurrences pour valider la fiabilité.
      </p>
    </div>
  </div>
  <div
    v-else
    class="events-placeholder glass"
  >
    <div class="placeholder-icon">
      🎯
    </div>
    <h4>Types d'Événements Tradables</h4>
    <p>Cette analyse nécessite des archives de type "Corrélation événement/paire".</p>
    <p class="hint">
      Créez des analyses de corrélation pour débloquer cette fonctionnalité.
    </p>
  </div>
</template>

<script setup lang="ts">
interface TradableEvent {
  event_name: string
  tradability_score: number
  avg_volatility_increase: number
  occurrence_count: number
  affected_pairs: string[]
}

interface TradableEventsResult {
  tradable_events?: TradableEvent[]
}

defineProps<{
  result: TradableEventsResult
}>()

function getScoreClass(score: number): string {
  if (score >= 75) return 'score-excellent'
  if (score >= 50) return 'score-good'
  if (score >= 25) return 'score-average'
  return 'score-poor'
}
</script>

<style scoped>
.events-section { margin-bottom: 40px; }
.events-section h3 { font-size: 18px; margin-bottom: 20px; color: #e2e8f0; }
.events-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; margin-bottom: 20px; }
.event-card { padding: 20px; border-radius: 12px; border: 1px solid rgba(255, 255, 255, 0.05); transition: all 0.2s; }
.event-card:hover { transform: translateY(-2px); border-color: rgba(78, 205, 196, 0.3); }
.event-header { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
.event-rank { font-size: 14px; font-weight: 700; color: #718096; background: rgba(255, 255, 255, 0.05); padding: 2px 8px; border-radius: 4px; }
.event-rank-1 .event-rank { color: #fbbf24; background: rgba(251, 191, 36, 0.1); }
.event-name { font-weight: 600; color: #fff; font-size: 15px; }
.event-metrics { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; margin-bottom: 16px; padding-bottom: 16px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); }
.event-metric { display: flex; flex-direction: column; gap: 4px; }
.metric-label { font-size: 10px; color: #a0aec0; text-transform: uppercase; }
.metric-value { font-size: 14px; font-weight: 700; color: #fff; }
.metric-value.score { font-size: 16px; }
.score-excellent { color: #4ecdc4; } .score-good { color: #3b82f6; } .score-average { color: #f59e0b; } .score-poor { color: #ef4444; }
.event-pairs { font-size: 12px; color: #a0aec0; }
.pairs-label { margin-right: 6px; } .pairs-list { color: #e2e8f0; }
.insight-box { padding: 16px; border-radius: 8px; border-left: 4px solid #fbbf24; background: rgba(251, 191, 36, 0.1); }
.insight-box h4 { color: #fbbf24; margin: 0 0 8px 0; font-size: 14px; }
.insight-box p { margin: 0; font-size: 13px; color: #e2e8f0; line-height: 1.5; }
.events-placeholder { padding: 40px; text-align: center; border-radius: 12px; border: 1px dashed rgba(255, 255, 255, 0.1); margin-bottom: 40px; }
.placeholder-icon { font-size: 40px; margin-bottom: 16px; opacity: 0.5; }
.events-placeholder h4 { color: #e2e8f0; margin-bottom: 8px; }
.events-placeholder p { color: #a0aec0; font-size: 13px; margin: 0; }
.events-placeholder .hint { color: #4ecdc4; margin-top: 8px; font-size: 12px; }
.glass { background: rgba(30, 30, 45, 0.6); backdrop-filter: blur(10px); }
</style>
