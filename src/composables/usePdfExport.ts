import { ref } from 'vue'
import { jsPDF } from 'jspdf'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { readFile } from '@tauri-apps/plugin-fs'
import { recupererDonneesArchivees, recupererDonneesBacktestArchivees } from '../utils/pdf/dataFetcher'
import {
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
      // Cas spécial: si SEULEMENT "comparative" est sélectionné, appeler directement la fonction Rust
      if (reportTypes.length === 1 && reportTypes.includes('comparative')) {
        return await generateComparativeReport(preview)
      }

      // Cas spécial: si SEULEMENT "global_analysis" est sélectionné
      if (reportTypes.length === 1 && reportTypes.includes('global_analysis')) {
        const doc = new jsPDF()
        await generateGlobalAnalysisReport(doc, 20, filters)
        
        if (preview) {
          return doc.output('datauristring')
        } else {
          doc.save(`analyse_globale_${new Date().toISOString().split('T')[0]}.pdf`)
          return true
        }
      }

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
      
      // Vérifier si nous avons des rapports "non-meta" et non-global
      const hasOtherReports = reportTypes.some(t => !t.startsWith('meta_') && t !== 'global_analysis')

      // Récupération Volatilité Brute (Période)
      if (reportTypes.includes('ranking')) {
        periodDataList = await recupererDonneesArchivees(filters.pairs, (p) => {
          progress.value = p * 0.25
        }, 'Volatilité brute Paire/Période')
      }

      // Récupération Corrélation (Événements)
      if (reportTypes.includes('correlation_event')) {
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

      // Vérification des données - SEULEMENT pour les rapports non-global
      if (hasOtherReports && periodDataList.length === 0 && eventDataList.length === 0 && backtestDataList.length === 0) {
        throw new Error("Aucune archive trouvée pour les rapports sélectionnés.")
      }

      // 3. Génération des rapports
      for (const type of reportTypes) {
        if (pageAdded) {
          doc.addPage()
          yPos = 20
        }
        
        switch (type) {
          case 'ranking':
            if (periodDataList.length > 0) await generateRankingReport(doc, periodDataList, yPos)
            break
          case 'backtest':
            if (backtestDataList.length > 0) await generateBacktestReport(doc, backtestDataList, yPos)
            break
          case 'global_analysis':
            await generateGlobalAnalysisReport(doc, yPos, filters)
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

  async function generateGlobalAnalysisReport(doc: jsPDF, yPos: number, filters: ExportFilters) {
    try {
      const archives = await invoke<any[]>('list_archives')
      
      // ===== 1. RÉCUPÉRATION HEATMAP =====
      const heatmapArchives = archives
        .filter(a => a.archive_type === 'Heatmap' || a.archive_type === 'HEATMAP')
        .map(a => {
          try {
            const parsed = JSON.parse(a.data_json)
            const rootData = parsed
            const heatmapData = rootData.heatmapData || rootData
            return heatmapData.data || heatmapData
          } catch { return null }
        })
        .filter(Boolean)

      const heatmapPairsAcc: Record<string, { sum: number, count: number }> = {}
      const heatmapEventsAcc: Record<string, { sum: number, count: number }> = {}

      for (const dataMap of heatmapArchives) {
        if (!dataMap || typeof dataMap !== 'object') continue
        
        for (const [eventName, pairMap] of Object.entries(dataMap)) {
          if (!pairMap || typeof pairMap !== 'object') continue
          
          const values = Object.values(pairMap)
          if (values.length === 0 || values.some(v => typeof v !== 'number')) continue
          
          for (const [pairName, score] of Object.entries(pairMap)) {
            if (typeof score !== 'number') continue
            if (!heatmapPairsAcc[pairName]) heatmapPairsAcc[pairName] = { sum: 0, count: 0 }
            heatmapPairsAcc[pairName].sum += score
            heatmapPairsAcc[pairName].count++
          }
          
          for (const score of Object.values(pairMap)) {
            if (typeof score === 'number') {
              if (!heatmapEventsAcc[eventName]) heatmapEventsAcc[eventName] = { sum: 0, count: 0 }
              heatmapEventsAcc[eventName].sum += score
              heatmapEventsAcc[eventName].count++
            }
          }
        }
      }

      // ===== 2. RÉCUPÉRATION CORRÉLATION VOLATILITÉ =====
      const corrVolArchives = archives
        .filter(a => a.archive_type === 'Correlation de la volatilité Paire/Evenement')
        .map(a => {
          try { return JSON.parse(a.data_json) } catch { return null }
        })
        .filter(Boolean)

      const eventStats: Record<string, { sumImpact: number, count: number }> = {}
      const pairVolStats: Record<string, { sumImpact: number, count: number }> = {}

      for (const item of corrVolArchives) {
        if (!item) continue
        const eventLabel = item.eventLabel || 'Unknown'
        const pair = item.pair || 'Unknown'
        const impact = item.volatilityIncreasePercent || 0

        if (!eventStats[eventLabel]) eventStats[eventLabel] = { sumImpact: 0, count: 0 }
        eventStats[eventLabel].sumImpact += impact
        eventStats[eventLabel].count++

        if (!pairVolStats[pair]) pairVolStats[pair] = { sumImpact: 0, count: 0 }
        pairVolStats[pair].sumImpact += impact
        pairVolStats[pair].count++
      }

      // ===== 3. RÉCUPÉRATION POINTS D'ENTRÉE =====
      const entryArchives = archives
        .filter(a => a.archive_type === "Analyse Point d'Entrée")
        .map(a => {
          try {
            const raw = JSON.parse(a.data_json)
            const entryAnalysis = raw.entryAnalysis
            if (!entryAnalysis) return null
            return {
              id: a.id,
              date: a.created_at,
              symbol: raw.symbol || entryAnalysis.symbol || 'Unknown',
              pair: a.pair || raw.symbol || entryAnalysis.symbol || 'Unknown',
              hour: raw.hour,
              quarter: raw.quarter,
              entryAnalysis: entryAnalysis
            }
          } catch { return null }
        })
        .filter(Boolean)

      const pairEntryStats: Record<string, {
        analyses: any[]
        avgPeakMinute: number
        avgDuration: number
        maxMovement: number
        avgConsistency: number
      }> = {}

      for (const a of entryArchives) {
        const pair = a.pair
        if (!pairEntryStats[pair]) {
          pairEntryStats[pair] = {
            analyses: [],
            avgPeakMinute: 0,
            avgDuration: 0,
            maxMovement: 0,
            avgConsistency: 0
          }
        }
        pairEntryStats[pair].analyses.push(a)
      }

      for (const pair in pairEntryStats) {
        const analyses = pairEntryStats[pair].analyses
        pairEntryStats[pair].avgPeakMinute = Math.round(
          analyses.reduce((sum, a) => sum + (a.entryAnalysis?.peak_minute || 0), 0) / analyses.length
        )
        pairEntryStats[pair].avgDuration = Math.round(
          analyses.reduce((sum, a) => sum + (a.entryAnalysis?.movement_duration_minutes || 0), 0) / analyses.length
        )
        pairEntryStats[pair].maxMovement = Math.max(
          ...analyses.map(a => a.entryAnalysis?.avg_movement_pips || 0)
        )
        const avgWinRate = analyses.reduce((sum, a) => sum + (a.entryAnalysis?.real_win_rate || 0), 0) / analyses.length
        pairEntryStats[pair].avgConsistency = Math.round(avgWinRate > 1 ? avgWinRate : avgWinRate * 100)
      }

      // ===== CONSTRUCTION DU RAPPORT =====
      doc.setFontSize(16)
      doc.text('Rapport Global d\'Analyse', 14, yPos)
      
      yPos += 8
      doc.setFontSize(9)
      doc.text(`Généré le ${new Date().toLocaleDateString('fr-FR')} | Période: ${filters.periodStart.split('T')[0]} au ${filters.periodEnd.split('T')[0]}`, 14, yPos)
      yPos += 6
      doc.text(`Paires analysées: ${filters.pairs.join(', ')}`, 14, yPos)

      // ===== SECTION 1: HEATMAP =====
      if (heatmapArchives.length > 0) {
        yPos += 12
        doc.setFontSize(13)
        doc.setTextColor(70, 130, 180)
        doc.text('1. ANALYSE HEATMAP - Corrélations de Paires', 14, yPos)
        doc.setTextColor(0, 0, 0)
        yPos += 7

        doc.setFontSize(10)
        doc.text(`Archives heatmap: ${heatmapArchives.length}`, 14, yPos)
        yPos += 6

        const topHeatmapPairs = Object.entries(heatmapPairsAcc)
          .map(([pair, stats]) => ({ pair, score: Math.round(stats.sum / stats.count) }))
          .sort((a, b) => b.score - a.score)
          .slice(0, 5)

        const topHeatmapEvents = Object.entries(heatmapEventsAcc)
          .map(([name, stats]) => ({ 
            name: name.replace(/[^\x20-\x7E]/g, '').trim() || name,
            impact_score: parseFloat((stats.sum / stats.count / 10).toFixed(1)) 
          }))
          .sort((a, b) => b.impact_score - a.impact_score)
          .slice(0, 5)

        doc.setFontSize(9)
        doc.text('Paires Corrélées Principales:', 16, yPos)
        yPos += 6
        topHeatmapPairs.forEach((p, i) => {
          doc.text(`  ${i + 1}. ${p.pair}: Score ${p.score}`, 18, yPos)
          yPos += 5
        })

        yPos += 4
        doc.text('Événements Impactants:', 16, yPos)
        yPos += 6
        topHeatmapEvents.forEach((e, i) => {
          const eventName = e.name.length > 50 ? e.name.substring(0, 47) + '...' : e.name
          doc.text(`  ${i + 1}. ${eventName}: Score ${e.impact_score}`, 18, yPos)
          yPos += 5
        })
      }

      // ===== SECTION 2: VOLATILITÉ & ÉVÉNEMENTS =====
      if (corrVolArchives.length > 0) {
        yPos += 8
        doc.setFontSize(13)
        doc.setTextColor(70, 130, 180)
        doc.text('2. ANALYSE VOLATILITÉ - Événements Impactants', 14, yPos)
        doc.setTextColor(0, 0, 0)
        yPos += 7

        doc.setFontSize(10)
        doc.text(`Archives corrélation volatilité: ${corrVolArchives.length}`, 14, yPos)
        yPos += 6

        const topEvents = Object.entries(eventStats)
          .map(([event, stats]) => ({ 
            event: event.replace(/[^\x20-\x7E]/g, '').trim() || event,
            avgImpact: parseFloat((stats.sumImpact / stats.count).toFixed(2)) 
          }))
          .sort((a, b) => b.avgImpact - a.avgImpact)
          .slice(0, 5)

        doc.setFontSize(9)
        doc.text('Top 5 Événements par Impact:', 16, yPos)
        yPos += 6
        topEvents.forEach((e, i) => {
          const eventName = e.event.length > 50 ? e.event.substring(0, 47) + '...' : e.event
          doc.text(`  ${i + 1}. ${eventName}`, 18, yPos)
          doc.text(`Impact: +${e.avgImpact}%`, 120, yPos)
          yPos += 5
        })

        // Paires affectées
        yPos += 5
        const topAffectedPairs = Object.entries(pairVolStats)
          .map(([pair, stats]) => ({ pair, avgImpact: parseFloat((stats.sumImpact / stats.count).toFixed(2)) }))
          .sort((a, b) => b.avgImpact - a.avgImpact)
          .slice(0, 5)

        doc.setFontSize(9)
        doc.text('Top 5 Paires Affectées:', 16, yPos)
        yPos += 6
        topAffectedPairs.forEach((p, i) => {
          doc.text(`  ${i + 1}. ${p.pair}: Impact +${p.avgImpact}%`, 18, yPos)
          yPos += 5
        })
      }

      // ===== SECTION 3: POINTS D'ENTRÉE =====
      if (Object.keys(pairEntryStats).length > 0) {
        yPos += 8
        doc.setFontSize(13)
        doc.setTextColor(70, 130, 180)
        doc.text('3. POINTS D\'ENTRÉE - Analyse par Paire', 14, yPos)
        doc.setTextColor(0, 0, 0)
        yPos += 7

        doc.setFontSize(10)
        doc.text(`Archives points d'entrée: ${entryArchives.length} analyses | ${Object.keys(pairEntryStats).length} paires`, 14, yPos)
        yPos += 8

        const topEntryPairs = Object.entries(pairEntryStats)
          .map(([pair, stats]) => ({
            pair,
            ...stats
          }))
          .sort((a, b) => b.avgConsistency - a.avgConsistency)
          .slice(0, 10)

        doc.setFontSize(8)
        const headers = ['Paire', 'Analyses', 'Pic(min)', 'Durée(min)', 'Mvmt(pips)', 'Win Rate']
        const colX = [14, 45, 65, 85, 110, 140]
        
        doc.setTextColor(70, 130, 180)
        headers.forEach((h, i) => doc.text(h, colX[i], yPos))
        yPos += 5
        doc.setTextColor(0, 0, 0)

        topEntryPairs.forEach(p => {
          doc.text(p.pair, colX[0], yPos)
          doc.text(p.analyses.length.toString(), colX[1], yPos)
          doc.text(Math.round(p.avgPeakMinute || 0).toString(), colX[2], yPos)
          doc.text(Math.round(p.avgDuration || 0).toString(), colX[3], yPos)
          doc.text(p.maxMovement.toFixed(1), colX[4], yPos)
          doc.text(`${p.avgConsistency}%`, colX[5], yPos)
          yPos += 5
          
          if (yPos > 270) {
            doc.addPage()
            yPos = 20
          }
        })
      }

      // ===== SECTION 4: CONCLUSION =====
      yPos += 12
      doc.setFontSize(13)
      doc.setTextColor(70, 130, 180)
      doc.text('4. CONCLUSION - Points Forts de l\'Analyse', 14, yPos)
      doc.setTextColor(0, 0, 0)
      yPos += 8

      doc.setFontSize(9)
      const conclusions: string[] = []

      // Point fort 1: Heatmap
      if (heatmapArchives.length > 0) {
        const dominantPair = Object.entries(heatmapPairsAcc).length > 0 
          ? Object.entries(heatmapPairsAcc).sort((a, b) => (b[1].sum / b[1].count) - (a[1].sum / a[1].count))[0][0]
          : null
        if (dominantPair) {
          conclusions.push(`• La paire dominante ${dominantPair} montre des corrélations importantes avec d'autres paires.`)
        }
      }

      // Point fort 2: Volatilité
      if (corrVolArchives.length > 0) {
        const topEvent = Object.entries(eventStats).length > 0
          ? Object.entries(eventStats).sort((a, b) => (b[1].sumImpact / b[1].count) - (a[1].sumImpact / a[1].count))[0]
          : null
        if (topEvent) {
          const impact = (topEvent[1].sumImpact / topEvent[1].count).toFixed(2)
          const cleanEventName = topEvent[0].replace(/[^\x20-\x7E]/g, '').trim() || topEvent[0]
          conclusions.push(`• L'événement "${cleanEventName}" a un impact volatilité de +${impact}% en moyenne.`)
        }
      }

      // Point fort 3: Points d'entrée
      if (Object.keys(pairEntryStats).length > 0) {
        const bestPair = Object.entries(pairEntryStats)
          .sort((a, b) => b[1].avgConsistency - a[1].avgConsistency)[0]
        if (bestPair && bestPair[1].avgConsistency > 0) {
          conclusions.push(`• La paire ${bestPair[0]} affiche le meilleur taux de réussite avec ${bestPair[1].avgConsistency}% de win rate.`)
        }
      }

      // Point fort global
      const totalArchives = heatmapArchives.length + corrVolArchives.length + entryArchives.length
      conclusions.push(`• Analyse basée sur ${totalArchives} archives validées couvrant ${Object.keys(pairEntryStats).length} paires.`)

      conclusions.forEach((conclusion, idx) => {
        doc.text(conclusion, 16, yPos)
        yPos += 7
        if (yPos > 270) {
          doc.addPage()
          yPos = 20
        }
      })

    } catch (e) {
      console.error("Erreur génération rapport global:", e)
    }
  }

  async function generateComparativeReport(preview: boolean) {
    try {
      progress.value = 50
      
      let outputPath = "TEMP_PREVIEW"
      
      if (!preview) {
         try {
             // Utiliser la boîte de dialogue native pour choisir où sauvegarder
             const selectedPath = await invoke<string>('plugin:dialog|save', {
                 defaultPath: `analyse_comparative_${new Date().toISOString().split('T')[0]}.pdf`,
                 filters: [{ name: 'PDF', extensions: ['pdf'] }]
             });
             if (selectedPath) {
                 outputPath = selectedPath;
             } else {
                 return false; // Annulé par l'utilisateur
             }
         } catch (e) {
             console.warn("Save dialog failed", e);
             // Continue with temp path if dialog fails, user will just see preview-like behavior maybe?
         }
      }
      
      const resultPath = await invoke<string>('export_comparative_analysis_pdf', {
        outputPath: outputPath
      })
      
      progress.value = 100
      
      if (preview) {
         try {
             // Lecture binaire directe du fichier
             const fileData = await readFile(resultPath);
             const blob = new Blob([fileData], { type: 'application/pdf' });
             return URL.createObjectURL(blob);
         } catch (e) {
             console.error("Erreur lecture fichier PDF:", e);
             // Fallback
             return convertFileSrc(resultPath);
         }
      } else {
        return true
      }
    } catch (e) {
      throw new Error(`Erreur génération PDF comparatif: ${e}`)
    }
  }

  return {
    generatePdf,
    isGenerating,
    progress,
    error
  }
}
