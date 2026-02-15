<template>
  <div class="heatmap-header">
    <div class="heatmap-scale">
      <button class="scale-item" :class="{ active: currentFilter === 70 }" @click="$emit('filter-click', 70)" title="Filtrer : Score ≥ 70">
        <span class="scale-color heat-very-high" />Score > 70 (Excellent)
      </button>
      <button class="scale-item" :class="{ active: currentFilter === 40 }" @click="$emit('filter-click', 40)" title="Filtrer : Score ≥ 40">
        <span class="scale-color heat-medium" />Score 40-70 (Moyen)
      </button>
      <button class="scale-item" :class="{ active: currentFilter === 0 }" @click="$emit('filter-click', 0)" title="Tout afficher">
        <span class="scale-color heat-very-low" />Score &lt; 40 (Faible)
      </button>
    </div>

    <div class="header-actions">
      <div class="limit-control">
        <label for="heatmap-limit">Evenements:</label>
        <select id="heatmap-limit" :value="maxEvents ?? 50" @change="onLimitChange">
          <option :value="50">50</option>
          <option :value="150">150</option>
          <option :value="300">300</option>
          <option :value="0">Tous (lent)</option>
        </select>
      </div>
      <button 
        class="action-button reload-button"
        title="Recharger la heatmap"
        @click="$emit('reload')"
      >
        🔄 Recharger
      </button>
      <button 
        class="action-button archive-button"
        title="Archiver la heatmap"
        @click="$emit('archive')"
      >
        💾 Archiver la heatmap
      </button>
      <button 
        class="action-button archive-button"
        title="Archiver les 5 meilleurs événements/Paire"
        @click="$emit('archive-top-5')"
      >
        ⭐ Top 5 par Paire
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  currentFilter?: number
  maxEvents?: number
}>()

const emit = defineEmits<{
  'filter-click': [value: number]
  'limit-change': [value: number]
  'reload': []
  'archive': []
  'archive-top-5': []
}>()

function onLimitChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const value = Number.parseInt(target.value, 10)
  if (!Number.isNaN(value)) {
    emit('limit-change', value)
  }
}
</script>

<style scoped>
.heatmap-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding-bottom: 15px; border-bottom: 2px solid #2d3748; flex-wrap: wrap; gap: 20px; }
.heatmap-scale { display: flex; gap: 15px; align-items: center; font-size: 0.85em; color: #cbd5e0; flex-wrap: wrap; }

.scale-item { 
  display: flex; 
  align-items: center; 
  gap: 8px; 
  white-space: nowrap; 
  background: transparent; 
  border: 1px solid transparent; 
  color: #cbd5e0; 
  padding: 6px 12px; 
  border-radius: 6px; 
  cursor: pointer; 
  transition: all 0.2s;
  font-family: inherit;
  font-size: inherit;
}
.scale-item:hover { background: #2d3748; border-color: #4a5568; }
.scale-item.active { background: #2d3748; border-color: #58a6ff; color: #58a6ff; font-weight: 600; }
.scale-color { display: inline-block; width: 16px; height: 16px; border-radius: 3px; border: 1px solid #1a202c; }
.heat-very-high { background: #238636; } /* Vert foncé */
.heat-high { background: #3fb950; }      /* Vert clair */
.heat-medium { background: #d29922; }    /* Orange */
.heat-low { background: #f85149; }       /* Rouge clair */
.heat-very-low { background: #da3633; }  /* Rouge foncé */

.header-actions { display: flex; gap: 10px; }
.limit-control {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #cbd5e0;
  font-size: 0.85em;
}
.limit-control select {
  background: #ffffff;
  border: 1px solid #30363d;
  color: #000000;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 0.9em;
}
.limit-control select option {
  color: #000000;
}
.action-button {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.9em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  border: none;
  white-space: nowrap;
}
.reload-button {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: white;
  box-shadow: 0 2px 8px rgba(56, 139, 253, 0.3);
}
.reload-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(56, 139, 253, 0.4);
}
.archive-button {
  background: #238636;
  color: white;
  box-shadow: 0 2px 8px rgba(35, 134, 54, 0.3);
}
.archive-button:hover {
  background: #2ea043;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(35, 134, 54, 0.4);
}
</style>
