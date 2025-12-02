<template>
  <div class="metrics-section">
    <h4>METRIQUES</h4>
    <table class="metrics-table">
      <thead>
        <tr>
          <th class="label-column"></th>
          <th v-for="metric in displayedMetrics" :key="metric.label" class="metric-header">
            {{ metric.label }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr class="value-row">
          <td class="row-label">Valeur pour la période</td>
          <td v-for="metric in displayedMetrics" :key="`value-${metric.label}`" class="metric-value">
            <span :class="['value-cell', getMetricStatus(metric.value15, metric.goodThreshold, metric.excellentThreshold)]">
              {{ formatNumber(metric.value15, metric.decimals ?? 2) }}<span class="suffix">{{ metric.suffix }}</span>
            </span>
          </td>
        </tr>
        <tr class="average-row">
          <td class="row-label">Moyenne globale</td>
          <td v-for="metric in displayedMetrics" :key="`avg-${metric.label}`" class="metric-average">
            {{ formatNumber(metric.valueGlobal, metric.decimals ?? 2) }}<span class="suffix">{{ metric.suffix }}</span>
          </td>
        </tr>
        <tr class="threshold-row">
          <td class="row-label">Seuil</td>
          <td v-for="metric in displayedMetrics" :key="`thr-${metric.label}`" class="metric-threshold">
            >{{ formatNumber(metric.excellentThreshold, metric.decimals ?? 2) }}
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
import type { SliceAnalysis } from '../../utils/straddleAnalysis'
import {
  buildMetricsConfig,
  formatNumber,
  getMetricClass,
  getMetricStatus,
  getMetricStatusText
} from './MetricsGrid.helpers'

interface AnalysisData {
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
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  border: 1px solid #30363d;
}

.metrics-section h4 {
  margin: 0 0 15px 0;
  color: #e2e8f0;
}

.metrics-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
}

.metrics-table th {
  background: #1a1f2e;
  color: #e2e8f0;
  font-weight: 600;
  padding: 12px 8px;
  text-align: center;
  border: 1px solid #30363d;
  font-size: 0.85em;
}

.metrics-table th.label-column {
  text-align: left;
  background: #0d1117;
  min-width: 140px;
}

.metrics-table td {
  padding: 10px 8px;
  text-align: center;
  border: 1px solid #30363d;
}

.row-label {
  text-align: left;
  padding: 10px 12px;
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
  padding: 4px 8px;
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
  margin-left: 2px;
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
  padding: 6px 8px;
}

.status-badge {
  display: inline-block;
  padding: 4px 8px;
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
</style>
