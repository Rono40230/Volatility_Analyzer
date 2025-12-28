import { eventTranslations } from './eventTranslations'

export function formatEventLabel(name: string): string {
  const translation = eventTranslations[name]
  if (translation) {
    return `${name} (${translation.fr}) ${translation.flag}`
  }
  return name
}
