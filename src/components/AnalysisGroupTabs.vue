<template>
  <div class="tab-group-selector">
    <button
      class="group-button"
      :class="{ active: modelValue === 'correlation' }"
      @click="$emit('update:modelValue', 'correlation')"
    >
      📈 Analyse de Corrélation
    </button>
    <button
      class="group-button"
      :class="{ active: modelValue === 'retrospective' }"
      @click="$emit('update:modelValue', 'retrospective')"
    >
      📊 Métriques Rétrospectives
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{ modelValue: 'correlation' | 'retrospective' }>()
defineEmits<{ 'update:modelValue': [value: 'correlation' | 'retrospective'] }>()
</script>

<style scoped>
.tab-group-selector {
  display: flex;
  gap: 10px;
  padding: 15px 20px;
  background: #0d1117;
  border-bottom: 3px solid #1f6feb;
  align-items: center;
}

.group-button {
  flex: 0 1 auto;
  padding: 12px 18px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #8b949e;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
}

.group-button:hover {
  background: #1c2128;
  border-color: #58a6ff;
  color: #58a6ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.3);
}

.group-button.active {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: #ffffff;
  border-color: #58a6ff;
  box-shadow: 0 4px 12px rgba(88, 166, 255, 0.4);
}
</style>
