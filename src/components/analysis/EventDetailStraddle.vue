<script setup lang="ts">
interface Props {
  avgATR: number
  avgPeakDelay: number
}

defineProps<Props>()
</script>

<template>
  <div class="straddle-section">
    <h3 class="section-title">Straddle Setup</h3>
    <div class="setup-grid">
      <div class="setup-item">
        <span class="setup-label">Stop Loss</span>
        <span class="setup-value">{{ Math.round((avgATR || 0) * 1.5) }}p</span>
      </div>
      <div class="setup-item">
        <span class="setup-label">Take Profit</span>
        <span class="setup-value">{{ Math.round((avgATR || 0) * 3) }}p</span>
      </div>
      <div class="setup-item">
        <span class="setup-label">Ratio</span>
        <span class="setup-value">1:2</span>
      </div>
      <div class="setup-item">
        <span class="setup-label">Placement</span>
        <span class="setup-value">{{ Math.round((avgPeakDelay || 0) * 60) }}sec avant</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.straddle-section {
  padding: 24px;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #4ecdc4;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.setup-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
}

.setup-item {
  background: linear-gradient(135deg, rgba(147, 51, 234, 0.15), rgba(79, 70, 229, 0.15));
  border: 1px solid rgba(147, 51, 234, 0.3);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
}

.setup-label {
  display: block;
  font-size: 11px;
  color: #a0aec0;
  text-transform: uppercase;
  margin-bottom: 6px;
}

.setup-value {
  display: block;
  font-size: 16px;
  font-weight: 700;
  color: #a855f7;
}
</style>
