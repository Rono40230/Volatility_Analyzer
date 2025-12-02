<template>
  <div class="view-mode-selector">
    <button
      class="mode-button"
      :class="{ active: modelValue === 'peak-delay' }"
      @click="$emit('update:modelValue', 'peak-delay')"
    >
      ⏰ Peak Delays
    </button>
    <button
      class="mode-button"
      :class="{ active: modelValue === 'decay' }"
      @click="$emit('update:modelValue', 'decay')"
    >
      📉 Volatility Decay
    </button>
    <button
      class="mode-button"
      :class="{ active: modelValue === 'entry-timing' }"
      @click="$emit('update:modelValue', 'entry-timing')"
    >
      📊 Entry Windows
    </button>
    <button
      class="mode-button"
      :class="{ active: modelValue === 'bias' }"
      @click="$emit('update:modelValue', 'bias')"
    >
      🎯 Directional Bias
    </button>
    <button
      class="mode-button"
      :class="{ active: modelValue === 'whipsaw' }"
      @click="$emit('update:modelValue', 'whipsaw')"
    >
      ⚡ Whipsaw Causes
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{ modelValue: 'peak-delay' | 'decay' | 'entry-timing' | 'bias' | 'whipsaw' }>()
defineEmits<{ 'update:modelValue': [value: 'peak-delay' | 'decay' | 'entry-timing' | 'bias' | 'whipsaw'] }>()
</script>

<style scoped>
.view-mode-selector {
  display: flex;
  gap: 15px;
  padding: 20px;
  background: #0d1117;
  border-bottom: 2px solid #30363d;
  align-items: center;
  flex-wrap: wrap;
}

.mode-button {
  flex: 0 1 auto;
  padding: 15px 20px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1.1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.mode-button:hover {
  background: #1c2128;
  border-color: #58a6ff;
  color: #58a6ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.3);
}

.mode-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 4px 12px rgba(88, 166, 255, 0.4);
}
</style>
