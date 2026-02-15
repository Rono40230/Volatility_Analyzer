<template>
  <div class="metrics-section">
    <h4>METRIQUES</h4>
    <table class="metrics-table">
      <thead>
        <tr>
          <th class="label-column"></th>
          <th v-for="metric in displayedMetrics" :key="metric.label" class="metric-header">
            <MetricTooltip :title="metric.label">
              <span class="metric-title">
                {{ metric.label }}
              </span>
              <template #definition>
                <div class="tooltip-section">
                  <div class="tooltip-section-title">📖 Definition de la Metrique</div>
                  <div class="tooltip-section-text">{{ metric.definition || 'Indicateur de volatilite et de qualite du mouvement.' }}</div>
                </div>
              </template>
              <template #usage>
                <div class="tooltip-section">
                  <div class="tooltip-section-title">📊 Interpretation du Score</div>
                  <div class="tooltip-section-text">{{ metric.usage || 'Donne la lecture pratique de la metrique.' }}</div>
                </div>
              </template>
              <template #scoring>
                <div class="tooltip-section">
                  <div class="tooltip-section-title">🎨 Echelle de Couleurs</div>
                  <div class="tooltip-section-text tooltip-multiline">{{ metric.scoring || 'Echelle non specifiee.' }}</div>
                </div>
              </template>
              <template #realUseCases>
                <div class="tooltip-section">
                  <div class="tooltip-section-title">🎯 Cas d'Usage Reel</div>
                  <div class="tooltip-section-text tooltip-multiline">{{ metric.realUseCases || 'Exemples d\'application selon le contexte.' }}</div>
                </div>
              </template>
            </MetricTooltip>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr class="value-row">
          <td class="row-label">Valeur pour la période</td>
          <td v-for="metric in displayedMetrics" :key="`value-${metric.label}`" class="metric-value">
            <span :class="['value-cell', getMetricStatus(metric.value15, metric.goodThreshold, metric.excellentThreshold)]">
              {{ metric.value15.toFixed(metric.decimals ?? 2) }}{{ metric.suffix ? (' ' + metric.suffix) : '' }}
            </span>
          </td>
        </tr>
        <tr class="average-row">
          <td class="row-label">Moyenne globale</td>
          <td v-for="metric in displayedMetrics" :key="`avg-${metric.label}`" class="metric-average">
            {{ metric.valueGlobal.toFixed(metric.decimals ?? 2) }}{{ metric.suffix ? (' ' + metric.suffix) : '' }}
          </td>
        </tr>
        <tr class="threshold-row">
          <td class="row-label">Seuil</td>
          <td v-for="metric in displayedMetrics" :key="`thr-${metric.label}`" class="metric-threshold">
            {{ metric.excellentThreshold.toFixed(metric.decimals ?? 2) }}{{ metric.suffix ? (' ' + metric.suffix) : '' }}
          </td>
        </tr>
        <tr class="status-row">
          <td class="row-label">Appréciation</td>
          <td v-for="metric in displayedMetrics" :key="`status-${metric.label}`" class="metric-status">
            <span :class="['status-badge', getMetricStatus(metric.value15, metric.goodThreshold, metric.excellentThreshold)]">
              {{ getMetricStatusText(metric.value15, metric.goodThreshold, metric.excellentThreshold) }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SliceAnalysis } from '../../utils/volatilityScore'
import MetricTooltip from '../MetricTooltip.vue'
import {
  buildMetricsConfig,
  formatNumber,
  getMetricClass,
  getMetricStatus,
  getMetricStatusText
} from './MetricsGrid.helpers'

interface AnalysisData {
  symbol?: string
  [key: string]: unknown
}

interface Props {
  analysis: SliceAnalysis
  analysisData: AnalysisData
}

const props = defineProps<Props>()

const displayedMetrics = computed(() => buildMetricsConfig(props.analysis, props.analysisData))
</script>

<style scoped>
.metrics-section {
  background: #0d1117;
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 0; /* Removed margin */
  border: 1px solid #30363d;
  flex-shrink: 0; /* Don't shrink */
}

.metrics-section h4 {
  margin: 0 0 8px 0;
  color: #e2e8f0;
  font-size: 0.9em;
}

.metrics-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8em; /* Reduced font size */
  table-layout: fixed;
}

.metrics-table th {
  background: #1a1f2e;
  color: #e2e8f0;
  font-weight: 600;
  padding: 8px 4px;
  text-align: center;
  border: 1px solid #30363d;
  font-size: 0.85em;
}

.metric-title {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: help;
  border-bottom: 1px dotted rgba(148, 163, 184, 0.7);
  padding-bottom: 1px;
}

.tooltip-multiline {
  white-space: pre-wrap;
}

.metrics-table th.label-column {
  text-align: left;
  background: #0d1117;
  min-width: 140px;
}

.metrics-table td {
  padding: 4px 6px;
  text-align: center;
  border: 1px solid #30363d;
}

.row-label {
  text-align: left;
  padding: 4px 8px;
  font-weight: 500;
  color: #cbd5e0;
  font-size: 0.85em;
  background: #0d1117;
  min-width: 140px;
  border-right: 2px solid #30363d;
}

.value-row .metric-value {
  font-weight: 600;
  font-size: 1em;
}

.value-cell {
  display: inline-block;
  padding: 2px 4px;
  border-radius: 4px;
}

.value-cell.excellent {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.value-cell.good {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.value-cell.acceptable {
  color: #eab308;
  background: rgba(234, 179, 8, 0.1);
}

.value-cell.poor {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.suffix {
  font-size: 0.8em;
  opacity: 0.7;
  margin-left: 2px;
}

.prefix {
  font-size: 0.8em;
  opacity: 0.7;
  margin-right: 2px;
}

.average-row {
  background: rgba(45, 55, 72, 0.3);
}

.metric-average {
  color: #a0aec0;
  font-size: 0.9em;
  opacity: 0.7;
}

.threshold-row {
  background: rgba(45, 55, 72, 0.2);
}

.metric-threshold {
  color: #718096;
  font-size: 0.85em;
  opacity: 0.6;
}

.status-row .metric-status {
  padding: 2px 4px;
}

.status-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.85em;
  font-weight: 500;
}

.status-badge.excellent {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.status-badge.good {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

.status-badge.acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #eab308;
}

.status-badge.poor {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

@media (max-width: 900px) {
  .metrics-section {
    padding: 10px;
  }

  .metrics-table {
    font-size: 0.75em;
  }

  .metrics-table th,
  .metrics-table td {
    padding: 4px 3px;
  }

  .metrics-table th.label-column,
  .row-label {
    min-width: 120px;
  }
}
</style>
