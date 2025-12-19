export const formatTime = (minute: number, hour?: number, quarter?: number) => {
  if (hour === undefined || quarter === undefined) return `${minute}m`
  
  const startMin = quarter * 15
  let totalMin = startMin + minute
  let finalH = hour
  
  while (totalMin >= 60) {
    totalMin -= 60
    finalH = (finalH + 1) % 24
  }
  
  while (totalMin < 0) {
    totalMin += 60
    finalH = (finalH - 1 + 24) % 24
  }
  
  const mm = totalMin.toString().padStart(2, '0')
  const hh = finalH.toString().padStart(2, '0')
  return `${hh}:${mm}`
}

export const getEventMinute = (timeStr: string, hour?: number, quarter?: number) => {
  if (hour === undefined || quarter === undefined) return 0
  
  const [h, m] = timeStr.split(':').map(Number)
  const eventTotalMin = h * 60 + m
  
  const startMin = quarter * 15
  const quarterStartTotalMin = hour * 60 + startMin
  
  // Handle day wrap
  let diff = eventTotalMin - quarterStartTotalMin
  if (diff < -720) diff += 1440 // Event next day
  if (diff > 720) diff -= 1440 // Event prev day
  
  return diff
}

export const getEventColor = (impact: string) => {
  switch (impact.toLowerCase()) {
    case 'high': return '#ef4444' // red-500
    case 'medium': return '#f97316' // orange-500
    case 'low': return '#eab308' // yellow-500
    default: return '#94a3b8' // slate-400
  }
}

export const getX = (minute: number, minMinute: number, totalRange: number) => {
  const width = 340 // 380 - 40
  const offset = minute - minMinute
  // Ensure we don't divide by zero, though totalRange should be > 0
  const range = totalRange || 1
  return 40 + (offset / range) * width
}

export const getY = (val: number, maxValue: number) => {
  const height = 160 // 180 - 20
  if (maxValue === 0) return 180
  return 180 - (val / maxValue) * height
}
