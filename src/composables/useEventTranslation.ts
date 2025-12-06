import { eventTranslations } from '../stores/eventTranslations'

/**
 * Composable pour la traduction des noms d'événements économiques
 * Source unique : stores/eventTranslations (avec drapeaux et traductions FR)
 */

export function useEventTranslation() {

    const translateEventName = (eventName: string): string => {
        const translation = eventTranslations[eventName]
        return translation ? `${eventName} (${translation.fr}) ${translation.flag}` : eventName
    }

    return {
        translateEventName
    }
}
