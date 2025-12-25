<template>
  <div class="tab-content">
    <div class="controls-bar">
      <div class="control-group">
        <label for="threshold">Seuil d'occurrences :</label>
        <div class="select-wrapper">
          <select id="threshold" :value="threshold" @change="$emit('update:threshold', Number(($event.target as HTMLSelectElement).value))">
            <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
          </select>
          <span class="select-arrow">▼</span>
        </div>
      </div>
      <div class="stats-badge" v-if="!loading"><strong>{{ events.length }}</strong> types trouvés</div>
    </div>

    <div v-if="loading" class="state-container">
      <div class="spinner"></div>
      <p>Analyse en cours...</p>
    </div>
    
    <div v-else-if="events.length === 0" class="state-container empty">
      <span class="empty-icon">✨</span>
      <p>Aucun événement avec moins de {{ threshold }} occurrences trouvé.</p>
      <p class="sub-text">Votre base de données est propre !</p>
    </div>
    
    <div v-else class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Description de l'événement</th>
            <th class="text-right">Occurrences</th>
            <th class="text-right">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="event in events" :key="event.description">
            <td>{{ event.description }}</td>
            <td class="text-right">
              <span class="count-badge" :class="getCountClass(event.count)">{{ event.count }}</span>
            </td>
            <td class="text-right">
              <button class="btn-icon" title="Voir les événements" @click="('preview', 'description', event.description, event.description)">👁️</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
interface RareEventSummary { description: string; count: number }
defineProps<{ events: RareEventSummary[]; loading: boolean; threshold: number }>()
defineEmits<{ (e: 'update:threshold', value: number): void; (e: 'preview', type: string, value: string, title: string): void }>()

function getCountClass(count: number) {
  if (count === 1) return 'count-1'
  if (count <= 3) return 'count-low'
  return 'count-mid'
}
</script>

<style scoped>
.tab-content { display: flex; flex-direction: column; flex: 1; min-height: 0; gap: 20px; }
.controls-bar { display: flex; justify-content: space-between; align-items: center; background: #0f172a; padding: 12px 16px; border-radius: 8px; border: 1px solid #334155; }
.control-group { display: flex; align-items: center; gap: 12px; }
.control-group label { color: #cbd5e1; font-weight: 500; }
.select-wrapper { position: relative; }
select { appearance: none; background: #334155; border: 1px solid #475569; color: white; padding: 6px 32px 6px 12px; border-radius: 6px; font-size: 0.9rem; cursor: pointer; transition: border-color 0.2s; }
select:hover { border-color: #64748b; }
select:focus { outline: none; border-color: #f59e0b; box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.2); }
.select-arrow { position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: #94a3b8; pointer-events: none; font-size: 0.7rem; }
.stats-badge { background: rgba(59, 130, 246, 0.1); color: #60a5fa; padding: 4px 12px; border-radius: 20px; font-size: 0.875rem; border: 1px solid rgba(59, 130, 246, 0.2); }
.state-container { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #94a3b8; gap: 16px; }
.spinner { width: 40px; height: 40px; border: 3px solid rgba(255, 255, 255, 0.1); border-radius: 50%; border-top-color: #f59e0b; animation: spin 1s linear infinite; }
.empty-icon { font-size: 48px; margin-bottom: 8px; }
.sub-text { font-size: 0.875rem; color: #64748b; }
.table-wrapper { flex: 1; overflow-y: auto; border: 1px solid #334155; border-radius: 8px; background: #0f172a; min-height: 0; }
table { width: 100%; border-collapse: collapse; }
th { background: #1e293b; color: #94a3b8; font-weight: 600; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.05em; padding: 12px 16px; position: sticky; top: 0; border-bottom: 1px solid #334155; }
td { padding: 12px 16px; border-bottom: 1px solid #1e293b; color: #e2e8f0; font-size: 0.9rem; }
tr:last-child td { border-bottom: none; }
tr:hover td { background: rgba(255, 255, 255, 0.02); }
.text-right { text-align: right; }
.count-badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.8rem; font-weight: 600; min-width: 24px; text-align: center; }
.count-1 { background: rgba(239, 68, 68, 0.15); color: #fca5a5; }
.count-low { background: rgba(245, 158, 11, 0.15); color: #fcd34d; }
.count-mid { background: rgba(59, 130, 246, 0.15); color: #93c5fd; }
.btn-icon { background: transparent; border: none; cursor: pointer; font-size: 1.1rem; padding: 4px 8px; border-radius: 4px; transition: background 0.2s; margin-right: 8px; }
.btn-icon:hover { background: rgba(255, 255, 255, 0.1); }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
