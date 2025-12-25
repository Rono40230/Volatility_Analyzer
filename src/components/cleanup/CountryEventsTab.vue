<template>
  <div class="tab-content">
    <div class="controls-bar">
      <div class="control-group">
        <label>Sélectionnez un pays à supprimer :</label>
      </div>
      <div class="stats-badge" v-if="!loading">
        <strong>{{ countries.length }}</strong> pays trouvés
        <span class="separator">•</span>
        <strong>{{ totalEvents.toLocaleString() }}</strong> événements
      </div>
    </div>

    <div v-if="loading" class="state-container">
      <div class="spinner"></div>
      <p>Chargement des pays...</p>
    </div>

    <div v-else class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Pays / Devise</th>
            <th class="text-right">Événements</th>
            <th class="text-right">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in countries" :key="c.symbol">
            <td><strong>{{ c.country_name }}</strong> <span class="text-muted">({{ c.symbol }})</span></td>
            <td class="text-right">{{ c.count.toLocaleString() }}</td>
            <td class="text-right action-cell">
              <button class="btn-icon" title="Voir les événements" @click="$emit('preview', 'symbol', c.symbol, c.country_name)">
                👁️
              </button>
              <button class="btn-sm-danger" @click="$emit('delete', c)">
                Supprimer
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, computed } from 'vue'

interface CurrencySummary { symbol: string; country_name: string; count: number }

const props = defineProps<{
  countries: CurrencySummary[]
  loading: boolean
}>()

defineEmits<{
  (e: 'preview', type: string, value: string, title: string): void
  (e: 'delete', country: CurrencySummary): void
}>()

const totalEvents = computed(() => props.countries.reduce((acc, c) => acc + c.count, 0))
</script>

<style scoped>
.tab-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  gap: 20px;
}

.controls-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #0f172a;
  padding: 12px 16px;
  border-radius: 8px;
  border: 1px solid #334155;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.control-group label {
  color: #cbd5e1;
  font-weight: 500;
}

.stats-badge {
  background: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.875rem;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.separator {
  margin: 0 8px;
  opacity: 0.5;
}

.state-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  gap: 16px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  border-top-color: #f59e0b;
  animation: spin 1s linear infinite;
}

.table-wrapper {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #334155;
  border-radius: 8px;
  background: #0f172a;
  min-height: 0;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th {
  background: #1e293b;
  color: #94a3b8;
  font-weight: 600;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 12px 16px;
  position: sticky;
  top: 0;
  border-bottom: 1px solid #334155;
}

td {
  padding: 12px 16px;
  border-bottom: 1px solid #1e293b;
  color: #e2e8f0;
  font-size: 0.9rem;
}

tr:last-child td {
  border-bottom: none;
}

tr:hover td {
  background: rgba(255, 255, 255, 0.02);
}

.text-right {
  text-align: right;
}

.text-muted {
  color: #64748b;
  font-weight: normal;
  font-size: 0.85em;
}

.btn-icon {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 1.1rem;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.2s;
  margin-right: 8px;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.1);
}

.btn-sm-danger {
  padding: 4px 12px;
  background: rgba(239, 68, 68, 0.1);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  transition: all 0.2s;
}

.btn-sm-danger:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.4);
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
