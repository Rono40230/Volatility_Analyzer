<template>
  <div class="results-container">
    <div class="graph-section full-width graph-large">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
        <h3>📊 Cycle complet de volatilité (Peak + Décroissance){{ eventLabel ? ' - ' + eventLabel : '' }}</h3>
        <button v-if="!isArchiveMode" class="btn-archive" @click="$emit('archive')">💾 Archiver</button>
      </div>
      <div class="graph-container graph-container-large">
        <svg viewBox="0 0 900 400" class="graph graph-enlarged">
          <defs>
            <linearGradient id="peakGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" style="stop-color:#58a6ff;stop-opacity:0.3" />
              <stop offset="100%" style="stop-color:#58a6ff;stop-opacity:0" />
            </linearGradient>
            <linearGradient id="decayGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" style="stop-color:#f85149;stop-opacity:0.2" />
              <stop offset="100%" style="stop-color:#f85149;stop-opacity:0" />
            </linearGradient>
            <linearGradient id="stableGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" style="stop-color:#3fb950;stop-opacity:0.15" />
              <stop offset="100%" style="stop-color:#3fb950;stop-opacity:0" />
            </linearGradient>
          </defs>
          <line x1="80" y1="320" x2="850" y2="320" stroke="#4a5568" stroke-width="2" />
          <line x1="80" y1="40" x2="80" y2="320" stroke="#4a5568" stroke-width="2" />
          <line x1="80" y1="180" x2="850" y2="180" stroke="#2d3748" stroke-width="1" stroke-dasharray="5,5" />
          <line x1="80" y1="250" x2="850" y2="250" stroke="#2d3748" stroke-width="1" stroke-dasharray="5,5" />
          <text x="65" y="325" font-size="12" text-anchor="end" fill="#8b949e">0%</text>
          <text x="65" y="185" font-size="12" text-anchor="end" fill="#8b949e">50%</text>
          <text x="65" y="45" font-size="12" text-anchor="end" fill="#8b949e">100%</text>
          <path d="M 110 280 Q 250 140 400 50 L 400 320 L 110 320 Z" fill="url(#peakGradient)" />
          <path d="M 400 50 Q 550 170 700 260 L 700 320 L 400 320 Z" fill="url(#decayGradient)" />
          <path d="M 700 260 L 820 270 L 820 320 L 700 320 Z" fill="url(#stableGradient)" />
          <path d="M 110 280 Q 250 140 400 50" stroke="#58a6ff" stroke-width="4" fill="none" stroke-linecap="round" />
          <path d="M 400 50 Q 550 170 700 260" stroke="#f85149" stroke-width="4" fill="none" stroke-linecap="round" />
          <line x1="700" y1="260" x2="820" y2="270" stroke="#3fb950" stroke-width="3" stroke-linecap="round" />
          <circle cx="110" cy="280" r="6" fill="#58a6ff" stroke="#fff" stroke-width="2" />
          <circle cx="400" cy="50" r="6" fill="#f85149" stroke="#fff" stroke-width="2" />
          <circle cx="700" cy="260" r="6" fill="#3fb950" stroke="#fff" stroke-width="2" />
          <line x1="400" y1="50" x2="400" y2="340" stroke="#f85149" stroke-width="1" stroke-dasharray="4,4" opacity="0.5" />
          <line x1="700" y1="260" x2="700" y2="340" stroke="#3fb950" stroke-width="1" stroke-dasharray="4,4" opacity="0.5" />
          <text x="110" y="355" font-size="13" text-anchor="middle" fill="#8b949e">T0</text>
          <text x="400" y="355" font-size="13" text-anchor="middle" fill="#f85149" font-weight="bold">T+{{ peakDelay }} min</text>
          <text x="700" y="355" font-size="13" text-anchor="middle" fill="#3fb950" font-weight="bold">T+{{ totalDuration }} min</text>
          <text x="200" y="30" font-size="14" fill="#58a6ff" font-weight="bold">📈 Phase Peak</text>
          <text x="500" y="30" font-size="14" fill="#f85149" font-weight="bold">📉 Phase Décroissance</text>
          <text x="740" y="290" font-size="12" fill="#3fb950">✓ Stabilisé</text>
          <text x="110" y="390" font-size="11" fill="#58a6ff">ATR max: {{ peakAtr.toFixed(4) }}</text>
          <text x="400" y="390" font-size="11" fill="#f85149">Taux: {{ decayRate !== undefined ? formatPointsWithPips('EURUSD', decayRate) : 'N/A' }}/min</text>
          <text x="700" y="390" font-size="11" fill="#3fb950">Demi-vie: ~9 min (50%)</text>
        </svg>
      </div>
    </div>

    <div class="analysis-grid-2-cols">
      <div class="interpretation-block">
        <p><strong>💡 Interprétation</strong></p>
        <ul>
          <li>Pic atteint à <strong>T+{{ peakDelay }} min</strong></li>
          <li>Volatilité revient à la normale après <strong>{{ totalDuration }} min</strong></li>
          <li>Confiance: <strong>{{ confidence }}%</strong> (basé sur {{ eventCount }} événements)</li>
        </ul>
      </div>

      <div class="interpretation-block">
        <p><strong>⚡ Recommandations Straddle</strong></p>
        <ul>
          <li>Taux de décroissance: <strong>{{ decayRate !== undefined ? formatPointsWithPips('EURUSD', decayRate) : 'N/A' }}/min</strong> ({{ decaySpeed }})</li>
          <li>Demi-vie: <strong>~9 min</strong> (50% de vol restant)</li>
          <li>TP/SL optimal: À <strong>T+{{ decayTimeout }} min</strong> (vol réduite à ~5%)</li>
          <li>Sortie max: <strong>T+{{ maxExit }} min</strong> (avant stabilisation)</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { formatPointsWithPips } from '../utils/pipConverter'

