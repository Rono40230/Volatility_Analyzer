import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Archive {
    id: number
    title: string
    archive_type: string
    period_start: string
    period_end: string
    comment: string | null
    created_at: string
    data_json: string
}

export const useArchiveStore = defineStore('archive', () => {
    const archives = ref<Archive[]>([])
    const currentArchive = ref<Archive | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function sauvegarderArchive(
        title: string,
        archiveType: string,
        periodStart: string,
        periodEnd: string,
        comment: string | null,
        dataJson: string
    ): Promise<Archive> {
        loading.value = true
        error.value = null
        try {
            const archive = await invoke<Archive>('save_archive', {
                title,
                archiveType,
                periodStart,
                periodEnd,
                comment,
                dataJson
            })
            archives.value.unshift(archive)
            return archive
        } catch (e) {
            error.value = e as string
            throw e
        } finally {
            loading.value = false
        }
    }

    async function chargerArchives() {
        loading.value = true
        error.value = null
        try {
            archives.value = await invoke<Archive[]>('list_archives')
        } catch (e) {
            error.value = e as string
            throw e
        } finally {
            loading.value = false
        }
    }

    async function chargerArchive(archiveId: number) {
        loading.value = true
        error.value = null
        try {
            currentArchive.value = await invoke<Archive>('get_archive', { archiveId })
            return currentArchive.value
        } catch (e) {
            error.value = e as string
            throw e
        } finally {
            loading.value = false
        }
    }

    async function supprimerArchive(archiveId: number) {
        loading.value = true
        error.value = null
        try {
            await invoke('delete_archive', { archiveId })
            archives.value = archives.value.filter(a => a.id !== archiveId)
            if (currentArchive.value?.id === archiveId) {
                currentArchive.value = null
            }
        } catch (e) {
            error.value = e as string
            throw e
        } finally {
            loading.value = false
        }
    }

    async function supprimerToutesArchives() {
        loading.value = true
        error.value = null
        try {
            await invoke('delete_all_archives')
            archives.value = []
            currentArchive.value = null
        } catch (e) {
            error.value = e as string
            throw e
        } finally {
            loading.value = false
        }
    }

    return {
        archives,
        currentArchive,
        loading,
        error,
        sauvegarderArchive,
        chargerArchives,
        chargerArchive,
        supprimerArchive,
        supprimerToutesArchives,
        // Aliases for compatibility
        saveArchive: sauvegarderArchive,
        loadArchives: chargerArchives,
        loadArchive: chargerArchive,
        deleteArchive: supprimerArchive
    }
})
