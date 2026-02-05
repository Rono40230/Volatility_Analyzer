import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import { formaterPointsAvecPips, obtenirPointsParPip } from '../../pipConverter'

function formatEntryTime(hour: number, quarter: number, delayMinutes: number = 0): string {
  const startMin = quarter * 15
  const totalMin = startMin + delayMinutes
  
  const finalHour = (hour + Math.floor(totalMin / 60)) % 24
  const finalMin = totalMin % 60
  
  return `${String(finalHour).padStart(2, '0')}:${String(finalMin).padStart(2, '0')}`
}

export async function generateBidiPeriodReport(doc: jsPDF, dataList: any[], startY: number = 20) {
  doc.setFontSize(16)
  doc.text('Fiche Straddle Simultané : Paire/Période', 14, startY)
  doc.setFontSize(10)
  doc.text('Volatilité Brute - Session Trading (Source: Archives)', 14, startY + 6)
  
  const rowsSimultaneous: any[] = []

  dataList.forEach(data => {
    // Structure attendue: ArchivedAnalysisData
    const plan = data.tradingPlan
    const slice = data.sliceAnalyses && data.sliceAnalyses.length > 0 ? data.sliceAnalyses[0].slice : null
    const symbol = data.analysisResult ? data.analysisResult.symbol : (data.symbol || 'Inconnu')
    const pointsPerPip = obtenirPointsParPip(symbol)
    
    // Récupération des paramètres avec fallback (Priorité: Rust > Frontend Plan > Fallback)
    const params = slice?.stats?.straddle_parameters
    
    // SL Recovery (Simultané)
    let slRecovery = 0
    if (params?.sl_recovery_pips) {
      slRecovery = params.sl_recovery_pips
    } else {
      slRecovery = 0
    }

    // Trailing Stop
    let ts = 0
    if (params?.trailing_stop_pips) {
      ts = params.trailing_stop_pips
    } else if (data.whipsawAnalysis?.trailing_stop_adjusted) {
      ts = data.whipsawAnalysis.trailing_stop_adjusted / pointsPerPip
    } else if (plan?.trailingStopCoefficient && plan?.atrPoints) {
      // Estimation approximative si manquant
      ts = (plan.atrPoints * plan.trailingStopCoefficient) / pointsPerPip
    }

    // Duration
    let duration = 0
    if (params?.timeout_minutes) {
      duration = params.timeout_minutes
    } else if (plan?.tradeDurationMinutes) {
      duration = plan.tradeDurationMinutes
    } else if (plan?.duration) {
      duration = plan.duration
    }

    // Entry Time
    let entryTime = 'N/A'
    if (slice) {
      const delay = data.entryWindowAnalysis?.optimal_entry_minutes ?? 0
      entryTime = formatEntryTime(slice.hour, slice.quarter, delay)
    }

    if (slice) {
      rowsSimultaneous.push([
        symbol,
        slice.startTime || 'N/A',
        entryTime,
        slRecovery ? formaterPointsAvecPips(symbol, slRecovery) : 'N/A',
        ts ? formaterPointsAvecPips(symbol, ts) : 'N/A',
        duration ? `${duration}m` : 'N/A'
      ])
    } else {
      rowsSimultaneous.push([symbol, 'N/A', 'N/A', 'N/A', 'N/A', 'N/A'])
    }
  })

  doc.setFontSize(12)
  doc.setTextColor(142, 68, 173)
  doc.text('Stratégie Simultanée (Straddle)', 14, startY + 15)

  autoTable(doc, {
    startY: startY + 18,
    head: [['Paire', 'Période', 'Entrée', 'SL Rec.', 'T.Stop', 'Durée']],
    body: rowsSimultaneous,
    theme: 'grid',
    headStyles: { fillColor: [142, 68, 173] },
    styles: { fontSize: 9, cellPadding: 3 },
    columnStyles: {
      0: { fontStyle: 'bold' },
      3: { halign: 'right' },
      4: { halign: 'right' },
      5: { halign: 'center' }
    }
  })
}
