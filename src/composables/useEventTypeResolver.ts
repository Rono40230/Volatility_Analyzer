// Composable pour résoudre (fuzzy match) un nom d'événement vers un type exact
import type { Ref } from 'vue'

interface EventTypeItem {
  name: string
  count: number
}

/**
 * Fournit resolveEventType() pour trouver un match exact ou fuzzy
 * parmi les types d'événements disponibles.
 */
export function useEventTypeResolver(
  eventTypes: Ref<EventTypeItem[]>,
  getEventLabel: (name: string) => string
) {
  function normalizeEventName(value: string): string {
    return value
      .toLowerCase()
      .replace(/\(.*?\)/g, '')
      .replace(/[\p{Emoji_Presentation}\p{Emoji}\uFE0F]/gu, '')
      .replace(/[^a-z0-9]+/g, ' ')
      .trim()
  }

  function scoreEventMatch(a: string, b: string): number {
    const aTokens = normalizeEventName(a).split(' ').filter(Boolean)
    const bTokens = normalizeEventName(b).split(' ').filter(Boolean)
    if (!aTokens.length || !bTokens.length) return 0
    const bSet = new Set(bTokens)
    const common = aTokens.filter(t => bSet.has(t)).length
    return common / Math.max(aTokens.length, bTokens.length)
  }

  function resolveEventType(input: string): string | null {
    if (!input) return null
    const direct = eventTypes.value.find(et => et.name === input)
    if (direct) return direct.name
    const byLabel = eventTypes.value.find(et => getEventLabel(et.name) === input)
    if (byLabel) return byLabel.name

    const normalizedInput = normalizeEventName(input)
    const normalizedMatch = eventTypes.value.find(et => normalizeEventName(et.name) === normalizedInput)
    if (normalizedMatch) return normalizedMatch.name
    const normalizedLabelMatch = eventTypes.value.find(et => normalizeEventName(getEventLabel(et.name)) === normalizedInput)
    if (normalizedLabelMatch) return normalizedLabelMatch.name

    let best: { name: string; score: number } | null = null
    for (const et of eventTypes.value) {
      const score = scoreEventMatch(input, et.name)
      if (!best || score > best.score) best = { name: et.name, score }
    }
    if (best && best.score >= 0.6) return best.name
    return null
  }

  return { resolveEventType, normalizeEventName, scoreEventMatch }
}
