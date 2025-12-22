import { computed, ref, onMounted, onUnmounted } from 'vue'

export function useRetroGraphDataPoints(props: {
  atrTimelineBefore?: number[],
  atrTimelineAfter?: number[],
  meilleurMoment?: number,
  eventDatetime?: string,
  pointValue?: number
}) {
  const screenWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1024)
  const Y_BASELINE = 380, Y_MID = 215, Y_TOP = 50
  const pointValue = computed(() => props.pointValue || 1.0)

  const svgMargins = computed(() => {
    if (screenWidth.value > 1024) return { left: 40, right: 999, t0: 445, labelY: 30 }
    if (screenWidth.value > 768) return { left: 50, right: 980, t0: 435, labelY: 40 }
    if (screenWidth.value > 480) return { left: 60, right: 960, t0: 425, labelY: 50 }
    return { left: 80, right: 940, t0: 415, labelY: 70 }
  })

  onMounted(() => window.addEventListener('resize', () => { screenWidth.value = window.innerWidth }))
  onUnmounted(() => window.removeEventListener('resize', () => { screenWidth.value = window.innerWidth }))

  const allAtrValues = computed(() => [...(props.atrTimelineBefore || []), ...(props.atrTimelineAfter || [])])
  const minAtr = computed(() => Math.min(...allAtrValues.value, 0))
  const maxAtr = computed(() => Math.max(...allAtrValues.value, 0.001))

  const minAtrLabel = computed(() => Math.ceil(minAtr.value / pointValue.value).toString())
  const maxAtrLabel = computed(() => Math.ceil(maxAtr.value / pointValue.value).toString())
  const midAtrLabel = computed(() => Math.ceil(((minAtr.value + maxAtr.value) / 2) / pointValue.value).toString())

  const mapAtrToY = (atr: number): number => {
    if (maxAtr.value === minAtr.value) return Y_BASELINE
    return Y_BASELINE - ((atr - minAtr.value) / (maxAtr.value - minAtr.value)) * (Y_BASELINE - Y_TOP)
  }

  // Génération des graduations Y (tous les 1 pip si possible)
  const yAxisTicks = computed(() => {
    const minPips = Math.floor(minAtr.value / pointValue.value)
    const maxPips = Math.ceil(maxAtr.value / pointValue.value)
    
    const range = maxPips - minPips
    let step = 1
    if (range > 20) step = 2
    if (range > 50) step = 5
    if (range > 100) step = 10
    
    const ticks = []
    for (let i = minPips; i <= maxPips; i += step) {
      ticks.push(i)
    }
    return ticks
  })

  const mapPipToY = (pipValue: number) => {
    return mapAtrToY(pipValue * pointValue.value)
  }

  const getTimeLabel = (offset: number): string => {
    let dateStr = props.eventDatetime || ''
    // Force UTC si pas de timezone spécifiée (format YYYY-MM-DDTHH:mm:ss venant du backend)
    if (dateStr && !dateStr.endsWith('Z') && !dateStr.includes('+')) {
      dateStr += 'Z'
    }
    const baseTime = dateStr ? new Date(dateStr).getTime() : Date.now()
    
    const d = new Date(baseTime + offset * 60000)
    const m = Math.round(d.getMinutes() / 5) * 5
    d.setMinutes(m)
    // Retourne uniquement le décalage relatif (ex: "T-5", "T+15") au lieu de l'heure absolue
    if (offset === 0) return 'T0'
    return offset > 0 ? `T+${offset}` : `T${offset}`
  }

  const getXPositionBefore = (min: number): number => {
    const m = svgMargins.value
    return m.left + ((min + 30) / 30) * (m.t0 - m.left)
  }

  const getXPositionAfter = (min: number): number => {
    const m = svgMargins.value
    return m.t0 + (min / 90) * (m.right - m.t0)
  }

  const bestMomentX = computed(() => {
    if ((props.meilleurMoment || 0) <= 0) return svgMargins.value.t0
    return getXPositionBefore(-(props.meilleurMoment || 0))
  })

  const beforePointsString = computed(() => {
    if (!props.atrTimelineBefore?.length) return ''
    const pts = props.atrTimelineBefore.map((a, i) => `${getXPositionBefore(-30 + i)},${mapAtrToY(a)}`)
    if (props.atrTimelineAfter?.length) pts.push(`${getXPositionAfter(0)},${mapAtrToY(props.atrTimelineAfter[0])}`)
    return pts.join(' ')
  })

  const afterPointsString = computed(() =>
    props.atrTimelineAfter?.map((a, i) => `${getXPositionAfter(i)},${mapAtrToY(a)}`).join(' ') || ''
  )

  const curvePathBefore = computed(() => {
    if (!props.atrTimelineBefore?.length) return ''
    const pts = props.atrTimelineBefore.map((a, i) => `${getXPositionBefore(-30 + i)},${mapAtrToY(a)}`)
    if (props.atrTimelineAfter?.length) pts.push(`${getXPositionAfter(0)},${mapAtrToY(props.atrTimelineAfter[0])}`)
    pts.push(`${getXPositionAfter(0)},${Y_BASELINE}`, `${getXPositionBefore(-30)},${Y_BASELINE}`)
    return `M${pts.join('L')}Z`
  })

  const curvePathAfter = computed(() => {
    if (!props.atrTimelineAfter?.length) return ''
    const pts = props.atrTimelineAfter.map((a, i) => `${getXPositionAfter(i)},${mapAtrToY(a)}`)
    pts.push(`${getXPositionAfter(90)},${Y_BASELINE}`, `${getXPositionAfter(0)},${Y_BASELINE}`)
    return `M${pts.join('L')}Z`
  })

  const formatValue = (val: number) => Math.ceil(val / pointValue.value)

  return {
    svgMargins,
    yAxisBaseline: Y_BASELINE,
    yMidLine: Y_MID,
    yTopLine: Y_TOP,
    minAtrLabel,
    maxAtrLabel,
    midAtrLabel,
    mapAtrToY,
    getTimeLabel,
    getXPositionBefore,
    getXPositionAfter,
    ceilValue: Math.ceil,
    formatValue,
    bestMomentX,
    beforePointsString,
    afterPointsString,
    curvePathBefore,
    curvePathAfter,
    yAxisTicks,
    mapPipToY
  }
}
