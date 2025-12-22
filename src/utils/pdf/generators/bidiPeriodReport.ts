import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import { formaterPointsAvecPips, convertirPointsEnPips, obtenirPointsParPip } from '../../pipConverter'

function formatEntryTime(hour: number, quarter: number, delayMinutes: number = 0): string {
  const startMin = quarter * 15
  const totalMin = startMin + delayMinutes
  
  const finalHour = (hour + Math.floor(totalMin / 60)) % 24
  const finalMin = totalMin % 60
  
  return `${String(finalHour).padStart(2, '0')}:${String(finalMin).padStart(2, '0')}`
}

interface JsPDFWithAutoTable extends jsPDF {
  lastAutoTable: {
    finalY: number
  }
}

export async function generateBidiPeriodReport(doc: jsPDF, dataList: any[], startY: number = 20) {
  doc.setFontSize(16)
  doc.text('Fiche Bidi : Paire/Période', 14, startY)
  doc.setFontSize(10)
  doc.text('Volatilité Brute - Session Trading (Source: Archives)', 14, startY + 6)
  
  const rowsDirectional: any[] = []
  const rowsSimultaneous: any[] = []

  dataList.forEach(data => {
    // Structure attendue: ArchivedAnalysisData
    const plan = data.tradingPlan
    const slice = data.sliceAnalyses && data.sliceAnalyses.length > 0 ? data.sliceAnalyses[0].slice : null
    const symbol = data.analysisResult ? data.analysisResult.symbol : (data.symbol || 'Inconnu')
    const pointsPerPip = obtenirPointsParPip(symbol)
    
    // Récupération des paramètres avec fallback (Priorité: Rust > Frontend Plan > Fallback)
    const params = slice?.stats?.straddle_parameters
    
    // Offset
    let offset = 0
    if (params?.offset_pips) {
      offset = params.offset_pips
    } else if (data.offsetOptimal?.offset_points) {
      offset = convertirPointsEnPips(symbol, data.offsetOptimal.offset_points)
    } else if (plan?.offset) {
      offset = plan.offset
    }

    // SL (Directionnel)
    let sl = 0
    if (params?.stop_loss_pips) {
      sl = params.stop_loss_pips
    } else if (plan?.slPips) {
      sl = plan.slPips
    } else if (plan?.sl) {
      sl = plan.sl
    }

    // SL Recovery (Simultané)
    let slRecovery = 0
    if (params?.sl_recovery_pips) {
      slRecovery = params.sl_recovery_pips
    } else {
      slRecovery = sl * 1.5 // Fallback standard
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
      // Directionnel
      rowsDirectional.push([
        symbol,
        slice.startTime || 'N/A',
        entryTime,
        offset ? formaterPointsAvecPips(symbol, offset) : 'N/A',
        sl ? formaterPointsAvecPips(symbol, sl) : 'N/A',
        ts ? formaterPointsAvecPips(symbol, ts) : 'N/A',
        duration ? `${duration}m` : 'N/A'
      ])

      // Simultané
      rowsSimultaneous.push([
        symbol,
        slice.startTime || 'N/A',
        entryTime,
        slRecovery ? formaterPointsAvecPips(symbol, slRecovery) : 'N/A',
        ts ? formaterPointsAvecPips(symbol, ts) : 'N/A',
        duration ? `${duration}m` : 'N/A'
      ])
    } else {
      // Fallback
      rowsDirectional.push([symbol, 'N/A', 'N/A', 'N/A', 'N/A', 'N/A', 'N/A'])
      rowsSimultaneous.push([symbol, 'N/A', 'N/A', 'N/A', 'N/A', 'N/A'])
    }
  })

  // Table 1: Directionnel
  doc.setFontSize(12)
  doc.setTextColor(41, 128, 185)
  doc.text('Stratégie Directionnelle (Buy Stop / Sell Stop)', 14, startY + 15)
  
  autoTable(doc, {
    startY: startY + 18,
    head: [['Paire', 'Période', 'Entrée', 'Offset', 'SL', 'T.Stop', 'Durée']],
    body: rowsDirectional,
    theme: 'grid',
    headStyles: { fillColor: [41, 128, 185] },
    styles: { fontSize: 9, cellPadding: 3 },
    columnStyles: {
      0: { fontStyle: 'bold' },
      3: { halign: 'right' },
      4: { halign: 'right' },
      5: { halign: 'right' },
      6: { halign: 'center' }
    }
  })

  // Table 2: Simultané
  const finalY = (doc as unknown as JsPDFWithAutoTable).lastAutoTable.finalY + 15
  
  // Check if new page needed
  if (finalY > 250) {
    doc.addPage()
    doc.setFontSize(12)
    doc.setTextColor(142, 68, 173)
    doc.text('Stratégie Simultanée (Straddle)', 14, 20)
    
    autoTable(doc, {
      startY: 25,
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
  } else {
    doc.setFontSize(12)
    doc.setTextColor(142, 68, 173)
    doc.text('Stratégie Simultanée (Straddle)', 14, finalY - 3)
    
    autoTable(doc, {
      startY: finalY,
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
}