interface Props {
  peakDelay: number
  decayTimeout: number
  peakAtr: number
  decayRate: number
  decaySpeed: string
  confidence: number
  eventCount: number
  isArchiveMode?: boolean
  eventLabel?: string
}

const props = defineProps<Props>()

defineEmits<{ archive: [] }>()

const totalDuration = computed(() => props.peakDelay + props.decayTimeout)
const maxExit = computed(() => Math.ceil(props.decayTimeout * 1.5))
</script>

<style scoped>
.results-container { display: flex; flex-direction: column; gap: 20px; width: 100%; height: 100%; overflow-y: auto; flex: 1; }
.graph-section { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; flex-shrink: 0; height: auto; min-height: 450px; }
.graph-section h3 { margin: 0 0 20px 0; color: #58a6ff; font-size: 1.1em; }
.graph-container { width: 100%; background: #0d1117; border-radius: 6px; padding: 10px; height: 400px; }
.graph { width: 100%; max-width: 100%; aspect-ratio: auto; }
.analysis-grid-2-cols { display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px; width: 100%; flex-shrink: 0; height: auto; min-height: auto; }
.interpretation-block { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; border-left: 3px solid #58a6ff; }
.interpretation-block p { margin: 0 0 15px 0; color: #58a6ff; font-weight: 600; font-size: 1em; }
.interpretation-block ul { margin: 0; padding-left: 20px; list-style: none; }
.interpretation-block li { margin: 10px 0; color: #c9d1d9; line-height: 1.5; font-size: 0.95em; }

.btn-archive { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; padding: 10px 20px; border-radius: 6px; font-weight: 600; cursor: pointer; transition: all 0.2s; font-size: 0.95em; }
.btn-archive:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }
.btn-archive:active { transform: translateY(0); }
@media (max-width: 768px) { .analysis-grid-2-cols { grid-template-columns: 1fr; } }
</style>
