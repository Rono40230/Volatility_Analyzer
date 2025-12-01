<template>
  <div v-if="isOpen" class="formulas-overlay" @click.self="close">
    <div class="formulas-container">
      <div class="formulas-header">
        <div class="formulas-title">
          <span class="formulas-title-icon">🧮</span>
          <h2 class="formulas-title-text">Formules & Calculs</h2>
        </div>
        <button class="formulas-close" @click="close">✕</button>
      </div>

      <div class="formulas-body">
        <nav class="formulas-sidebar">
          <div class="formulas-search">
            <input v-model="searchQuery" type="text" placeholder="Chercher une formule..." />
          </div>

          <div class="formulas-categories">
            <button
              :class="['formulas-category-btn', { active: selectedCategory === 'all' }]"
              @click="selectCategory('all')"
            >
              <span class="formulas-category-emoji">📋</span>
              <span class="formulas-category-label">Toutes les formules</span>
              <span class="formulas-category-count">({{ allFormules.length }})</span>
            </button>

            <div class="formulas-separator"></div>

            <button
              v-for="cat in categories"
              :key="cat.id"
              :class="['formulas-category-btn', { active: selectedCategory === cat.id }]"
              @click="selectCategory(cat.id)"
            >
              <span class="formulas-category-emoji">{{ cat.emoji }}</span>
              <span class="formulas-category-label">{{ cat.titre }}</span>
              <span class="formulas-category-count">({{ cat.formules.length }})</span>
            </button>
          </div>
        </nav>

        <div class="formulas-content">
          <FormuleDetailPanel
            :formule="formuleSélectionnée"
            :prev-id="formulePrecedente"
            :next-id="formuleSuivante"
            :position="formulasTriees.findIndex(f => f.id === selectedFormuleId) + 1"
            :total="formulasTriees.length"
            @copy="copierFormule"
            @prev="selectedFormuleId = formulePrecedente || selectedFormuleId"
            @next="selectedFormuleId = formuleSuivante || selectedFormuleId"
          />
        </div>
      </div>

      <div class="formulas-footer">
        <button class="formulas-btn formulas-btn-export">📥 Exporter PDF</button>
        <button class="formulas-btn formulas-btn-close" @click="close">Fermer</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import '../styles/formulas-modal.css'
import FormuleDetailPanel from './FormuleDetailPanel.vue'
import { useFormulasLogic } from '../composables/useFormulasLogic'

interface Props {
  isOpen: boolean
}

defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

const {
  searchQuery,
  selectedCategory,
  selectedFormuleId,
  allFormules,
  selectCategory,
  formulasTriees,
  formuleSélectionnée,
  formulePrecedente,
  formuleSuivante,
  copierFormule,
  categories
} = useFormulasLogic()

const close = () => emit('close')
</script>

