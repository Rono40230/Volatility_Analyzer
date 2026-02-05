import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import { formaterPointsAvecPips, convertirPointsEnPips } from '../../pipConverter'

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
  doc.text('Fiche Straddle Simultané : Paire/Événements', 14, startY)
  doc.setFontSize(10)
  doc.text('Corrélation - News Trading (Source: Archives)', 14, startY + 6)
  
  const rowsSimultaneous: any[] = []

  dataList.forEach(data => {
    // Structure attendue: RetroAnalysisResult (Archive "Corrélation de la volatilité")
    const symbol = data.pair || data.symbol || 'Inconnu'
    const event = data.eventType || data.event_type || 'N/A'
    
    // Les données d'archives sont généralement en POINTS
    // On doit les convertir en PIPS pour formaterPointsAvecPips
    
    // SL Recovery (Simultané)
    let slRecoveryPips = 0
    if (data.stopLossRecoverySimultaneous) {
      slRecoveryPips = convertirPointsEnPips(symbol, data.stopLossRecoverySimultaneous)
    }

    // Trailing Stop (Simultané)
    let tsPips = 0
    if (data.trailingStopSimultaneous) {
      tsPips = convertirPointsEnPips(symbol, data.trailingStopSimultaneous)
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

    // Simultané
    rowsSimultaneous.push([
      symbol,
      event,
      entryTime,
      slRecoveryPips ? formaterPointsAvecPips(symbol, slRecoveryPips) : 'N/A',
      tsPips ? formaterPointsAvecPips(symbol, tsPips) : 'N/A',
      duration ? `${duration}m` : 'N/A'
    ])
  })

  doc.setFontSize(12)
  doc.setTextColor(142, 68, 173)
  doc.text('Stratégie Simultanée (Straddle)', 14, startY + 15)

  autoTable(doc, {
    startY: startY + 18,
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
