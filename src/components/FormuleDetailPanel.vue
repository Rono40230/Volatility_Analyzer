<template>
  <div v-if="formule" class="formule-detail">
    <div class="formule-header">
      <div>
        <h3>{{ formule.titre }}</h3>
        <p class="definition">{{ formule.definition }}</p>
      </div>
      <button class="copy-btn" @click="emit('copy')" title="Copier la formule">
        📋
      </button>
    </div>

    <div class="formule-body">
      <section class="formule-section">
        <h4>📖 Définition</h4>
        <p>{{ formule.definition }}</p>
      </section>

      <section class="formule-section">
        <h4>🧮 Formule</h4>
        <div class="formula-box">
          <code>{{ formule.formule }}</code>
        </div>
      </section>

      <section class="formule-section">
        <h4>📥 Inputs</h4>
        <ul class="inputs-list">
          <li v-for="(input, idx) in formule.inputs" :key="idx">
            {{ input }}
          </li>
        </ul>
      </section>

      <section class="formule-section">
        <h4>📤 Output</h4>
        <div class="output-info">
          <div class="output-line">
            <span class="label">Type:</span>
            <span class="value">{{ formule.output.type }}</span>
          </div>
          <div class="output-line">
            <span class="label">Range:</span>
            <span class="value">{{ formule.output.range }}</span>
          </div>
          <div class="output-line">
            <span class="label">Unité:</span>
            <span class="value">{{ formule.output.unite }}</span>
          </div>
        </div>
      </section>

      <section class="formule-section">
        <h4>📊 Exemple Concret</h4>
        <div class="exemple-box">
          {{ formule.exemple }}
        </div>
      </section>

      <section v-if="formule.notes.length > 0" class="formule-section">
        <h4>⚠️ Notes & Limitations</h4>
        <ul class="notes-list">
          <li v-for="(note, idx) in formule.notes" :key="idx">
            {{ note }}
          </li>
        </ul>
      </section>
    </div>

    <div class="formule-navigation">
      <button :disabled="!prevId" @click="emit('prev')" class="nav-btn">
        ← Précédente
      </button>
      <span class="nav-info">{{ position }} / {{ total }}</span>
      <button :disabled="!nextId" @click="emit('next')" class="nav-btn">
        Suivante →
      </button>
    </div>
  </div>

  <div v-else class="empty-state">
    <div class="empty-icon">🔍</div>
    <p>Sélectionnez une formule dans la liste</p>
  </div>
</template>

<script setup lang="ts">
import '../styles/formule-detail-panel.css'
import type { Formule } from '../data/formules'

interface Props {
  formule: Formule | null
  prevId: string | null
  nextId: string | null
  position: number
  total: number
}

defineProps<Props>()
const emit = defineEmits<{
  copy: []
  prev: []
  next: []
}>()
</script>
