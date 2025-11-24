<template>
  <tr>
    <td>
      <div class="filename-content">
        <span class="file-icon">📄</span>
        <span>{{ file.filename }}</span>
      </div>
    </td>
    <td>{{ formatSize(file.size_bytes) }}</td>
    <td>{{ file.event_count ? file.event_count.toLocaleString() : 'N/A' }}</td>
    <td>{{ file.created }}</td>
    <td>{{ file.modified }}</td>
    <td class="actions-col">
      <button class="btn-delete" title="Supprimer ce fichier" @click="$emit('delete')">
        🗑️
      </button>
    </td>
  </tr>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'

interface CalendarFileInfo {
  filename: string
  path: string
  size_bytes: number
  created: string
  modified: string
  event_count: number | null
}

defineProps<{
  file: CalendarFileInfo
}>()

defineEmits<{
  delete: []
}>()

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<style scoped>
.filename-content {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e6edf3;
  font-weight: 500;
}

.file-icon {
  font-size: 1.2em;
}

.actions-col {
  width: 80px;
  text-align: center;
}

.btn-delete {
  padding: 6px 12px;
  background: #da3633;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1.1em;
  transition: all 0.2s;
}

.btn-delete:hover {
  background: #f85149;
  transform: scale(1.1);
}

.btn-delete:active {
  transform: scale(0.95);
}
</style>
