<script setup lang="ts">
import { conversionData } from '../data/conversionTableData'
</script>

<template>
  <div class="conversion-container">
    <div class="conversion-header">
      <div class="conversion-title-group">
        <span class="conversion-icon">📊</span>
        <div>
          <h2 class="conversion-title">Conversion Points MT5 vers Pips</h2>
          <p class="conversion-subtitle">Tableau de correspondance pour vos instruments de trading</p>
        </div>
      </div>
    </div>

    <div class="conversion-info-box">
      <div class="info-header">
        <span class="info-icon">🔍</span>
        <h3>Qu'est-ce qu'un point MT5 ?</h3>
      </div>
      <p>
        Sur MetaTrader 5, un <strong>point</strong> représente la plus petite variation de prix possible. 
        Pour la plupart des paires Forex, MT5 affiche 5 décimales (ex: 1.10538), donc <strong>1 pip = 10 points MT5</strong>.
      </p>
    </div>

    <div class="table-wrapper">
      <table class="conversion-table">
        <thead>
          <tr>
            <th>INSTRUMENT</th>
            <th>FORMAT PRIX MT5</th>
            <th>1 PIP =</th>
            <th>VALEUR EN $ (1 LOT)</th>
            <th>CONVERSION</th>
            <th>EXEMPLE</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="category in conversionData" :key="category.name">
            <tr class="category-row">
              <td colspan="6">{{ category.name }}</td>
            </tr>
            <tr v-for="row in category.rows" :key="row.symbol">
              <td class="symbol">{{ row.symbol }}</td>
              <td>{{ row.priceFormat }}</td>
              <td>{{ row.pipValue }}</td>
              <td>{{ row.lotValue }}</td>
              <td class="formula">{{ row.formula }}</td>
              <td class="example">{{ row.example }}</td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <div class="conversion-footer">
      <div class="footer-section">
        <h4>📌 Règles de Conversion :</h4>
        <ul>
          <li><strong>Paires Forex (EUR/USD, GBP/JPY, etc.) :</strong> 1 pip = 10 points → Divisez par 10</li>
          <li><strong>Métaux Précieux (XAU/USD, XAG/USD) :</strong> 1 pip = 10 points → Divisez par 10</li>
          <li><strong>Indices (USATEC, USAIDX, DEUIDX) :</strong> 1 pip = 1 point → Pas de conversion</li>
          <li><strong>Crypto (BTC/USD) :</strong> 1 pip = 1 point → Pas de conversion</li>
        </ul>
      </div>

      <div class="footer-section">
        <h4>💰 Valeur en $ (pour 1 lot standard) :</h4>
        <ul>
          <li><strong>EUR/USD, GBP/USD, USD/CAD, USD/JPY :</strong> 1 pip = $10 (0.0001 × 100,000 unités)</li>
          <li><strong>GBP/JPY, CAD/JPY :</strong> Variable selon le taux USD/JPY (≈$6.67 si USD/JPY=150)*</li>
          <li><strong>XAU/USD (Or) :</strong> 1 pip = $10 (0.10 × 100 oz)</li>
          <li><strong>XAG/USD (Argent) :</strong> 1 pip = $50 (0.01 × 5,000 oz)</li>
          <li><strong>USATEC, USAIDX, BTC/USD :</strong> 1 pip = $1 par contrat</li>
          <li><strong>DEUIDX (DAX) :</strong> 1 pip = €1 par contrat</li>
        </ul>
        <p class="footer-note">*Variable : La valeur en $ dépend du taux de change avec l'USD au moment du trade.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversion-container {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 1rem;
  color: #e2e8f0;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.conversion-header {
  background: linear-gradient(135deg, #1a1f2e 0%, #2d3748 100%);
  padding: 1.5rem;
  border-radius: 12px;
  border: 1px solid #2d3748;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}

.conversion-title-group {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.conversion-icon {
  font-size: 2.5rem;
}

.conversion-title {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 700;
  color: #fff;
}

.conversion-subtitle {
  margin: 0.25rem 0 0 0;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.7);
}

.conversion-info-box {
  background: rgba(52, 152, 219, 0.1);
  border-left: 4px solid #3498db;
  padding: 1.25rem;
  border-radius: 0 8px 8px 0;
}

.info-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.info-header h3 {
  margin: 0;
  font-size: 1.1rem;
  color: #3498db;
}

.conversion-info-box p {
  margin: 0;
  line-height: 1.5;
  font-size: 0.95rem;
  color: #cbd5e0;
}

.table-wrapper {
  overflow-x: auto;
  border-radius: 12px;
  border: 1px solid #2d3748;
  background: #1a1f2e;
}

.conversion-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.conversion-table th {
  background: #6c5ce7;
  color: white;
  text-align: left;
  padding: 1rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.conversion-table td {
  padding: 0.85rem 1rem;
  border-bottom: 1px solid #2d3748;
}

.category-row td {
  background: rgba(45, 55, 72, 0.5);
  font-weight: 700;
  color: #a0aec0;
  padding: 0.75rem 1rem;
  font-size: 0.85rem;
}

.symbol {
  font-weight: 600;
  color: #3498db;
}

.formula {
  font-family: 'Courier New', Courier, monospace;
  background: rgba(241, 196, 15, 0.1);
  color: #f39c12;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
}

.example {
  color: #2ecc71;
  font-weight: 600;
}

.conversion-footer {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
  background: rgba(243, 156, 18, 0.05);
  border: 1px solid rgba(243, 156, 18, 0.2);
  padding: 1.5rem;
  border-radius: 12px;
}

.footer-section h4 {
  margin: 0 0 1rem 0;
  color: #e67e22;
  font-size: 1rem;
}

.footer-section ul {
  margin: 0;
  padding-left: 1.25rem;
  list-style-type: none;
}

.footer-section li {
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
  position: relative;
  color: #cbd5e0;
}

.footer-section li::before {
  content: "•";
  position: absolute;
  left: -1rem;
  color: #e67e22;
}

.footer-note {
  margin-top: 1rem;
  font-size: 0.8rem;
  font-style: italic;
  color: #a0aec0;
}

@media (max-width: 900px) {
  .conversion-footer {
    grid-template-columns: 1fr;
  }
}
</style>
