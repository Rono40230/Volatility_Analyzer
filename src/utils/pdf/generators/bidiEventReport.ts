import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import { formaterPointsAvecPips, convertirPointsEnPips } from '../../pipConverter'

interface JsPDFWithAutoTable extends jsPDF {
  lastAutoTable: {
    finalY: number
  }
}

function calculateEntryTime(eventDatetime: string | undefined, meilleurMomentMinutes: number | undefined): string {
  if (!eventDatetime) return 'N/A'
  
  // Force UTC si pas de timezone (format backend)
  let dateStr = eventDatetime
  if (dateStr && !dateStr.endsWith('Z') && !dateStr.includes('+')) {
    dateStr += 'Z'
  }
  
  const baseTime = new Date(dateStr).getTime()
  // meilleurMoment est en minutes AVANT l'événement (ex: 5 pour T-5)
  // On soustrait ces minutes
  const offsetMinutes = -(meilleurMomentMinutes || 0)
  
  const d = new Date(baseTime + offsetMinutes * 60000)
  
  // Arrondi aux 5 minutes les plus proches (comme sur le graphique)
  const m = Math.round(d.getMinutes() / 5) * 5
  d.setMinutes(m)
  d.setSeconds(0)
  
  return d.toLocaleTimeString('fr-FR', { 
    hour: '2-digit', 
    minute: '2-digit', 
    hour12: false, 
    timeZone: 'Europe/Paris' 
  })
}

export async function generateBidiEventReport(doc: jsPDF, dataList: any[], startY: number = 20) {
  doc.setFontSize(16)
  doc.text('Fiche Bidi : Paire/Événements', 14, startY)
  doc.setFontSize(10)
  doc.text('Corrélation - News Trading (Source: Archives)', 14, startY + 6)
  
  const rowsDirectional: any[] = []
  const rowsSimultaneous: any[] = []

  dataList.forEach(data => {
    // Structure attendue: RetroAnalysisResult (Archive "Corrélation de la volatilité")
    const symbol = data.pair || data.symbol || 'Inconnu'
    const event = data.eventType || data.event_type || 'N/A'
    
    // Les données d'archives sont généralement en POINTS
    // On doit les convertir en PIPS pour formaterPointsAvecPips
    
    // Offset
    let offsetPips = 0
    if (data.offset) {
      offsetPips = convertirPointsEnPips(symbol, data.offset)
    } else if (data.optimalOffset) {
      offsetPips = convertirPointsEnPips(symbol, data.optimalOffset)
    }

    // SL
    let slPips = 0
    if (data.stopLoss) {
      slPips = convertirPointsEnPips(symbol, data.stopLoss)
    } else if (data.sl) {
      slPips = convertirPointsEnPips(symbol, data.sl)
    }

    // SL Recovery
    let slRecoveryPips = 0
    if (data.stopLossRecovery) {
      slRecoveryPips = convertirPointsEnPips(symbol, data.stopLossRecovery)
    } else {
      slRecoveryPips = slPips * 1.5 // Fallback
    }

    // Trailing Stop
    let tsPips = 0
    if (data.trailingStop) {
      tsPips = convertirPointsEnPips(symbol, data.trailingStop)
    }

    // Duration
    let duration = 0
    if (data.timeout) {
      duration = data.timeout
    } else if (data.duration) {
      duration = data.duration
    }

    // Entrée (Heure exacte calculée)
    const entryTime = calculateEntryTime(data.eventDatetime, data.meilleurMoment)

    // Directionnel
    rowsDirectional.push([
      symbol,
      event,
      entryTime,
      offsetPips ? formaterPointsAvecPips(symbol, offsetPips) : 'N/A',
      slPips ? formaterPointsAvecPips(symbol, slPips) : 'N/A',
      tsPips ? formaterPointsAvecPips(symbol, tsPips) : 'N/A',
      duration ? `${duration}m` : 'N/A'
    ])

    // Simultané (Sans Offset)
    rowsSimultaneous.push([
      symbol,
      event,
      entryTime,
      slRecoveryPips ? formaterPointsAvecPips(symbol, slRecoveryPips) : 'N/A',
      tsPips ? formaterPointsAvecPips(symbol, tsPips) : 'N/A',
      duration ? `${duration}m` : 'N/A'
    ])
  })

  // Table 1: Directionnel
  doc.setFontSize(12)
  doc.setTextColor(41, 128, 185)
  doc.text('Stratégie Directionnelle (Buy Stop / Sell Stop)', 14, startY + 15)
  
  autoTable(doc, {
    startY: startY + 18,
    head: [['Paire', 'Événement', 'Entrée', 'Offset', 'SL', 'T.Stop', 'Durée']],
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
      head: [['Paire', 'Événement', 'Entrée', 'SL Rec.', 'T.Stop', 'Durée']],
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
      head: [['Paire', 'Événement', 'Entrée', 'SL Rec.', 'T.Stop', 'Durée']],
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
