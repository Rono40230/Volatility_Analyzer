<template>
  <div class="searchable-dropdown-wrapper">
    <div class="dropdown-header">
      <input
        ref="inputRef"
        v-model="searchQuery"
        type="text"
        placeholder="-- Choisir --"
        class="dropdown-input"
        @focus="openDropdown"
        @blur="handleInputBlur"
        @keydown.escape="closeDropdown"
        @keydown.down="handleKeyDown"
        @keydown.enter.prevent="selectFirstFiltered"
      />
      <button
        class="dropdown-toggle"
        :class="{ active: isOpen }"
        @click="toggleDropdown"
        type="button"
        aria-label="Basculer dropdown"
      >
        ▼
      </button>
      <button
        v-if="searchQuery"
        class="clear-button"
        @click="handleClear"
        type="button"
        aria-label="Effacer la recherche"
      >
        ✕
      </button>
    </div>

    <div v-if="isOpen" class="dropdown-menu">
      <div v-if="filteredEvents.length === 0" class="empty-message">
        📭 Aucun événement trouvé
      </div>
      <button
        v-for="(event, index) in filteredEvents"
        :key="event.name"
        class="dropdown-item"
        :class="{ highlighted: index === highlightedIndex }"
        @click="selectEvent(event)"
        @mouseenter="highlightedIndex = index"
      >
        <span class="event-name">{{ event.label }}</span>
        <span class="event-count">({{ event.count }} occurrences)</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useSearchableEventDropdown, type EventOption } from '../composables/useSearchableEventDropdown'

interface Props {
  modelValue: string
  events: EventOption[]
  loading?: boolean
  error?: string | null
}

interface Emits {
  (e: 'update:modelValue', value: string): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  error: null
})

const emit = defineEmits<Emits>()

const inputRef = ref<HTMLInputElement | null>(null)
const highlightedIndex = ref(-1)
const { searchQuery, isOpen, closeDropdown, openDropdown, toggleDropdown } = useSearchableEventDropdown()

const filteredEvents = computed(() => {
  if (!searchQuery.value.trim()) return props.events
  const query = searchQuery.value.toLowerCase()
  return props.events.filter(e =>
    e.label.toLowerCase().includes(query) ||
    e.name.toLowerCase().includes(query)
  )
})

const handleClear = () => {
  searchQuery.value = ''
  emit('update:modelValue', '')
  nextTick(() => inputRef.value?.focus())
}

const selectEvent = (event: EventOption) => {
  searchQuery.value = event.label
  emit('update:modelValue', event.name)
  closeDropdown()
}

const selectFirstFiltered = () => {
  if (filteredEvents.value.length > 0) {
    selectEvent(filteredEvents.value[0])
  }
}

const handleKeyDown = () => {
  if (filteredEvents.value.length === 0) return
  highlightedIndex.value = Math.min(highlightedIndex.value + 1, filteredEvents.value.length - 1)
}

const handleInputBlur = () => {
  setTimeout(() => {
    closeDropdown()
  }, 150)
}
</script>

<style scoped>
.searchable-dropdown-wrapper { position: relative; }
.dropdown-header { display: flex; align-items: center; width: 700px; padding: 12px 16px; border: 2px solid #ccc; border-radius: 8px; background: #ffffff; transition: all 0.3s; overflow: hidden; gap: 8px; box-sizing: border-box; }
.dropdown-header:hover { border-color: #999; background: #f9f9f9; }
.dropdown-input { flex: 1; background: transparent; border: none; color: #000000; font-size: 1em; outline: none; cursor: text; padding: 0; margin: 0; }
.dropdown-input::placeholder { color: #999999; }
.dropdown-toggle { padding: 0; margin: 0; background: transparent; border: none; color: #666666; cursor: pointer; transition: transform 0.2s, color 0.2s; font-size: 0.8em; flex-shrink: 0; }
.dropdown-toggle:hover { color: #000000; }
.dropdown-toggle.active { transform: rotate(180deg); color: #000000; }
.clear-button { padding: 0; margin: 0; background: transparent; border: none; color: #999999; cursor: pointer; font-size: 1em; transition: color 0.2s; line-height: 1; flex-shrink: 0; }
.clear-button:hover { color: #cc0000; }
.dropdown-menu { position: absolute; top: 100%; left: 0; width: 700px; margin-top: 4px; background: #ffffff; border: 2px solid #ccc; border-radius: 8px; max-height: 300px; overflow-y: auto; z-index: 10; box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15); }
.empty-message { padding: 16px; text-align: center; color: #999999; font-size: 0.9em; }
.dropdown-item { display: flex; align-items: center; gap: 12px; width: 100%; padding: 12px 16px; background: transparent; border: none; color: #000000; text-align: left; cursor: pointer; transition: all 0.2s; font-size: 0.95em; box-sizing: border-box; }
.dropdown-item:hover, .dropdown-item.highlighted { background: #e8f0ff; color: #0066cc; }
.event-name { font-weight: 500; flex: 1; }
.event-count { color: #999999; font-size: 0.85em; white-space: nowrap; }
</style>
