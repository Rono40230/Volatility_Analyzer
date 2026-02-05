import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { ArchivedBacktestData } from './dataFetcher'
import { formaterPointsAvecPips } from '../pipConverter'

export async function generateBacktestReport(doc: jsPDF, dataList: ArchivedBacktestData[], startY: number = 20) {
  doc.setFontSize(16)
  doc.text('Rapport de Performance Backtest', 14, startY)
  doc.setFontSize(10)
  doc.text('Simulation historique sur les événements sélectionnés', 14, startY + 6)
  
  let yPos = startY + 15

  for (const data of dataList) {
    const res = data.result
    const cfg = data.config
    
    // Nouvelle page si nécessaire
    if (yPos > 250) {
      doc.addPage()
      yPos = 20
    }

    // En-tête de section
    doc.setFontSize(14)
    doc.setTextColor(41, 128, 185)
    doc.text(`• ${res.symbol} - ${res.event_name}`, 14, yPos)
    doc.setTextColor(0, 0, 0)
    doc.setFontSize(10)
    yPos += 8

    // Paramètres utilisés
    doc.setFontSize(9)
    doc.setTextColor(100)
    const offsetStr = formaterPointsAvecPips(res.symbol, cfg.offset_pips)
    const slStr = formaterPointsAvecPips(res.symbol, cfg.stop_loss_pips)
    doc.text(`Mode: Simultané | Offset: ${offsetStr} | SL: ${slStr} | Spread: ${cfg.spread_pips} | Slippage: ${cfg.slippage_pips}`, 14, yPos)
    doc.setTextColor(0)
    doc.setFontSize(10)
    yPos += 8

    // Tableau des métriques
    const metrics = [
      ['Win Rate', `${res.win_rate_percent.toFixed(1)}%`],
      ['Profit Factor', res.profit_factor.toFixed(2)],
      ['Total Trades', res.total_trades.toString()],
      ['Total Pips', formaterPointsAvecPips(res.symbol, res.total_pips)],
      ['Max Drawdown', formaterPointsAvecPips(res.symbol, res.max_drawdown_pips)],
      ['Avg Pips/Trade', formaterPointsAvecPips(res.symbol, res.average_pips_per_trade)]
    ]

    autoTable(doc, {
      startY: yPos,
      head: [['Métrique', 'Valeur', 'Métrique', 'Valeur']],
      body: [
        [metrics[0][0], metrics[0][1], metrics[3][0], metrics[3][1]],
        [metrics[1][0], metrics[1][1], metrics[4][0], metrics[4][1]],
        [metrics[2][0], metrics[2][1], metrics[5][0], metrics[5][1]]
      ],
      theme: 'grid',
      headStyles: { fillColor: [52, 73, 94] },
      styles: { fontSize: 10, cellPadding: 2 },
      margin: { left: 14, right: 14 }
    })

    // @ts-ignore
    yPos = doc.lastAutoTable.finalY + 10

    // Petit tableau des 5 derniers trades
    const lastTrades = res.trades.slice(-5).reverse().map((t: any) => [
      new Date(t.event_date).toLocaleDateString(),
      t.outcome,
      formaterPointsAvecPips(res.symbol, t.pips_net),
      `${t.duration_minutes}m`
    ])

    if (lastTrades.length > 0) {
      doc.text('Derniers trades:', 14, yPos)
      yPos += 5
      
      autoTable(doc, {
        startY: yPos,
        head: [['Date', 'Résultat', 'Pips', 'Durée']],
        body: lastTrades,
        theme: 'plain',
        styles: { fontSize: 8, cellPadding: 1 },
        margin: { left: 14, right: 100 } // Tableau compact à gauche
      })
      
      // @ts-ignore
      yPos = doc.lastAutoTable.finalY + 15
    } else {
      yPos += 10
    }
  }
}
