<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content analysis-modal">
      <div class="modal-header">
        <h2>🧠 Analyse du Backtest</h2>
        <button class="close-btn" @click="close">✕</button>
      </div>
      
      <div class="modal-body">
        <!-- 1. Verdict Rentabilité -->
        <div class="analysis-section verdict" :class="verdictClass">
          <div class="section-icon">{{ verdictIcon }}</div>
          <div class="section-content">
            <h3>Verdict : {{ verdictTitle }}</h3>
            <p>{{ verdictText }}</p>
          </div>
        </div>

        <!-- 2. Activité & Déclenchement -->
        <div class="analysis-section">
          <h3>📡 Activité & Déclenchement</h3>
          <p>
            Sur <strong>{{ totalEvents }}</strong> événements, 
            <strong :class="noEntryClass">{{ noEntryCount }}</strong> n'ont pas déclenché ({{ noEntryPercent }}%).
          </p>
          <p class="advice">{{ activityAdvice }}</p>
        </div>

        <!-- 3. Risque & Drawdown -->
        <div class="analysis-section">
          <h3>🛡️ Risque & Drawdown</h3>
          <p>
            Drawdown Max : <strong class="text-danger">{{ maxDrawdown }} pips</strong>.
            <span v-if="consecutiveLosses > 2">
              Attention, vous avez subi une série de <strong>{{ consecutiveLosses }} pertes consécutives</strong>.
            </span>
          </p>
          <p class="advice">{{ riskAdvice }}</p>
        </div>

        <!-- 4. Qualité des Sorties (MFE/MAE) -->
        <div class="analysis-section">
          <h3>🎯 Qualité des Sorties</h3>
          <div class="metrics-row">
            <div class="metric">
              <span class="label">MFE Moyen (Potentiel)</span>
              <span class="value text-success">{{ avgMfe }} pips</span>
            </div>
            <div class="metric">
              <span class="label">MAE Moyen (Risque)</span>
              <span class="value text-danger">{{ avgMae }} pips</span>
            </div>
          </div>
          <p class="advice">{{ exitAdvice }}</p>
        </div>

        <!-- 5. Analyse Avancee -->
        <div class="analysis-section">
          <h3>📌 Analyse Avancee</h3>
          <BacktestAnalysisDetails :advanced="advanced" />
        </div>

        <!-- 6. Recommandation Finale -->
        <div class="analysis-section recommendation">
          <h3>💡 Piste d'optimisation</h3>
          <p>{{ finalRecommendation }}</p>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-primary" @click="close">Fermer</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { BacktestResult, BacktestConfig } from '../stores/backtest'
import { useBacktestAnalysis } from '../composables/useBacktestAnalysis'
import BacktestAnalysisDetails from './backtest/BacktestAnalysisDetails.vue'

const props = defineProps<{
  isOpen: boolean
  result: BacktestResult
  config: BacktestConfig
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const close = () => emit('close')

const {
  totalEvents,
  noEntryCount,
  noEntryPercent,
  maxDrawdown,
  consecutiveLosses,
  avgMfe,
  avgMae,
  verdictClass,
  verdictIcon,
  verdictTitle,
  verdictText,
  noEntryClass,
  activityAdvice,
  riskAdvice,
  exitAdvice,
  finalRecommendation,
  advanced
} = useBacktestAnalysis(props.result, props.config)

</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content.analysis-modal {
  background: #1a202c;
  width: 600px;
  max-width: 90vw;
  border-radius: 12px;
  border: 1px solid #2d3748;
  display: flex;
  flex-direction: column;
  max-height: 90vh;
}

.modal-header {
  padding: 16px 24px;
  border-bottom: 1px solid #2d3748;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h2 { margin: 0; font-size: 1.2rem; color: #e2e8f0; }
.close-btn { background: none; border: none; color: #a0aec0; font-size: 1.5rem; cursor: pointer; }

.modal-body {
  padding: 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.analysis-section {
  background: #2d3748;
  padding: 16px;
  border-radius: 8px;
}

.analysis-section h3 {
  margin: 0 0 10px 0;
  font-size: 1rem;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.analysis-section p {
  margin: 0;
  color: #e2e8f0;
  line-height: 1.5;
}

.analysis-section .advice {
  margin-top: 8px;
  font-style: italic;
  color: #cbd5e0;
  font-size: 0.9rem;
  border-left: 3px solid #4299e1;
  padding-left: 10px;
}

/* Verdict Styles */
.verdict {
  display: flex;
  align-items: center;
  gap: 16px;
  border: 1px solid transparent;
}
.verdict-good { background: rgba(16, 185, 129, 0.1); border-color: #10b981; }
.verdict-neutral { background: rgba(245, 158, 11, 0.1); border-color: #f59e0b; }
.verdict-bad { background: rgba(239, 68, 68, 0.1); border-color: #ef4444; }

.section-icon { font-size: 2rem; }
.section-content h3 { color: inherit; margin-bottom: 4px; }

/* Metrics Row */
.metrics-row {
  display: flex;
  gap: 20px;
  margin-bottom: 8px;
}
.metric {
  display: flex;
  flex-direction: column;
}
.metric .label { font-size: 0.8rem; color: #a0aec0; }
.metric .value { font-size: 1.1rem; font-weight: bold; }

.text-success { color: #48bb78; }
.text-danger { color: #f56565; }
.text-warning { color: #ed8936; }

.recommendation {
  background: linear-gradient(135deg, rgba(66, 153, 225, 0.1), rgba(102, 126, 234, 0.1));
  border: 1px solid #4299e1;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid #2d3748;
  display: flex;
  justify-content: flex-end;
}

.btn-primary {
  background: #4299e1;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
}
.btn-primary:hover { background: #3182ce; }
</style>
