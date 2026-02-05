import jsPDF from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { BacktestResult, BacktestConfig } from '../stores/backtest'
import { Ref } from 'vue'

export function useBacktestExport(
  result: Ref<BacktestResult | null>,
  config: Ref<BacktestConfig | undefined>
) {

  function formatDate(iso: string) {
    return new Date(iso).toLocaleString('fr-FR')
  }

  function exportJson() {
    if (!result.value) return
    const dataStr = JSON.stringify(result.value, null, 2)
    const blob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `backtest_result_${new Date().toISOString().split('T')[0]}.json`
    link.click()
    URL.revokeObjectURL(url)
  }

  function exportPdf() {
    if (!result.value) return
    const doc = new jsPDF()
    
    // Titre
    doc.setFontSize(18)
    doc.text('Rapport de Backtest', 14, 20)
    
    doc.setFontSize(10)
    doc.text(`Généré le ${new Date().toLocaleString('fr-FR')}`, 14, 28)
    
    // Configuration
    doc.setFontSize(12)
    doc.text('Configuration', 14, 40)
    doc.setFontSize(10)
    if (config.value) {
      doc.text(`Mode: Simultané`, 14, 48)
      doc.text(`Offset: ${config.value.offset_pips} pips`, 14, 54)
      doc.text(`SL: ${config.value.stop_loss_pips} pips`, 80, 54)
      doc.text(`TP: ${config.value.offset_pips * 2} pips`, 140, 54)
    }

    // Résultats
    doc.setFontSize(12)
    doc.text('Performance', 14, 70)
    doc.setFontSize(10)
    doc.text(`Win Rate: ${result.value.win_rate_percent.toFixed(1)}%`, 14, 78)
    doc.text(`Total Pips: ${result.value.total_pips.toFixed(1)}`, 60, 78)
    doc.text(`Profit Factor: ${result.value.profit_factor.toFixed(2)}`, 110, 78)
    doc.text(`Max Drawdown: -${result.value.max_drawdown_pips.toFixed(1)} pips`, 160, 78)
    doc.text(`Trades: ${result.value.total_trades} (${result.value.winning_trades}W / ${result.value.losing_trades}L)`, 14, 84)

    // Tableau des trades
    const tableData = result.value.trades.map(t => [
      formatDate(t.event_date),
      t.entry_time ? formatDate(t.entry_time).split(' ')[1] : '-',
      t.exit_time ? formatDate(t.exit_time).split(' ')[1] : '-',
      t.duration_minutes + 'm',
      t.pips_net.toFixed(1),
      t.outcome
    ])

    autoTable(doc, {
      startY: 95,
      head: [['Date', 'Entrée', 'Sortie', 'Durée', 'Pips', 'Résultat']],
      body: tableData,
      theme: 'grid',
      headStyles: { fillColor: [41, 98, 255] },
      styles: { fontSize: 8 },
      columnStyles: {
        4: { halign: 'right' }
      }
    })

    // Construction du nom de fichier
    const pair = result.value.symbol || 'UnknownPair'
    const event = result.value.event_name || 'UnknownEvent'
    const strategyMode = 'Simultane'
    const offset = config.value?.offset_pips ?? 0
    const sl = config.value?.stop_loss_pips ?? 0
    const timeout = config.value?.timeout_minutes ?? 0
    const spread = config.value?.spread_pips ?? 0

    const safePair = pair.replace(/[^a-zA-Z0-9]/g, '')
    const safeEvent = event.replace(/[^a-zA-Z0-9]/g, '_')
    
    const filename = `backtest_${safePair}_${safeEvent}_${strategyMode}_${offset}-${sl}-${timeout}-${spread}.pdf`

    doc.save(filename)
  }

  return {
    exportJson,
    exportPdf
  }
}
