<template>
  <section class="import-section">
    <h3>📅 Calendriers Économique</h3>
    <div v-if="calendarsMetadata.length > 0" class="info-box">
      <div>Calendriers: {{ calendarsMetadata.length }}</div>
      <div>Événements: {{ calendarsMetadata.reduce((s, c) => s + c.event_count, 0).toLocaleString() }}</div>
    </div>
    <div v-else class="info-box warning">Aucun calendrier importé.</div>
    <div v-if="calendarsMetadata.length > 0" class="table-container">
      <table class="data-table">
        <thead>
          <tr><th>Nom</th><th>Événements</th><th>Période</th><th>Actions</th></tr>
        </thead>
        <tbody>
          <tr v-for="cal in calendarsMetadata" :key="cal.id" :class="{ 'active-row': isActiveCalendar(cal.id) }">
            <td>
              <span v-if="isActiveCalendar(cal.id)" class="active-badge">✅ Actif</span>
              <strong>{{ cal.name }}</strong>
            </td>
            <td>{{ cal.event_count.toLocaleString() }}</td>
            <td>{{ formatCalendarPeriod(cal) }}</td>
            <td class="actions-cell">
              <button v-if="!isActiveCalendar(cal.id) && !cal.name.includes('Planning Hebdo') && !cal.name.startsWith('ForexFactory_Sync')" class="btn-activate" title="Utiliser ce calendrier" @click="$emit('setActive', cal.id)">Activer</button>
              <button class="btn-delete" @click="$emit('delete', cal.id)">🗑️ Supprimer</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="import-controls">
      <button class="btn-import" :disabled="loading" @click="$emit('import')">
        <span v-if="loading" class="spinner">⏳</span>
        <span v-else>📥</span>
        Importer calendrier
      </button>
      <button class="btn-clean" @click="$emit('clean-rare')">
        🧹 Nettoyage de la base de données
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">

interface CalendarMetadata {
  id: number
  name: string
  event_count: number
  start_date?: string
  end_date?: string
}

defineProps<{
  calendarsMetadata: CalendarMetadata[]
  loading: boolean
  activeCalendarId: number | null
}>()

defineEmits<{
  import: []
  delete: [id: number]
  setActive: [id: number]
  'clean-rare': []
}>()

function isActiveCalendar(id: number): boolean {
  return (new URL(location.href).searchParams.get('activeCalendarId') || localStorage.getItem('activeCalendarId')) === id.toString()
}

function formatCalendarPeriod(calendar: CalendarMetadata): string {
  if (!calendar.start_date && !calendar.end_date) return 'N/A'
  
  try {
    const startDate = calendar.start_date ? new Date(calendar.start_date) : null
    const endDate = calendar.end_date ? new Date(calendar.end_date) : null

    if (!startDate || !endDate) return 'Dates incomplètes'

    // Format spécifique pour les calendriers hebdo synchronisés
    if (calendar.name.startsWith('ForexFactory_Sync')) {
      const startDay = startDate.getDate()
      const startMonth = startDate.toLocaleDateString('fr-FR', { month: 'long' })
      
      const endDay = endDate.getDate()
      const endMonth = endDate.toLocaleDateString('fr-FR', { month: 'long' })
      const endYear = endDate.getFullYear()

      return `News du ${startDay} ${startMonth} au ${endDay} ${endMonth} ${endYear}`
    }

    // Format standard pour les autres calendriers (dd/mm/yyyy)
    const formatDate = (d: Date) => d.toLocaleDateString('fr-FR', { year: 'numeric', month: '2-digit', day: '2-digit' })
    return `du ${formatDate(startDate)} au ${formatDate(endDate)}`
  } catch {
    return 'Format invalide'
  }
}
</script>

<style scoped>
.import-section { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.import-section h3 { color: #e2e8f0; margin-top: 0; }
.info-box { padding: 15px; background: #2d3748; border-radius: 8px; color: #e2e8f0; margin-bottom: 20px; }
.info-box.warning { background: #7f3f1f; color: #fbbf24; }
.table-container { overflow-x: auto; margin-bottom: 20px; }
.data-table { width: 100%; border-collapse: collapse; }
.data-table th { background: #2d3748; padding: 12px; text-align: left; font-weight: 600; color: #e2e8f0; border-bottom: 2px solid #4a5568; }
.data-table td { padding: 12px; border-bottom: 1px solid #2d3748; color: #e2e8f0; }
.btn-import { display: block; width: 100%; padding: 12px 20px; background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%); color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; margin-top: 15px; transition: all 0.3s; }
.btn-import:hover { background: linear-gradient(135deg, #1664d9 0%, #2d7ee5 100%); transform: translateY(-2px); box-shadow: 0 4px 12px rgba(31, 111, 235, 0.4); }
.btn-import:disabled { opacity: 0.7; cursor: not-allowed; }
.import-controls { display: flex; flex-direction: column; gap: 10px; margin-top: 15px; }
.checkbox-label { display: flex; align-items: center; gap: 8px; color: #e2e8f0; font-size: 0.9em; cursor: pointer; }
.checkbox-label input { cursor: pointer; }
.spinner { display: inline-block; animation: spin 1s linear infinite; margin-right: 6px; }
@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
.btn-delete { padding: 6px 12px; background: #dc2626; color: white; border: none; border-radius: 6px; cursor: pointer; }
.active-row { background: rgba(34, 197, 94, 0.1); }
.active-badge { display: inline-block; background: #22c55e; color: white; padding: 2px 6px; border-radius: 4px; font-size: 0.7em; margin-right: 6px; }
.actions-cell { display: flex; gap: 8px; }
.btn-activate { padding: 6px 12px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; }
.btn-activate:hover { background: #2563eb; }
.btn-clean { display: block; width: 100%; padding: 12px 20px; background: #d97706; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; margin-top: 5px; transition: all 0.3s; }
.btn-clean:hover { background: #b45309; transform: translateY(-2px); }
</style>
