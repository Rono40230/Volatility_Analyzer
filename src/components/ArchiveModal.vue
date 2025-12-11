<template>
  <div v-if="show" class="modal-overlay" @click.self="closeModal">
    <div class="modal-content">
      <div class="modal-header"><h2>💾 Archiver l'analyse</h2><button class="close-btn" @click="closeModal">✕</button></div>
      <div class="modal-body">
        <div class="form-group"><label>Type d'archive</label><input type="text" :value="archiveType" disabled class="input-disabled"></div>
        <div class="form-group"><label>Période</label><input type="text" :value="periodLabel" disabled class="input-disabled"></div>
        <div class="form-group"><label>Nom de l'archive</label><input v-model="archiveTitle" type="text" class="input-field" placeholder="Nom de l'archive"></div>
        <div class="form-group"><label>Commentaires</label><textarea v-model="comment" class="textarea-field" placeholder="Ajoutez des notes ou commentaires..." rows="4" /></div>
      </div>
      <div class="modal-footer"><button class="btn-cancel" @click="closeModal">Annuler</button><button class="btn-save" :disabled="saving" @click="handleSave">{{ saving ? 'Sauvegarde...' : 'Archiver' }}</button></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useArchiveStore } from '../stores/archiveStore'

interface Props {
  show: boolean
  archiveType: string
  periodStart: string
  periodEnd: string
  symbol?: string
  timeframe?: string
  eventName?: string
  eventNameFr?: string
  eventFlag?: string
  dataJson: string
  defaultTitle?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'close': []
  'saved': []
}>()

const archiveStore = useArchiveStore()

const archiveTitle = ref('')
const comment = ref('')
const saving = ref(false)

const formatDate = (dateStr: string) => {
  // Si la chaîne semble déjà formatée (contient des espaces, pas de T typique ISO), on retourne tel quel
  // Ou si c'est une date invalide pour JS
  const date = new Date(dateStr)
  if (isNaN(date.getTime()) || (dateStr.includes(' ') && !dateStr.includes('T') && !dateStr.includes(':'))) {
    return dateStr
  }
  
  return date.toLocaleDateString('fr-FR', { 
    day: 'numeric', 
    month: 'long', 
    year: 'numeric' 
  })
}

const periodLabel = computed(() => {
  return `du ${formatDate(props.periodStart)} au ${formatDate(props.periodEnd)}`
})

// Générer le titre par défaut
watch(() => props.show, (newVal) => {
  if (newVal) {
    if (props.defaultTitle) {
      archiveTitle.value = props.defaultTitle
      comment.value = ''
      return
    }

    const periodStr = `(${periodLabel.value})`
    
    if (props.symbol && props.timeframe) {
      archiveTitle.value = `${periodStr} ${props.symbol} en ${props.timeframe}`
    } else if (props.symbol) {
      archiveTitle.value = `${periodStr} ${props.symbol}`
    } else if (props.eventName) {
      let title = `${periodStr}`
      if (props.eventFlag) {
        title += ` ${props.eventFlag}`
      }
      title += ` ${props.eventName}`
      if (props.eventNameFr) {
        title += ` (${props.eventNameFr})`
      }
      archiveTitle.value = title
    } else if (props.archiveType === 'Heatmap') {
      archiveTitle.value = `${periodStr} Heatmap`
    } else {
      archiveTitle.value = `${periodStr} Archive`
    }
    
    comment.value = ''
  }
})

async function handleSave() {
  if (!archiveTitle.value.trim()) {
    // Validation silencieuse - l'input field a focus/placeholder
    return
  }

  saving.value = true
  try {
    await archiveStore.saveArchive(
      archiveTitle.value,
      props.archiveType,
      props.periodStart,
      props.periodEnd,
      comment.value || null,
      props.dataJson
    )
    emit('saved')
    closeModal()
  } catch (error) {
    // Erreur gérée silencieusement - fermer la modal
  } finally {
    saving.value = false
  }
}

function closeModal() {
  emit('close')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-content {
  background: #1a202c;
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  border: 1px solid #2d3748;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 25px;
  border-bottom: 1px solid #2d3748;
}

.modal-header h2 {
  margin: 0;
  color: #e2e8f0;
  font-size: 1.5em;
}

.close-btn {
  background: none;
  border: none;
  color: #cbd5e0;
  font-size: 1.5em;
  cursor: pointer;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #2d3748;
  color: #fff;
}

.modal-body {
  padding: 25px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  color: #cbd5e0;
  font-weight: 600;
  font-size: 0.95em;
}

.input-field,
.input-disabled,
.textarea-field {
  width: 100%;
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #4a5568;
  background: #2d3748;
  color: #e2e8f0;
  font-size: 1em;
  transition: all 0.2s;
}

.input-field:focus,
.textarea-field:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.input-disabled {
  background: #1a202c;
  color: #a0aec0;
  cursor: not-allowed;
}

.textarea-field {
  resize: vertical;
  font-family: inherit;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 25px;
  border-top: 1px solid #2d3748;
}

.btn-cancel,
.btn-save {
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-cancel {
  background: #2d3748;
  color: #cbd5e0;
}

.btn-cancel:hover {
  background: #4a5568;
}

.btn-save {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-save:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
