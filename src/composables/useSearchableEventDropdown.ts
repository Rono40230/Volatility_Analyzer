import { ref, computed } from 'vue'

export interface EventOption {
  name: string
  label: string
  count: number
}

export function useSearchableEventDropdown() {
  const searchQuery = ref('')
  const isOpen = ref(false)

  const filterEvents = (events: EventOption[]): EventOption[] => {
    if (!searchQuery.value.trim()) return events
    const query = searchQuery.value.toLowerCase()
    return events.filter(e => 
      e.label.toLowerCase().includes(query) || 
      e.name.toLowerCase().includes(query)
    )
  }

  const clearSearch = () => {
    searchQuery.value = ''
  }

  const closeDropdown = () => {
    isOpen.value = false
  }

  const openDropdown = () => {
    isOpen.value = true
  }

  const toggleDropdown = () => {
    isOpen.value = !isOpen.value
  }

  return {
    searchQuery,
    isOpen,
    filterEvents,
    clearSearch,
    closeDropdown,
    openDropdown,
    toggleDropdown
  }
}
