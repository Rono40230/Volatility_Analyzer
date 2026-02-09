<script setup lang="ts">
import { useConversionTable } from '../composables/useConversionTable'
import '../styles/conversion-table.css'

const emit = defineEmits<{ (e: 'close'): void }>()

const {
  conversions, loading, saving, editingSymbol, editRow,
  newSymbol, newPipValue, newMt5Digits, showAddRow, errorMsg,
  computeDigits, computeExample, startEdit, cancelEdit, saveEdit, addNew, resetToDefault,
} = useConversionTable()
</script>

<template>
  <div
    class="conv-modal-overlay"
    @click.self="emit('close')"
  >
    <div class="conv-modal">
      <div class="conv-header">
        <div class="conv-header-left">
          <span class="conv-header-icon">📊</span>
          <div>
            <h2>Table de Conversion</h2>
            <p class="conv-subtitle">
              Points MT5 → Pips — Modifiez les valeurs pour vos instruments
            </p>
          </div>
        </div>
        <button
          class="conv-btn-close"
          @click="emit('close')"
        >
          ✕
        </button>
      </div>

      <div
        v-if="errorMsg"
        class="conv-error"
      >
        {{ errorMsg }}
        <button @click="errorMsg = ''">
          ✕
        </button>
      </div>

      <div class="conv-info">
        <span>💡</span>
        <span>Les valeurs <strong>personnalisées</strong> (surlignées) s'appliquent automatiquement à <strong>tous les calculs</strong>.</span>
      </div>

      <div
        v-if="loading"
        class="conv-loading"
      >
        Chargement...
      </div>
      <div
        v-else
        class="conv-table-wrap"
      >
        <table class="conv-table">
          <thead>
            <tr>
              <th>SYMBOLE</th>
              <th>PIP VALUE</th>
              <th>MT5 DÉCIMALES</th>
              <th>EXEMPLE</th>
              <th>ACTIONS</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entry in conversions"
              :key="entry.symbol"
              :class="{ 'conv-custom-row': entry.is_custom }"
            >
              <template v-if="editingSymbol === entry.symbol">
                <td>
                  <span class="conv-symbol">{{ entry.symbol }}</span>
                </td>
                <td>
                  <div class="conv-pip-edit">
                    <input
                      v-model="editRow.pip_value"
                      type="text"
                      class="conv-input"
                    >
                    <span class="conv-unit-label">pips</span>
                  </div>
                </td>
                <td class="conv-mt5-digits">
                  {{ computeDigits(editRow.pip_value) }}
                </td>
                <td class="conv-conversion">
                  {{ computeExample(editRow.pip_value, editRow.mt5_digits) }}
                </td>
                <td class="conv-actions">
                  <button
                    class="conv-btn conv-btn-ok"
                    :disabled="saving"
                    @click="saveEdit"
                  >
                    ✓
                  </button>
                  <button
                    class="conv-btn conv-btn-no"
                    @click="cancelEdit"
                  >
                    ✕
                  </button>
                </td>
              </template>
              <template v-else>
                <td>
                  <span class="conv-symbol">
                    {{ entry.symbol }}
                    <span
                      v-if="entry.is_custom"
                      class="conv-badge"
                    >personnalisé</span>
                  </span>
                </td>
                <td>{{ entry.pip_value }} pips</td>
                <td class="conv-mt5-digits">
                  {{ computeDigits(entry.pip_value.toString()) }}
                </td>
                <td class="conv-conversion">
                  {{ computeExample(entry.pip_value.toString(), entry.mt5_digits.toString()) }}
                </td>
                <td class="conv-actions">
                  <button
                    class="conv-btn"
                    title="Modifier"
                    @click="startEdit(entry)"
                  >
                    ✏️
                  </button>
                  <button
                    v-if="entry.is_custom"
                    class="conv-btn conv-btn-reset"
                    title="Réinitialiser"
                    @click="resetToDefault(entry.symbol)"
                  >
                    ↩
                  </button>
                </td>
              </template>
            </tr>
            <tr
              v-if="showAddRow"
              class="conv-add-row"
            >
              <td>
                <input
                  v-model="newSymbol"
                  type="text"
                  class="conv-input"
                  placeholder="SYMBOL"
                >
              </td>
              <td>
                <div class="conv-pip-edit">
                  <input
                    v-model="newPipValue"
                    type="text"
                    class="conv-input"
                    placeholder="0.0001"
                  >
                  <span class="conv-unit-label">pips</span>
                </div>
              </td>
              <td class="conv-mt5-digits">
                {{ computeDigits(newPipValue) }}
              </td>
              <td class="conv-conversion">
                {{ computeExample(newPipValue, newMt5Digits) }}
              </td>
              <td class="conv-actions">
                <button
                  class="conv-btn conv-btn-ok"
                  :disabled="saving"
                  @click="addNew"
                >
                  ✓
                </button>
                <button
                  class="conv-btn conv-btn-no"
                  @click="showAddRow = false"
                >
                  ✕
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="conv-footer">
        <button
          class="conv-btn-add"
          @click="showAddRow = true"
        >
          ➕ Ajouter un symbole
        </button>
        <button
          class="conv-btn-secondary"
          @click="emit('close')"
        >
          Fermer
        </button>
      </div>
    </div>
  </div>
</template>
