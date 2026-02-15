import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { ArchivedAnalysisData } from '../../../composables/useMetricsModalLoad'

function fmtPips(value: number): string {
  return `${value.toFixed(1)} pips`
}

export async function generateRankingReport(doc: jsPDF, dataList: ArchivedAnalysisData[], startY: number = 20) {
  doc.setFontSize(16)
  doc.text('Classement des Opportunités', 14, startY)
  doc.setFontSize(10)
  doc.text('Basé sur le score de straddle validé (Quarter élu uniquement)', 14, startY + 6)
  
  const opportunities: any[] = []
  
  dataList.forEach(data => {
    if (data.sliceAnalyses && data.sliceAnalyses.length > 0) {
      const bestSlice = data.sliceAnalyses[0]
      const stats = bestSlice.slice.stats
      
      opportunities.push({
        symbol: data.analysisResult.symbol,
        time: bestSlice.slice.startTime,
        score: bestSlice.slice.straddleScore,
        volatility: stats.volatility_mean,
        noise: stats.noise_ratio_mean,
        breakout: stats.breakout_percentage
      })
    }
  })

  opportunities.sort((a, b) => b.score - a.score)

  const rows = opportunities.map((opp, index) => [
    index + 1,
    opp.symbol,
    opp.time,
    opp.score.toFixed(1),
    fmtPips(opp.volatility),
    opp.noise.toFixed(2),
    `${opp.breakout.toFixed(1)}%`
  ])

  autoTable(doc, {
    startY: startY + 10,
    head: [['Rang', 'Paire', 'Heure', 'Score', 'Volatilité', 'Bruit', 'Breakout']],
    body: rows,
    theme: 'striped',
    headStyles: { fillColor: [46, 204, 113] },
  })
}
