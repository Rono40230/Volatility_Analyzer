<template>
  <div
    class="tooltip-wrapper"
    :data-direction="direction || 'bottom'"
  >
    <div 
      class="tooltip-trigger"
      @mouseenter="showTooltip = true"
      @mouseleave="showTooltip = false"
    >
      <slot />
    </div>
    <transition name="tooltip-fade">
      <div
        v-if="showTooltip"
        class="tooltip-popup"
      >
        <div class="tooltip-content">
          <div class="tooltip-header">
            <span class="tooltip-title">{{ title }}</span>
            <button
              class="close-btn"
              @click="showTooltip = false"
            >
              ✕
            </button>
          </div>
          <div class="tooltip-body">
            <slot name="definition" />
            <slot name="interpretation" />
            <slot name="usage" />
            <slot name="color-scale" />
            <slot name="scoring" />
            <slot name="realUseCases" />
          </div>
        </div>
        <div class="tooltip-arrow" />
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  title: string
  direction?: 'bottom' | 'top'
}>()

const showTooltip = ref(false)
</script>

<style scoped>
.tooltip-wrapper {
  position: relative;
  display: inline-block;
}

.tooltip-trigger {
  cursor: help;
}

.tooltip-popup {
  position: absolute;
  background: linear-gradient(135deg, #1a202c 0%, #2d3748 100%);
  border: 1px solid #667eea;
  border-radius: 8px;
  padding: 0;
  z-index: 1000;
  min-width: 600px;
  max-width: 900px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(102, 126, 234, 0.3);
  backdrop-filter: blur(10px);
  left: 50%;
  transform: translateX(-50%);
}

/* Position vers le bas (défaut) */
.tooltip-popup {
  top: 110%;
}

/* Position vers le haut */
.tooltip-wrapper[data-direction="top"] .tooltip-popup {
  bottom: 110%;
  top: auto;
}

.tooltip-arrow {
  position: absolute;
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  left: 50%;
  transform: translateX(-50%);
}

/* Flèche vers le bas */
.tooltip-arrow {
  top: -8px;
  border-bottom: 8px solid #1a202c;
}

/* Flèche vers le haut */
.tooltip-wrapper[data-direction="top"] .tooltip-arrow {
  bottom: -8px;
  top: auto;
  border-bottom: none;
  border-top: 8px solid #1a202c;
}

.tooltip-content {
  position: relative;
}

.tooltip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #3d4758;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px 8px 0 0;
}

.tooltip-title {
  font-weight: 700;
  color: white;
  font-size: 1.05em;
  letter-spacing: 0.3px;
}

.close-btn {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 1.2em;
  padding: 0;
  opacity: 0.7;
  transition: opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.close-btn:hover {
  opacity: 1;
}

.tooltip-body {
  padding: 16px;
  color: #e2e8f0;
  font-size: 1em;
  line-height: 1.6;
}

/* Slots styling */
.tooltip-body :deep(> *) {
  margin-bottom: 12px;
}

.tooltip-body :deep(> *:last-child) {
  margin-bottom: 0;
}

.tooltip-body :deep(.tooltip-section-title) {
  font-weight: 600;
  color: #58a6ff;
  margin-bottom: 4px;
  font-size: 0.95em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.tooltip-body :deep(.tooltip-section-text) {
  color: #cbd5e0;
  font-size: 0.95em;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Transition animations */
.tooltip-fade-enter-active,
.tooltip-fade-leave-active {
  transition: all 0.2s ease;
}

.tooltip-fade-enter-from,
.tooltip-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}

/* Mobile responsiveness */
@media (max-width: 640px) {
  .tooltip-popup {
    position: fixed;
    bottom: auto;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    min-width: 90vw;
    max-width: 90vw;
  }

  .tooltip-arrow {
    display: none;
  }
}
</style>
