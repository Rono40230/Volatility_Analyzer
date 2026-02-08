import { computed, ref } from 'vue'
import { 
  formaterHeure as formaterHeureUtil, 
  obtenirMinuteEvenement as obtenirMinuteEvenementUtil, 
  obtenirCouleurEvenement, 
  obtenirX as obtenirXUtil, 
  obtenirY as obtenirYUtil 
} from '../components/charts/quarterlyProfileChartUtils'
import { useQuarterlyProfileEvents } from './useQuarterlyProfileEvents'

export interface ChartProps {
  profile: number[]
  optimalEntry?: number
  duration?: number
  entryLabel?: string
  hour?: number
  quarter?: number
  events?: Array<{
    time: string
    name: string
    impact: string
    currency: string
    frequency: number
  }>
}

export function useQuarterlyProfileChart(props: ChartProps) {
  const filterMode = ref<'none' | 'all' | 'peak'>('peak')

  const isExtended = computed(() => props.profile.length > 15)
  const minMinute = computed(() => isExtended.value ? -5 : 0)
  const maxMinute = computed(() => isExtended.value ? 30 : 14)
  const totalRange = computed(() => maxMinute.value - minMinute.value)

  const formaterHeure = (minute: number) => formaterHeureUtil(minute, props.hour, props.quarter)
  const obtenirMinuteEvenement = (timeStr: string) => obtenirMinuteEvenementUtil(timeStr, props.hour, props.quarter)

  const xAxisLabels = computed(() => {
    if (isExtended.value) {
      const labels = []
      for (let i = minMinute.value; i <= maxMinute.value; i += 5) {
        labels.push(i)
      }
      return labels
    }
    return [0, 5, 10, 14]
  })

  const maxIndex = computed(() => maxMinute.value - minMinute.value)
  const profileSlice = computed(() => props.profile.slice(0, maxIndex.value + 1))

  const maxValue = computed(() => {
    const values = profileSlice.value
    const max = values.length > 0 ? Math.max(...values) : 0
    const ceiling = Math.ceil(max)
    return ceiling === 0 ? 1 : ceiling
  })

  const yAxisTicks = computed(() => {
    const max = maxValue.value
    const ticks = []
    let step = 1
    if (max > 20) step = 2
    if (max > 50) step = 5
    
    for (let i = 0; i <= max; i += step) {
      ticks.push(i)
    }
    return ticks
  })

  const obtenirX = (minute: number) => obtenirXUtil(minute, minMinute.value, totalRange.value)
  const obtenirY = (val: number) => obtenirYUtil(val, maxValue.value)

  const points = computed(() => {
    return profileSlice.value.map((val, idx) => {
      const minute = minMinute.value + idx
      return `${obtenirX(minute)},${obtenirY(val)}`
    }).join(' ')
  })

  const areaPath = computed(() => {
    const pts = points.value
    const firstX = obtenirX(minMinute.value)
    const lastX = obtenirX(minMinute.value + profileSlice.value.length - 1)
    const bottomY = 180
    return `M ${firstX},${bottomY} L ${pts} L ${lastX},${bottomY} Z`
  })

  const volatilityZoneWidth = computed(() => {
    if (props.optimalEntry === undefined || props.duration === undefined) return 0
    const startX = obtenirX(props.optimalEntry)
    const endX = obtenirX(props.optimalEntry + props.duration)
    const maxX = 380
    return Math.min(endX, maxX) - startX
  })

  const { processedEvents } = useQuarterlyProfileEvents(props, filterMode)

  return {
    filterMode,
    isExtended,
    xAxisLabels,
    yAxisTicks,
    points,
    areaPath,
    volatilityZoneWidth,
    processedEvents,
    formaterHeure,
    obtenirMinuteEvenement,
    obtenirX,
    obtenirY,
    obtenirCouleurEvenement
  }
}
