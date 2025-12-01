import { ref, computed } from 'vue'
import { categories, formules, type Formule } from '../data/formules'

export function useFormulasLogic() {
  const searchQuery = ref('')
  const selectedCategory = ref('all')
  const selectedFormuleId = ref('')

  const allFormules = computed(() => {
    return Object.values(formules) as Formule[]
  })

  const selectCategory = (catId: string) => {
    selectedCategory.value = catId
    selectedFormuleId.value = ''
  }

  const formulasTriees = computed(() => {
    let formulesList: Formule[] = []

    if (selectedCategory.value === 'all') {
      formulesList = allFormules.value
    } else {
      const category = categories.find(c => c.id === selectedCategory.value)
      if (category) {
        formulesList = category.formules
          .map(id => formules[id])
          .filter(f => f !== undefined) as Formule[]
      }
    }

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      formulesList = formulesList.filter(
        f =>
          f.titre.toLowerCase().includes(query) ||
          f.definition.toLowerCase().includes(query)
      )
    }

    return formulesList
  })

  const formuleSélectionnée = computed(() => {
    if (!selectedFormuleId.value && formulasTriees.value.length > 0) {
      selectedFormuleId.value = formulasTriees.value[0].id
    }
    return formules[selectedFormuleId.value] || null
  })

  const formulePrecedente = computed(() => {
    const idx = formulasTriees.value.findIndex(f => f.id === selectedFormuleId.value)
    return idx > 0 ? formulasTriees.value[idx - 1].id : null
  })

  const formuleSuivante = computed(() => {
    const idx = formulasTriees.value.findIndex(f => f.id === selectedFormuleId.value)
    return idx < formulasTriees.value.length - 1 ? formulasTriees.value[idx + 1].id : null
  })

  const copierFormule = () => {
    if (formuleSélectionnée.value) {
      navigator.clipboard.writeText(formuleSélectionnée.value.formule)
    }
  }

  return {
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
  }
}
