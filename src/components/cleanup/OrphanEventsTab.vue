<template>
  <div class="tab-content">
    <div class="controls-bar">
      <div class="control-group">
        <label>Détection des données corrompues</label>
      </div>
    </div>

    <div v-if="loading" class="state-container">
      <div class="spinner"></div>
      <p>Analyse des orphelins...</p>
    </div>

    <div v-else-if="orphans.length === 0" class="state-container empty">
      <span class="empty-icon">🛡️</span>
      <p>Aucun événement orphelin détecté.</p>
      <p class="sub-text">Toutes les données semblent valides.</p>
    </div>

    <div v-else class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Raison</th>
            <th class="text-right">Événements affectés</th>
            <th class="text-right">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="o in orphans" :key="o.reason">
            <td>{{ o.reason }}</td>
            <td class="text-right">
              <span class="count-badge count-1">{{ o.count }}</span>
            </td>
            <td class="text-right">
              <button class="btn-icon" title="Voir les événements" @click="$emit('preview', getOrphanFilterType(o.reason), '', o.reason)">
                👁️
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">

interface OrphanEventSummary { reason: string; count: number }

defineProps<{
  orphans: OrphanEventSummary[]
  loading: boolean
}>()

defineEmits<{
  (e: 'preview', type: string, value: string, title: string): void
}>()

function getOrphanFilterType(reason: string): string {
  if (reason.includes("Symbole")) return "orphan_symbol"
  if (reason.includes("Description")) return "orphan_desc"
  if (reason.includes("Impact")) return "orphan_impact"
  return ""
}
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

.empty-icon {
  font-size: 48px;
  margin-bottom: 8px;
}

.sub-text {
  font-size: 0.875rem;
  color: #64748b;
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

.count-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
  min-width: 24px;
  text-align: center;
}

.count-1 { background: rgba(239, 68, 68, 0.15); color: #fca5a5; }

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

@keyframes spin { to { transform: rotate(360deg); } }
</style>
