import { ref } from 'vue'
import { jsPDF } from 'jspdf'
import { recupererDonneesArchivees, recupererDonneesBacktestArchivees } from '../utils/pdf/dataFetcher'
import { 
  generateBidiPeriodReport,
  generateBidiEventReport,
  generateRankingReport
} from '../utils/pdf/reportGenerators'
import { generateBacktestReport } from '../utils/pdf/backtestReportGenerator'

export interface ExportFilters {
  periodStart: string
  periodEnd: string
  pairs: string[]
}

export function usePdfExport() {
  const isGenerating = ref(false)
  const progress = ref(0)
  const error = ref<string | null>(null)

  async function generatePdf(reportTypes: string[], filters: ExportFilters, preview = false) {
    isGenerating.value = true
    progress.value = 0
    error.value = null

    try {
      const doc = new jsPDF()
      let pageAdded = false
      
      // Titre global
      doc.setFontSize(20)
      doc.text('Rapport d\'Analyse Historique', 14, 22)
      doc.setFontSize(10)
      doc.text(`Généré le ${new Date().toLocaleDateString('fr-FR')}`, 14, 30)
      doc.text('Source: Archives validées', 14, 36)
      
      // Filtres appliqués
      doc.text(`Période: ${filters.periodStart} au ${filters.periodEnd}`, 14, 42)
      doc.text(`Paires analysées: ${filters.pairs.length}`, 14, 48)

      let yPos = 60
      const totalReports = reportTypes.length
      let currentReport = 0

      // 1. Récupération des données
      let periodDataList: any[] = []
      let eventDataList: any[] = []
      
      // Récupération Volatilité Brute (Période)
      if (reportTypes.includes('bidi_period') || reportTypes.includes('ranking')) {
        periodDataList = await recupererDonneesArchivees(filters.pairs, (p) => {
          progress.value = p * 0.25
        }, 'Volatilité brute Paire/Période')
      }

      // Récupération Corrélation (Événements)
      if (reportTypes.includes('bidi_event')) {
        eventDataList = await recupererDonneesArchivees(filters.pairs, (p) => {
          progress.value = 25 + (p * 0.25)
        }, 'Correlation de la volatilité Paire/Evenement')
      }

      // Récupération Backtest
      let backtestDataList: any[] = []
      if (reportTypes.includes('backtest')) {
        backtestDataList = await recupererDonneesBacktestArchivees(filters.pairs, (p) => {
           progress.value = 50 + (p * 0.25)
        })
      }

      // Vérification des données
      if (periodDataList.length === 0 && eventDataList.length === 0 && backtestDataList.length === 0) {
        throw new Error("Aucune archive trouvée pour les rapports sélectionnés.")
      }

      // 3. Génération des rapports
      for (const type of reportTypes) {
        if (pageAdded) {
          doc.addPage()
          yPos = 20
        }
        
        switch (type) {
          case 'bidi_period':
            if (periodDataList.length > 0) await generateBidiPeriodReport(doc, periodDataList, yPos)
            break
          case 'bidi_event':
            if (eventDataList.length > 0) await generateBidiEventReport(doc, eventDataList, yPos)
            break
          case 'ranking':
            if (periodDataList.length > 0) await generateRankingReport(doc, periodDataList, yPos)
            break
          case 'backtest':
            if (backtestDataList.length > 0) await generateBacktestReport(doc, backtestDataList, yPos)
            break
        }
        
        pageAdded = true
        currentReport++
        // Mise à jour du progrès restant
        progress.value = 75 + ((currentReport / totalReports) * 25)
      }

      if (preview) {
        // Utiliser datauristring au lieu de bloburl pour éviter les erreurs de sécurité WebKit/Tauri
        return doc.output('datauristring')
      } else {
        doc.save(`analyse_export_${new Date().toISOString().split('T')[0]}.pdf`)
        return true
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    } finally {
      isGenerating.value = false
    }
  }

  return {
    generatePdf,
    isGenerating,
    progress,
    error
  }
}
