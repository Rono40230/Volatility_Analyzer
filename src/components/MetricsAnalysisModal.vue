<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <!-- En-tête -->
      <div class="modal-header">
        <div class="header-title">
          <span class="icon">🎯</span>
          <h2>ANALYSE COMPLÈTE - STRADDLE SCALPING</h2>
        </div>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <!-- Diagnostic Global -->
      <div class="modal-section">
        <div class="section-header">
          <span class="icon">📊</span>
          <h3>DIAGNOSTIC GLOBAL</h3>
        </div>
        <div class="diagnostic-grid">
          <div class="diagnostic-item">
            <span class="label">Status</span>
            <span :class="['status-badge', getStatusClass(analysisData?.globalMetrics)]">
              {{ getStatusText(analysisData?.globalMetrics) }}
            </span>
          </div>
          <div class="diagnostic-item">
            <span class="label">Confiance</span>
            <span class="value">{{ analysisData?.confidence ?? 'N/A' }}/100</span>
          </div>
          <div class="diagnostic-item">
            <span class="label">Stratégie</span>
            <span class="value">{{ analysisData?.strategy ?? 'N/A' }}</span>
          </div>
          <div class="diagnostic-item">
            <span class="label">Meilleures heures</span>
            <span class="value">{{ analysisData?.bestHours ?? 'N/A' }}</span>
          </div>
        </div>
      </div>

      <!-- TOP 3 Tranches 15min -->
      <div class="modal-section">
        <div class="section-header">
          <span class="icon">⭐</span>
          <h3>TOP 3 HEURES D'ANALYSE DÉTAILLÉE</h3>
        </div>

        <div v-if="sliceAnalyses && sliceAnalyses.length > 0" class="slices-container">
          <!-- Pour chaque TOP 3 -->
          <div v-for="analysis in sliceAnalyses" :key="`slice-${analysis.rank}`" class="slice-card" :class="getRankClass(analysis.rank)">
            <!-- Rang + Heure -->
            <div class="slice-header">
              <div class="rank-badge" :class="`rank-${analysis.rank}`">
                <span class="rank-number">{{ analysis.rank }}</span>
                <span v-if="analysis.rank === 1" class="rank-medal">🏆</span>
                <span v-else-if="analysis.rank === 2" class="rank-medal">🥈</span>
                <span v-else class="rank-medal">🥉</span>
              </div>
              <div class="slice-time">
                <div class="time">{{ analysis.slice.startTime }}</div>
                <div class="score" :class="`score-${getScoreSeverity(analysis.slice.straddleScore)}`">
                  Score: {{ analysis.slice.straddleScore.toFixed(0) }}/100
                </div>
              </div>
            </div>

            <!-- Métriques Détaillées -->
            <div class="metrics-section">
              <h4>📈 Métriques (15min | Moyenne globale | Seuil Straddle)</h4>
              <div class="metrics-grid">
                <!-- ATR -->
                <div class="metric-item">
                  <span class="metric-name">ATR Moyen</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.atr_mean, 0.001, 0.002)">
                      {{ formatNumber(analysis.slice.stats.atr_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_atr ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.001</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.atr_mean, 0.001)]">
                    {{ getMetricStatusText(analysis.slice.stats.atr_mean, 0.001) }}
                  </span>
                </div>

                <!-- Range -->
                <div class="metric-item">
                  <span class="metric-name">Range (H-L)</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.range_mean, 0.0015, 0.0025)">
                      {{ formatNumber(analysis.slice.stats.range_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_range ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.0025</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.range_mean, 0.0025)]">
                    {{ getMetricStatusText(analysis.slice.stats.range_mean, 0.0025) }}
                  </span>
                </div>

                <!-- Volatility -->
                <div class="metric-item">
                  <span class="metric-name">Volatilité %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.volatility_mean * 100, 15, 30)">
                      {{ formatNumber(analysis.slice.stats.volatility_mean * 100, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber((analysisData?.globalMetrics.mean_volatility ?? 0) * 100, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>15%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.volatility_mean * 100, 15)]">
                    {{ getMetricStatusText(analysis.slice.stats.volatility_mean * 100, 15) }}
                  </span>
                </div>

                <!-- BodyRange -->
                <div class="metric-item">
                  <span class="metric-name">Body Range %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.body_range_mean, 25, 45)">
                      {{ formatNumber(analysis.slice.stats.body_range_mean, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_body_range ?? 0, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>45%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.body_range_mean, 45)]">
                    {{ getMetricStatusText(analysis.slice.stats.body_range_mean, 45) }}
                  </span>
                </div>

                <!-- TickQuality -->
                <div class="metric-item">
                  <span class="metric-name">Tick Quality</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.tick_quality_mean, 0.0005, 0.001)">
                      {{ formatNumber(analysis.slice.stats.tick_quality_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_tick_quality ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.001</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.tick_quality_mean, 0.001)]">
                    {{ getMetricStatusText(analysis.slice.stats.tick_quality_mean, 0.001) }}
                  </span>
                </div>

                <!-- NoiseRatio -->
                <div class="metric-item">
                  <span class="metric-name">Noise Ratio</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(2.5 - analysis.slice.stats.noise_ratio_mean, 0, 1.5)">
                      {{ formatNumber(analysis.slice.stats.noise_ratio_mean, 2) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_noise_ratio ?? 0, 2) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold"><2.0</span>
                  </div>
                  <span :class="['status', getNoiseStatus(analysis.slice.stats.noise_ratio_mean)]">
                    {{ getNoiseStatusText(analysis.slice.stats.noise_ratio_mean) }}
                  </span>
                </div>

                <!-- Imbalance -->
                <div class="metric-item">
                  <span class="metric-name">Volume Imbalance</span>
                  <div class="metric-values">
                    <span class="value15" :class="getImbalanceClass(analysis.slice.stats.volume_imbalance_mean)">
                      {{ formatNumber(analysis.slice.stats.volume_imbalance_mean, 3) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_volume_imbalance ?? 0, 3) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">0.5-2.0</span>
                  </div>
                  <span :class="['status', getImbalanceStatus(analysis.slice.stats.volume_imbalance_mean)]">
                    {{ getImbalanceStatusText(analysis.slice.stats.volume_imbalance_mean) }}
                  </span>
                </div>

                <!-- Breakout % -->
                <div class="metric-item">
                  <span class="metric-name">Breakout %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.breakout_percentage, 10, 20)">
                      {{ formatNumber(analysis.slice.stats.breakout_percentage, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_breakout_percentage ?? 0, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>15%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.breakout_percentage, 15)]">
                    {{ getMetricStatusText(analysis.slice.stats.breakout_percentage, 15) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Golden Combos -->
            <div v-if="analysis.goldenCombos.length > 0" class="combos-section">
              <h4>🔥 Golden Combos Détectés</h4>
              <div class="combos-list">
                <div v-for="(combo, idx) in analysis.goldenCombos" :key="`combo-${idx}`" class="combo-item" :class="`confidence-${combo.confidence.toLowerCase()}`">
                  <div class="combo-header">
                    <span class="combo-name">{{ combo.name }}</span>
                    <span class="combo-confidence" :class="`badge-${combo.confidence.toLowerCase()}`">{{ combo.confidence }}</span>
                  </div>
                  <p class="combo-description">{{ combo.description }}</p>
                  <div class="combo-metrics">
                    <span class="metric">Win Rate: {{ (combo.winRate * 100).toFixed(0) }}%</span>
                    <span class="metric">Avg Gain: {{ combo.avgGainR.toFixed(1) }}R</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Pièges -->
            <div v-if="analysis.traps.length > 0" class="traps-section">
              <h4>⚠️ Pièges à Éviter</h4>
              <div class="traps-list">
                <div v-for="(trap, idx) in analysis.traps" :key="`trap-${idx}`" class="trap-item" :class="`severity-${trap.severity.toLowerCase()}`">
                  <div class="trap-header">
                    <span class="trap-name">{{ trap.name }}</span>
                    <span class="trap-severity" :class="`badge-${trap.severity.toLowerCase()}`">{{ trap.severity }}</span>
                  </div>
                  <p class="trap-description">{{ trap.description }}</p>
                  <div class="trap-detail">
                    <span><strong>{{ trap.metric }}:</strong> {{ trap.value.toFixed(2) }} (seuil: {{ trap.threshold.toFixed(2) }})</span>
                  </div>
                  <div class="trap-recommendation">
                    <strong>💡 Conseil:</strong> {{ trap.recommendation }}
                  </div>
                </div>
              </div>
            </div>

            <!-- Plan d'Action -->
            <div class="plan-section">
              <h4>💡 PLAN D'ACTION PERSONNALISÉ</h4>
              <div class="plan-grid">
                <div class="plan-item">
                  <div class="label">ENTRY TIME</div>
                  <div class="value">{{ analysis.tradingPlan.entryTime }}</div>
                </div>
                <div class="plan-item">
                  <div class="label">STOP LOSS</div>
                  <div class="value detailed">
                    <span>{{ analysis.tradingPlan.slPips }} pips</span>
                    <span class="secondary">{{ analysis.tradingPlan.slPoints }} points</span>
                    <span class="secondary">${{ analysis.tradingPlan.slUsd }}</span>
                  </div>
                </div>
                <div class="plan-item">
                  <div class="label">TAKE PROFIT</div>
                  <div class="value detailed">
                    <span>{{ analysis.tradingPlan.tpPips }} pips</span>
                    <span class="secondary">{{ analysis.tradingPlan.tpPoints }} points</span>
                    <span class="secondary">${{ analysis.tradingPlan.tpUsd }}</span>
                  </div>
                </div>
                <div class="plan-item">
                  <div class="label">POSITION SIZE</div>
                  <div class="value">{{ analysis.tradingPlan.positionSize }}% du risque</div>
                </div>
                <div class="plan-item">
                  <div class="label">RISK/REWARD</div>
                  <div class="value">{{ analysis.tradingPlan.riskReward }}</div>
                </div>
                <div class="plan-item">
                  <div class="label">WIN PROBABILITY</div>
                  <div class="value">{{ analysis.tradingPlan.winProbability }}%</div>
                </div>
                <div class="plan-item">
                  <div class="label">AVG GAIN</div>
                  <div class="value">{{ analysis.tradingPlan.avgGainR.toFixed(1) }}R</div>
                </div>
                <div class="plan-item">
                  <div class="label">MAX DURATION</div>
                  <div class="value">{{ analysis.tradingPlan.maxDuration }} min</div>
                </div>
                <div class="plan-item">
                  <div class="label">TRAILING STOP</div>
                  <div class="value">Activation: {{ analysis.tradingPlan.trailingStopActivation }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="no-data">
          <p>Aucune donnée disponible pour l'analyse</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="btn-primary" @click="close">Fermer l'analyse</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import { analyzeTop3Slices } from '../utils/straddleAnalysis'

interface Props {
  isOpen: boolean
  analysisResult: AnalysisResult | null
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const analysisData = ref<any>(null)
const sliceAnalyses = ref<SliceAnalysis[] | null>(null)

// Watcher pour analyser quand analysisResult change
watch(
  () => props.analysisResult,
  (result) => {
    if (result && props.isOpen) {
      analysisData.value = {
        globalMetrics: result.global_metrics,
        confidence: Math.round(result.confidence_score),
        strategy: 'SCALPING STANDARD',
        bestHours: result.best_hours.slice(0, 3).join(', ')
      }

      // Analyser les TOP 3 tranches 15min
      if (result.stats_15min && result.stats_15min.length > 0) {
        sliceAnalyses.value = analyzeTop3Slices(result.stats_15min)
      }
    }
  }
)

const close = () => {
  emit('close')
}

// Fonctions utilitaires de rendu
const formatNumber = (value: number, decimals: number): string => {
  return value.toFixed(decimals)
}

const getStatusClass = (metrics: any): string => {
  if (!metrics) return 'unknown'
  const confidence = metrics.mean_atr // Utiliser ATR comme proxy
  if (confidence > 0.0015) return 'excellent'
  if (confidence > 0.001) return 'good'
  if (confidence > 0.0005) return 'acceptable'
  return 'poor'
}

const getStatusText = (metrics: any): string => {
  const classes = ['Excellent', 'Good', 'Acceptable', 'Poor']
  return classes[['excellent', 'good', 'acceptable', 'poor'].indexOf(getStatusClass(metrics))]
}

const getMetricClass = (value: number, threshold1: number, threshold2: number): string => {
  if (value >= threshold2) return 'excellent'
  if (value >= threshold1) return 'good'
  return 'poor'
}

const getMetricStatus = (value: number, threshold: number): string => {
  return value >= threshold ? 'ok' : 'low'
}

const getMetricStatusText = (value: number, threshold: number): string => {
  return value >= threshold ? '✅ OK' : '❌ TOO LOW'
}

const getNoiseStatus = (value: number): string => {
  if (value < 2.0) return 'excellent'
  if (value < 3.0) return 'good'
  return 'poor'
}

const getNoiseStatusText = (value: number): string => {
  if (value < 2.0) return '✅ Signal pur'
  if (value < 3.0) return '🟡 Acceptable'
  return '❌ Chaotique'
}

const getImbalanceClass = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return 'excellent'
  if (distance > 0.5) return 'good'
  return 'poor'
}

const getImbalanceStatus = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return 'ok'
  return 'low'
}

const getImbalanceStatusText = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return `✅ Tendance marquée`
  if (distance > 0.5) return `🟡 Modérée`
  return `❌ Équilibrée`
}

const getScoreSeverity = (score: number): string => {
  if (score >= 75) return 'excellent'
  if (score >= 50) return 'good'
  if (score >= 25) return 'acceptable'
  return 'poor'
}

const getRankClass = (rank: number): string => {
  return `rank-${rank}`
}
</script>

<style scoped lang="css">
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: #1a1f2e;
  border: 2px solid #2d3748;
  border-radius: 12px;
  width: 100%;
  max-width: 1400px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
  color: #e2e8f0;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border-bottom: 2px solid #2d3748;
  background: linear-gradient(135deg, #1a1f2e 0%, #2d3748 100%);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title .icon {
  font-size: 24px;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #cbd5e0;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.modal-section {
  padding: 24px;
  border-bottom: 1px solid #2d3748;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.section-header .icon {
  font-size: 20px;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Diagnostic Grid */
.diagnostic-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.diagnostic-item {
  background: rgba(45, 55, 72, 0.5);
  border: 1px solid #2d3748;
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diagnostic-item .label {
  font-size: 12px;
  color: #a0aec0;
  text-transform: uppercase;
  font-weight: 600;
}

.diagnostic-item .value,
.diagnostic-item .status-badge {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
}

.status-badge {
  padding: 4px 8px;
  border-radius: 4px;
  width: fit-content;
}

.status-badge.excellent {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.status-badge.good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}

.status-badge.acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.status-badge.poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Slices Container */
.slices-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.slice-card {
  background: rgba(45, 55, 72, 0.4);
  border: 2px solid #2d3748;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s;
}

.slice-card:hover {
  background: rgba(45, 55, 72, 0.6);
  border-color: #4a5568;
}

.slice-card.rank-1 {
  border-color: #fbbf24;
  background: rgba(251, 191, 36, 0.05);
}

.slice-card.rank-2 {
  border-color: #a78bfa;
  background: rgba(167, 139, 250, 0.05);
}

.slice-card.rank-3 {
  border-color: #f97316;
  background: rgba(249, 115, 22, 0.05);
}

/* Slice Header */
.slice-header {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid #2d3748;
}

.rank-badge {
  width: 80px;
  height: 80px;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 24px;
  gap: 4px;
}

.rank-badge.rank-1 {
  background: rgba(251, 191, 36, 0.15);
  border: 2px solid #fbbf24;
  color: #fbbf24;
}

.rank-badge.rank-2 {
  background: rgba(167, 139, 250, 0.15);
  border: 2px solid #a78bfa;
  color: #a78bfa;
}

.rank-badge.rank-3 {
  background: rgba(249, 115, 22, 0.15);
  border: 2px solid #f97316;
  color: #f97316;
}

.rank-number {
  font-size: 28px;
}

.rank-medal {
  font-size: 20px;
}

.slice-time {
  flex: 1;
}

.slice-time .time {
  font-size: 18px;
  font-weight: 700;
  color: #fff;
}

.slice-time .score {
  font-size: 14px;
  margin-top: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  width: fit-content;
}

.score-excellent {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.score-good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}

.score-acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.score-poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Metrics Section */
.metrics-section {
  margin-bottom: 20px;
}

.metrics-section h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 12px;
}

.metric-item {
  background: rgba(45, 55, 72, 0.3);
  border: 1px solid #2d3748;
  border-radius: 8px;
  padding: 12px;
}

.metric-name {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: #a0aec0;
  text-transform: uppercase;
  margin-bottom: 6px;
}

.metric-values {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 13px;
}

.metric-values .value15 {
  font-weight: 700;
  min-width: 80px;
  font-family: monospace;
}

.metric-values .valueglobal {
  color: #a0aec0;
  font-family: monospace;
}

.metric-values .separator {
  color: #4a5568;
}

.metric-values .threshold {
  color: #f59e0b;
  font-weight: 600;
}

.metric-values .value15.excellent {
  color: #86efac;
}

.metric-values .value15.good {
  color: #93c5fd;
}

.metric-values .value15.poor {
  color: #fca5a5;
}

.status {
  display: inline-block;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
}

.status.ok {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.status.low {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Golden Combos */
.combos-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: 8px;
}

.combos-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #86efac;
}

.combos-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.combo-item {
  background: rgba(45, 55, 72, 0.4);
  border-left: 4px solid #22c55e;
  border-radius: 6px;
  padding: 12px;
}

.combo-item.confidence-jackpot {
  border-left-color: #fbbf24;
  background: rgba(251, 191, 36, 0.08);
}

.combo-item.confidence-excellent {
  border-left-color: #a78bfa;
  background: rgba(167, 139, 250, 0.08);
}

.combo-item.confidence-bon {
  border-left-color: #3b82f6;
  background: rgba(59, 130, 246, 0.08);
}

.combo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.combo-name {
  font-weight: 700;
  color: #fff;
}

.combo-confidence {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.combo-confidence.badge-jackpot {
  background: #fbbf24;
  color: #000;
}

.combo-confidence.badge-excellent {
  background: #a78bfa;
  color: #fff;
}

.combo-confidence.badge-bon {
  background: #3b82f6;
  color: #fff;
}

.combo-description {
  margin: 6px 0;
  font-size: 13px;
  color: #cbd5e0;
  line-height: 1.4;
}

.combo-metrics {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: #a0aec0;
}

.combo-metrics .metric {
  display: flex;
  gap: 4px;
}

/* Pièges */
.traps-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 8px;
}

.traps-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #fca5a5;
}

.traps-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.trap-item {
  background: rgba(45, 55, 72, 0.4);
  border-left: 4px solid #ef4444;
  border-radius: 6px;
  padding: 12px;
}

.trap-item.severity-critique {
  border-left-color: #dc2626;
  background: rgba(220, 38, 38, 0.08);
}

.trap-item.severity-haute {
  border-left-color: #ef4444;
  background: rgba(239, 68, 68, 0.08);
}

.trap-item.severity-moyenne {
  border-left-color: #f97316;
  background: rgba(249, 115, 22, 0.08);
}

.trap-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.trap-name {
  font-weight: 700;
  color: #fff;
}

.trap-severity {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.trap-severity.badge-critique {
  background: #dc2626;
  color: #fff;
}

.trap-severity.badge-haute {
  background: #ef4444;
  color: #fff;
}

.trap-severity.badge-moyenne {
  background: #f97316;
  color: #fff;
}

.trap-description {
  margin: 6px 0;
  font-size: 13px;
  color: #cbd5e0;
  line-height: 1.4;
}

.trap-detail {
  margin: 8px 0;
  font-size: 12px;
  color: #a0aec0;
  background: rgba(0, 0, 0, 0.2);
  padding: 6px 8px;
  border-radius: 4px;
  font-family: monospace;
}

.trap-recommendation {
  margin-top: 8px;
  font-size: 12px;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.1);
  padding: 6px 8px;
  border-radius: 4px;
}

/* Plan Section */
.plan-section {
  padding: 16px;
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 8px;
}

.plan-section h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #93c5fd;
}

.plan-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.plan-item {
  background: rgba(45, 55, 72, 0.4);
  border: 1px solid #2d3748;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.plan-item .label {
  font-size: 11px;
  font-weight: 700;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.plan-item .value {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  font-family: monospace;
}

.plan-item .value.detailed {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
}

.plan-item .value.detailed .secondary {
  font-size: 11px;
  color: #a0aec0;
}

/* Modal Footer */
.modal-footer {
  padding: 20px 24px;
  border-top: 1px solid #2d3748;
  background: rgba(45, 55, 72, 0.3);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-primary {
  padding: 10px 20px;
  background: #3b82f6;
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover {
  background: #2563eb;
}

/* No Data */
.no-data {
  text-align: center;
  padding: 40px;
  color: #a0aec0;
}

/* Scrollbar */
.modal-content::-webkit-scrollbar {
  width: 8px;
}

.modal-content::-webkit-scrollbar-track {
  background: rgba(45, 55, 72, 0.3);
}

.modal-content::-webkit-scrollbar-thumb {
  background: #4a5568;
  border-radius: 4px;
}

.modal-content::-webkit-scrollbar-thumb:hover {
  background: #718096;
}
</style>
