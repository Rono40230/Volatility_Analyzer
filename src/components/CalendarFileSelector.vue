<template>
  <div class="calendar-file-selector">
    <label for="calendar-select">📅 Fichier calendrier :</label>
    <select 
      id="calendar-select" 
      v-model="selectedFile" 
      class="calendar-dropdown"
      :disabled="loading || files.length === 0"
      @change="handleFileChange"
    >
      <option
        v-if="files.length === 0"
        value=""
      >
        Aucun fichier disponible
      </option>
      <option 
        v-for="file in files" 
        :key="file.path" 
        :value="file.filename"
      >
        {{ formatFileLabel(file) }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface CalendarFileInfo {
  filename: string
  path: string
  size_bytes: number
  created: string
  modified: string
  event_count: number | null
  date_range: string | null
}

const emit = defineEmits<{
  'file-selected': [filename: string]
}>()

const files = ref<CalendarFileInfo[]>([])
const selectedFile = ref<string>('')
const loading = ref(false)

// Charger la liste des fichiers
async function loadFiles() {
  loading.value = true
  try {
    const result = await invoke<CalendarFileInfo[]>('list_calendar_files')
    files.value = result
    
    // Si aucun fichier n'est sélectionné, charger le dernier choix sauvegardé
    if (!selectedFile.value && files.value.length > 0) {
      const saved = await invoke<string | null>('get_selected_calendar_file')
      
      if (saved && files.value.some(f => f.filename === saved)) {
        // Le fichier sauvegardé existe toujours
        selectedFile.value = saved
      } else {
        // Sélectionner le plus récent par défaut
        const mostRecent = files.value.sort((a, b) => 
          b.modified.localeCompare(a.modified)
        )[0]
        selectedFile.value = mostRecent.filename
        
        // Sauvegarder ce choix
        await invoke('set_selected_calendar_file', { filename: mostRecent.filename })
      }
      
      // Émettre l'événement pour le fichier initial
      emit('file-selected', selectedFile.value)
    }
  } catch (e) {
    // Erreur silencieuse - fichiers non disponibles
  } finally {
    loading.value = false
  }
}

// Gérer le changement de fichier
async function handleFileChange() {
  if (selectedFile.value) {
    // Sauvegarder le choix
    try {
      await invoke('set_selected_calendar_file', { filename: selectedFile.value })
      emit('file-selected', selectedFile.value)
    } catch (e) {
      // Erreur silencieuse - sauvegarde échouée
    }
  }
}

// Formater le label du fichier pour le dropdown - même format que CalendarImportSection
function formatFileLabel(file: CalendarFileInfo): string {
  if (file.date_range) {
    // Format du date_range du backend : "du 2025-01-01 au 2026-01-31"
    const match = file.date_range.match(/du (\d{4}-\d{2}-\d{2}) au (\d{4}-\d{2}-\d{2})/)
    
    if (match) {
      const [, startStr, endStr] = match
      
      try {
        // Parser au format YYYY-MM-DD
        const [startYear, startMonth, startDay] = startStr.split('-').map(Number)
        const [endYear, endMonth, endDay] = endStr.split('-').map(Number)
        
        const startDate = new Date(startYear, startMonth - 1, startDay)
        const endDate = new Date(endYear, endMonth - 1, endDay)
        
        // Vérifier que les dates sont valides
        if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
          return file.filename
        }
        
        // Fichiers ForexFactory_Sync - format spécial
        if (file.filename.startsWith('ForexFactory_Sync')) {
          const startDayFmt = startDate.getDate()
          const startMonthFmt = startDate.toLocaleDateString('fr-FR', { month: 'long' })
          const endDayFmt = endDate.getDate()
          const endMonthFmt = endDate.toLocaleDateString('fr-FR', { month: 'long' })
          const endYearFmt = endDate.getFullYear()
          
          return `News du ${startDayFmt} ${startMonthFmt} au ${endDayFmt} ${endMonthFmt} ${endYearFmt}`
        }
        
        // Format standard: du DD/MM/YYYY au DD/MM/YYYY
        const formatDate = (d: Date) => d.toLocaleDateString('fr-FR', { year: 'numeric', month: '2-digit', day: '2-digit' })
        return `du ${formatDate(startDate)} au ${formatDate(endDate)}`
      } catch {
        return file.filename
      }
    }
  }
  return file.filename
}

// Exposer une méthode pour recharger depuis le parent
defineExpose({
  refreshFiles: loadFiles
})

onMounted(() => {
  loadFiles()
})
</script>

<style scoped>
.calendar-file-selector {
  display: flex;
  align-items: center;
  gap: 12px;
}

label {
  font-weight: 600;
  color: #e6edf3;
  font-size: 0.95em;
  white-space: nowrap;
}

.calendar-dropdown {
  padding: 8px 16px;
  background: white;
  color: #000000;
  border: 2px solid #30363d;
  border-radius: 6px;
  font-size: 0.95em;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 500px;
  max-width: 650px;
}

.calendar-dropdown:hover:not(:disabled) {
  border-color: #58a6ff;
  background: #f0f4ff;
}

.calendar-dropdown:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.1);
}

.calendar-dropdown:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.calendar-dropdown option {
  background: white;
  color: #000000;
  padding: 8px;
}
</style>
