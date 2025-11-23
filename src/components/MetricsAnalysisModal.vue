<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <!-- En-tête -->
      <div class="modal-header">
        <div class="header-title">
          <span class="icon">🎯</span>
          <h2>Métriques du meilleur moment pour trader</h2>
        </div>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <!-- Meilleur Moment -->
      <div class="modal-section">
        <div v-if="sliceAnalyses && sliceAnalyses.length > 0" class="slices-container">
          <!-- Affiche seulement le meilleur moment (rank 1) -->
          <div v-for="analysis in sliceAnalyses.filter(a => a.rank === 1)" :key="`slice-${analysis.rank}`" class="slice-card" :class="getRankClass(analysis.rank)">
            <!-- Rang + Heure + Recommandation -->
            <div class="slice-header" style="display: flex; justify-content: space-between; align-items: flex-start; gap: 20px;">
              <div style="display: flex; gap: 12px; align-items: flex-start;">
                <div class="rank-badge" :class="`rank-${analysis.rank}`">
                  <span class="rank-medal">⭐</span>
                </div>
                <div class="slice-time">
                  <div class="time">{{ analysis.slice.startTime }}</div>
                  <div class="score" :class="`score-${getScoreSeverity(analysis.slice.straddleScore)}`">
                    Score: {{ analysis.slice.straddleScore.toFixed(0) }}/100
                  </div>
                </div>
              </div>

              <!-- Recommandation inline -->
              <div style="flex: 1; padding: 12px 16px; background: rgba(78, 205, 196, 0.1); border: 1px solid rgba(78, 205, 196, 0.3); border-radius: 6px; font-size: 12px;">
                <div style="color: #4ecdc4; margin-bottom: 6px; font-weight: bold;">🎯 RECOMMANDATION</div>
                <div style="color: #e0e0e0; line-height: 1.5;">
                  <span v-if="analysis.slice.straddleScore >= 75 && (!volatilityDuration || volatilityDuration.confidence_score >= 50)">
                    ✅ <strong>EXCELLENTES</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Straddle optimal. Position size: <strong>75-100%</strong>.
                  </span>
                  <span v-else-if="analysis.slice.straddleScore >= 60 && (!volatilityDuration || volatilityDuration.confidence_score >= 30)">
                    ⚠️ <strong>ACCEPTABLES</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Setup viable. Position size: <strong>50-75%</strong>.
                  </span>
                  <span v-else>
                    ❌ <strong>INSUFFISANTES</strong> ({{ analysis.slice.straddleScore.toFixed(0) }}/100) - Attendre un meilleur setup.
                  </span>
                </div>
              </div>
            </div>

            <!-- Métriques Détaillées -->
            <div class="metrics-section">
              <h4>METRIQUES</h4>
              <div class="metrics-grid">
                <!-- ATR -->
                <MetricTooltip title="ATR Moyen">
                <div class="metric-item">
                  <span class="metric-name">ATR Moyen</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.atr_mean, 0.001, 0.002)">
                      {{ formatNumber(analysis.slice.stats.atr_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_atr ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.001</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.atr_mean, 0.001)]">
                    {{ getMetricStatusText(analysis.slice.stats.atr_mean, 0.001) }}
                  </span>
                </div>
                  <template #definition>Average True Range sur 14 périodes : mesure de volatilité moyenne du créneau horaire.</template>
                  <template #usage>Score &gt;0.002 = Excellent (forte volatilité) | 0.001-0.002 = Bon | &lt;0.001 = Mauvais (peu volatil).</template>
                  <template #scoring>Calculé : (High-Low) moyenne sur 14 barres. Détermine largeur SL/TP. Plus ATR élevé = plus grande opportunité scalping.</template>
                </MetricTooltip>

                <!-- True Range -->
                <MetricTooltip title="True Range">
                <div class="metric-item">
                  <span class="metric-name">True Range</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.range_mean, 0.0015, 0.0025)">
                      {{ formatNumber(analysis.slice.stats.range_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_range ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.0025</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.range_mean, 0.0025)]">
                    {{ getMetricStatusText(analysis.slice.stats.range_mean, 0.0025) }}
                  </span>
                </div>
                  <template #definition>Max(High-Low, |High-Close[t-1]|, |Low-Close[t-1]|) : mouvement total exploitable incluant les gaps overnight et clôtures précédentes.</template>
                  <template #usage>Score >2.5% = Excellent | 1.5-2.5% = Bon | <1.5% = Faible.</template>
                  <template #scoring>True Range croissant = meilleur straddle setup. Capture les gaps contrairement au simple Range. Combine avec ATR pour détecter les vrais breakouts.</template>
                </MetricTooltip>

                <!-- Volatility -->
                <MetricTooltip title="Volatilité">
                <div class="metric-item">
                  <span class="metric-name">Volatilité %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.volatility_mean * 100, 15, 30)">
                      {{ formatNumber(analysis.slice.stats.volatility_mean * 100, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber((analysisData?.globalMetrics.mean_volatility ?? 0) * 100, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>15%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.volatility_mean * 100, 15)]">
                    {{ getMetricStatusText(analysis.slice.stats.volatility_mean * 100, 15) }}
                  </span>
                </div>
                  <template #definition>Ratio ATR / Close exprimé en pourcentage : mesure la volatilité relative à la source du mouvement.</template>
                  <template #usage>Score &gt;30% = Exceptionnellement volatil | 15-30% = Bon | &lt;15% = Faible. Créneau sans volatilité = pas de setup.</template>
                  <template #scoring>Formula: (ATR / Close) × 100. Ratio volatilité actuelle / volatilité globale ajuste les positions dynamiquement.</template>
                </MetricTooltip>

                <!-- BodyRange -->
                <MetricTooltip title="Body Range">
                <div class="metric-item">
                  <span class="metric-name">Body Range %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.body_range_mean, 25, 45)">
                      {{ formatNumber(analysis.slice.stats.body_range_mean, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_body_range ?? 0, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>45%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.body_range_mean, 45)]">
                    {{ getMetricStatusText(analysis.slice.stats.body_range_mean, 45) }}
                  </span>
                </div>
                  <template #definition>Pourcentage du range représenté par le body (Close-Open) : pureté du signal sans bruit des wicks.</template>
                  <template #usage>Score &gt;45% = Signal Très Pur (peu de bruit) | 25-45% = Acceptable | &lt;25% = Bruité (wicks dominants).</template>
                  <template #scoring>Formula: (|Close - Open| / (High - Low)) × 100. Corps fort = pression directionnelle claire, moins de faux mouvements.</template>
                </MetricTooltip>

                <!-- TickQuality -->
                <MetricTooltip title="Tick Quality">
                <div class="metric-item">
                  <span class="metric-name">Tick Quality</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.tick_quality_mean, 0.0005, 0.001)">
                      {{ formatNumber(analysis.slice.stats.tick_quality_mean, 5) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_tick_quality ?? 0, 5) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold">>0.001</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.tick_quality_mean, 0.001)]">
                    {{ getMetricStatusText(analysis.slice.stats.tick_quality_mean, 0.001) }}
                  </span>
                </div>
                  <template #definition>Mesure la douceur du pricing : variance des ticks. Élevé = mouvements chaotiques, faible = trend smooth.</template>
                  <template #usage>Score &gt;0.001 = Excellent (mouvements lisses et directionnels) | &lt;0.0005 = Mauvais (bruit, whipsaws)</template>
                  <template #scoring>Formula: Standard deviation des mouvements de tick. Détermine la qualité du signal pour entrée/scalp clean.</template>
                </MetricTooltip>

                <!-- NoiseRatio -->
                <MetricTooltip title="Noise Ratio">
                <div class="metric-item">
                  <span class="metric-name">Noise Ratio</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(2.5 - analysis.slice.stats.noise_ratio_mean, 0, 1.5)">
                      {{ formatNumber(analysis.slice.stats.noise_ratio_mean, 2) }}
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_noise_ratio ?? 0, 2) }}</span>
                    <span class="separator">|</span>
                    <span class="threshold"><2.0</span>
                  </div>
                  <span :class="['status', getNoiseStatus(analysis.slice.stats.noise_ratio_mean)]">
                    {{ getNoiseStatusText(analysis.slice.stats.noise_ratio_mean) }}
                  </span>
                </div>
                  <template #definition>Ratio wicks/body : mesure le "bruit" vs la vraie direction. Bas = signal pur, haut = beaucoup de fausses cassures.</template>
                  <template #usage>Score &lt;2.0 = Signal Excellent (peu de bruit) | 2.0-2.5 = Acceptable | &gt;2.5 = Très Bruité (éviter).</template>
                  <template #scoring>Formula: (Total_wicks_range / Body_range). Bas = direction confirmée. Élevé = beaucoup de rejets = whipsaws.</template>
                </MetricTooltip>

                <!-- Direction Strength -->
                <MetricTooltip title="Direction Strength">
                <div class="metric-item">
                  <span class="metric-name">Direction Strength</span>
                  <div class="metric-values">
                    <span class="value15" :class="getDirectionStrengthClass(analysis.slice.stats.volume_imbalance_mean)">
                      {{ formatNumber(analysis.slice.stats.volume_imbalance_mean * 100, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber((analysisData?.globalMetrics.mean_volume_imbalance ?? 0) * 100, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">20%+ optimal</span>
                  </div>
                  <span :class="['status', getDirectionStrengthStatus(analysis.slice.stats.volume_imbalance_mean)]">
                    {{ getDirectionStrengthStatusText(analysis.slice.stats.volume_imbalance_mean) }}
                  </span>
                </div>
                  <template #definition>Direction Strength = (|Body Range %| × Breakout %) / 100. Mesure puissance mouvement directionnel optimal Forex.</template>
                  <template #usage>
                    🟢 <strong>&gt;20%:</strong> Excellent directional<br>
                    🔵 <strong>10-20%:</strong> Bon<br>
                    🟠 <strong>5-10%:</strong> Moyen<br>
                    🔴 <strong>&lt;5%:</strong> Faible direction.
                  </template>
                  <template #scoring>Combine directionnalite (corps de bougie) + cassures identifiees = proxy force direction optimal Forex Straddle.</template>
                </MetricTooltip>

                <!-- Breakout % -->
                <MetricTooltip title="Breakout">
                <div class="metric-item">
                  <span class="metric-name">Breakout %</span>
                  <div class="metric-values">
                    <span class="value15" :class="getMetricClass(analysis.slice.stats.breakout_percentage, 10, 20)">
                      {{ formatNumber(analysis.slice.stats.breakout_percentage, 1) }}%
                    </span>
                    <span class="separator">|</span>
                    <span class="valueglobal">{{ formatNumber(analysisData?.globalMetrics.mean_breakout_percentage ?? 0, 1) }}%</span>
                    <span class="separator">|</span>
                    <span class="threshold">>15%</span>
                  </div>
                  <span :class="['status', getMetricStatus(analysis.slice.stats.breakout_percentage, 15)]">
                    {{ getMetricStatusText(analysis.slice.stats.breakout_percentage, 15) }}
                  </span>
                </div>
                  <template #definition>Pourcentage de fois où le prix casse les niveaux clés (support/résistance) du créneau : opportunités de tendance.</template>
                  <template #usage>Score &gt;15% = Breakouts Fréquents (momentum exploitable) | &lt;10% = Peu de breakouts (range-bound).</template>
                  <template #scoring>Formula: (Breakout_events / Total_periods) × 100. Haut = plus d'opportunités trendy, bas = plus d'oscillation/straddle pure.</template>
                </MetricTooltip>
              </div>
            </div>

            <!-- Movement Quality Analysis -->
            <div class="movement-quality-section">
              <h4>💫 Qualité du Mouvement</h4>
              
              <!-- Pas d'événements -->
              <div v-if="analysis.slice.stats.events.length === 0" style="color: #999;">
                ⚠️ Pas d'événement dans ce slice
              </div>
              
              <!-- Clé vide -->
              <div v-else-if="!getMovementQualityKey(analysis)" style="color: #999;">
                ⚠️ Clé vide générée
              </div>
              
              <!-- Données chargées -->
              <div v-else-if="movementQualities[getMovementQualityKey(analysis)]" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px; margin-top: 15px;">
                <!-- Score Qualité -->
                <MetricTooltip title="Score Qualité">
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                  <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Score Qualité</div>
                  <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                    {{ (movementQualities[getMovementQualityKey(analysis)]?.quality_score || 0).toFixed(1) }}/10
                  </div>
                </div>
                <template #definition>Notation globale 0-10 de la qualité du setup combinant tous les facteurs : volatilité, signal purity, mouvement directionnel.</template>
                <template #usage>Score &gt;7 = Excellent (trader) | 5-7 = Acceptable | &lt;5 = Mauvais (skip). Basé sur pondération : Volatilité 40%, Signal 35%, Direction 25%.</template>
                <template #scoring>Formula: (ATR_score × 0.4 + Body_Range_score × 0.35 + Direction_score × 0.25) / 10. Seuil global qualité.</template>
                </MetricTooltip>
                
                <!-- Mouvement Directionnel -->
                <MetricTooltip title="Mouvement Directionnel">
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                  <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Mouvement Directionnel</div>
                  <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                    {{ ((movementQualities[getMovementQualityKey(analysis)]?.directional_move_rate || 0) * 100).toFixed(0) }}%
                  </div>
                </div>
                <template #definition>Pourcentage du range total qui s'est déplacé dans une direction cohérente sans retracer significativement.</template>
                <template #usage>Score &gt;70% = Très directionnel (bon momentum) | 50-70% = Modérément directionnel | &lt;50% = Chaotique/bidirectionnel.</template>
                <template #scoring>Formula: (Net_directional_pips / Total_range) × 100. Élevé = tendance claire, faible = oscillation indécise.</template>
                </MetricTooltip>
                
                <!-- Whipsaw Rate -->
                <MetricTooltip title="Whipsaw Rate">
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                  <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Whipsaw Rate</div>
                  <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                    {{ ((movementQualities[getMovementQualityKey(analysis)]?.whipsaw_rate || 0) * 100).toFixed(0) }}%
                  </div>
                </div>
                <template #definition>Pourcentage de fausses sorties : fois où le prix dépasse SL temporairement avant de revenir vers TP (dangereux au scalp).</template>
                <template #usage>Score &lt;10% = Excellent (peu de faux signaux) | 10-20% = Acceptable | &gt;20% = Danger (trop de whipsaws, avoid).</template>
                <template #scoring>Formula: (Whipsaw_events / Total_trades) × 100. Barrière psychologique et cash-drag majeure. À minimiser absolument.</template>
                </MetricTooltip>
                
                <!-- Taux Succès -->
                <MetricTooltip title="Taux Succès">
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                  <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Taux Succès</div>
                  <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                    {{ ((movementQualities[getMovementQualityKey(analysis)]?.success_rate || 0) * 100).toFixed(0) }}%
                  </div>
                </div>
                <template #definition>Pourcentage d'événements dans ce créneau qui ont atteint leur objectif TP avant d'être arrêtés au SL (win rate brut).</template>
                <template #usage>Score &gt;60% = Excellent (trades qui marche) | 50-60% = Bon (profitable avec R/R) | &lt;50% = Mauvais (éviter ce créneau).</template>
                <template #scoring>Formula: (Winning_events / Total_events) × 100. Directement utilisé pour profitabilité espérance = WR × TP - (1-WR) × SL.</template>
                </MetricTooltip>
                
                <!-- Mouvement Moyen -->
                <MetricTooltip title="Mouvement Moyen">
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                  <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Mouvement Moyen</div>
                  <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                    {{ (movementQualities[getMovementQualityKey(analysis)]?.avg_pips_moved || 0).toFixed(1) }} <span style="color: #888; font-size: 11px;">pips</span>
                  </div>
                </div>
                <template #definition>Distance moyenne en pips que le prix parcourt par événement dans ce créneau historiquement.</template>
                <template #usage>Score &gt;15 pips = Excellent (suffisant pour scalp) | 10-15 pips = Bon | &lt;10 pips = Faible mouvement (skip).</template>
                <template #scoring>Formula: Sum(|move_pips|) / Number_events. Doit être &gt; SL pour que TP soit atteignable (SL +TP × R/R) = mouvement attendu.</template>
                </MetricTooltip>
              </div>
              
              <!-- Chargement en cours -->
              <div v-else class="quality-loading">
                ⏳ Analyse du mouvement en cours...
              </div>
            </div>

            <!-- Durée de Volatilité -->
            <div style="margin-top: 20px; padding: 20px; background: linear-gradient(135deg, rgba(45, 90, 123, 0.15) 0%, rgba(78, 205, 196, 0.1) 100%); border: 1px solid #2d5a7b; border-radius: 8px;">
              <h4>⏱️ DURÉE DE VOLATILITÉ</h4>
              <div style="margin-top: 15px; display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px;">
                <!-- Peak Duration -->
                <MetricTooltip title="Durée du Pic" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Durée Pic</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                      {{ volatilityDuration?.peak_duration_minutes || '—' }} <span style="color: #888; font-size: 11px;">min</span>
                    </div>
                  </div>
                  <template #definition>Nombre de minutes où la volatilité reste supérieure à 80% du pic observé pendant le créneau.</template>
                  <template #usage>Indique combien de temps le mouvement principal persiste avant de perdre son énergie. Exemple : NFP = 90-150min, données faibles = 150-270min.</template>
                  <template #scoring>Calculé à partir de l'ATR, Range et Body Range empiriques. Volatilité très élevée (ATR>50pts) = pic court. Volatilité faible (ATR<25pts) = pic long.</template>
                </MetricTooltip>

                <!-- Volatility Half-Life -->
                <MetricTooltip title="Demi-Vie de Volatilité" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Demi-Vie</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                      {{ volatilityDuration?.volatility_half_life_minutes || '—' }} <span style="color: #888; font-size: 11px;">min</span>
                    </div>
                  </div>
                  <template #definition>Nombre de minutes pour que la volatilité décroisse à 50% de son pic (décroissance exponentielle).</template>
                  <template #usage>Mesure la vitesse de dissipation de la volatilité. Demi-vie courte (30-50min) = volatilité s'effondre rapidement. Demi-vie longue (80-120min) = volatilité persiste.</template>
                  <template #scoring>Basée sur le Noise Ratio et la stabilité du mouvement. NR<1.5 (stable) = demi-vie longue 60-70% du pic. NR>2.5 (décroissant) = demi-vie courte 30-40% du pic.</template>
                </MetricTooltip>

                <!-- Trade Duration (NEW PARAM) -->
                <MetricTooltip title="Durée du Trade" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Durée Trade</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                      {{ tradingPlan?.tradeDurationMinutes || '—' }} <span style="color: #888; font-size: 11px;">min</span>
                    </div>
                  </div>
                  <template #definition>Durée optimale de trading recommandée = max(peak_duration, demi-vie × 2). C'est le temps maximum avant que la volatilité devienne insuffisante.</template>
                  <template #usage>Indique quand fermer le trade pour éviter les whipsaws en fin de mouvement. Exemple : si durée=150min et entrée 14h30, fermer avant 16h20. Crucial pour le trailing stop.</template>
                  <template #scoring>Formula : max(peak_duration, half_life × 2). Protège contre surtemps = perte. Exemple: pic 150min + half-life 60min → max(150, 120) = 150min de trading.</template>
                </MetricTooltip>

                <!-- Confidence Score -->
                <MetricTooltip title="Score de Confiance" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Confiance</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                      {{ volatilityDuration?.confidence_score || '—' }} <span style="color: #888; font-size: 11px;">%</span>
                    </div>
                  </div>
                  <template #definition>Fiabilité des métriques de durée basée sur la taille de l'échantillon historique du créneau.</template>
                  <template #usage>Score ≥90% = métriques très fiables (données abondantes). Score 50-75% = données partielles, variance possible. Influence la position size et le stop loss.</template>
                  <template #scoring>Basé sur le sample_size du créneau : ≥100 occurrences = 100%, 50-99 = 90%, 30-49 = 75%, 15-29 = 60%, &lt;15 = 50%.</template>
                </MetricTooltip>
              </div>
            </div>

            <!-- Paramètres Bidi Optimisés -->
            <div style="margin-top: 20px; padding: 20px; background: #1a1a2e; border: 1px solid #16213e; border-radius: 8px;">
              <h4>⚙️ PARAMÈTRES BIDI OPTIMISÉS</h4>
              <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px; margin-top: 15px;">
                <MetricTooltip title="Meilleur Moment" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Meilleur Moment</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">
                      {{ sliceAnalyses && sliceAnalyses.length > 0 ? calculateExactTime() : entryWindowAnalysis.optimal_offset + ' min' }}
                    </div>
                  </div>
                  <template #definition>L'heure exacte d'entrée optimale pour le straddle basée sur l'analyse historique des créneau horaires.</template>
                  <template #usage>Entrée au-delà de 14:00 avec ≥ 3 créneau optimaux et un taux de succès ≥ 55%.</template>
                  <template #scoring>Sélectionné parmi les 3 meilleurs créneau horaires du jour avec le plus haut taux de succès ajusté.</template>
                </MetricTooltip>
                <MetricTooltip title="Taux de Succès" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Taux de Succès</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ (entryWindowAnalysis.optimal_win_rate * 100).toFixed(0) }}% <span style="color: #888; font-size: 11px;">événement</span></div>
                  </div>
                  <template #definition>Pourcentage de fois où le créneau horaire a produit un mouvement straddle gagnant (atteint TP avant SL).</template>
                  <template #usage>Critère crucial : minimum 55% pour un biais positif. ≥65% = excellent signal. &lt;50% = dangereux.</template>
                  <template #scoring>Calculé sur l'historique complet du créneau avec ajustement volatilité/range/body-range.</template>
                </MetricTooltip>
                <MetricTooltip title="Stop Loss" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Stop Loss</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ analysis.tradingPlan.slPips }} <span style="color: #888; font-size: 11px;">pips</span></div>
                  </div>
                  <template #definition>Distance en pips entre l'entrée et le niveau de stop loss (limite de perte).</template>
                  <template #usage>Calculé dynamiquement : SL = (Score/100 × Range_actuelle) / 1.5. Exemple : score 60 = ±20 pips de range.</template>
                  <template #scoring>Formula: SL_pips = (Score/100) × (ATR × 2.5). Augmente avec la volatilité, diminue si score faible (&lt;50).</template>
                </MetricTooltip>
                <MetricTooltip title="Win Rate" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Win Rate</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ analysis.tradingPlan.winProbability }}% <span style="color: #888; font-size: 11px;">histo</span></div>
                  </div>
                  <template #definition>Pourcentage de trades théoriques gagnants selon l'historique des mouvements (atteint TP avant SL).</template>
                  <template #usage>Basé sur les histogrammes de distribution des mouvements du créneau. &gt;55% = profitable à long terme. &lt;50% = stop trading.</template>
                  <template #scoring>Calculé à partir de : success_rate du créneau + volatility_score + body_range_score. Ajustement variance inclus.</template>
                </MetricTooltip>
                <MetricTooltip title="Avg Gain" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Avg Gain</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ analysis.tradingPlan.avgGainR.toFixed(1) }}R <span style="color: #888; font-size: 11px;">moyen</span></div>
                  </div>
                  <template #definition>Espérance mathématique moyenne en "R" (risque unitaire). Exemple : 0.5R = 50% du risque en gain moyen.</template>
                  <template #usage>Critère clé : Avg Gain = (Win% × Win_avg) - (Loss% × Loss_avg) × Risk. &gt;0.3R = très bon. &lt;0 = à éviter.</template>
                  <template #scoring>Formula: AvgGain = (win_rate × avg_win_pips - (1-win_rate) × avg_loss_pips) / SL_pips. Mesure la profitabilité nette.</template>
                </MetricTooltip>
                <MetricTooltip title="Trailing Stop" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Trailing Stop</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ analysis.tradingPlan.trailingStopCoefficient.toFixed(2) }}x <span style="color: #888; font-size: 11px;">ATR</span></div>
                  </div>
                  <template #definition>Multiplicateur ATR pour recalculer dynamiquement le stop loss en hausse (protection des gains). Fixe le SL à [prix bas × (1.5 + volatilité_ratio)].</template>
                  <template #usage>Trailing = 1.5x ATR + (ATR_current/ATR_avg - 1) × 0.5. Exemple : ATR_actuel 0.002 = 1.8x. Permet de sécuriser les gains sans bloquer.</template>
                  <template #scoring>Formula: Coefficient = 1.5 + (ATR_current/ATR_moyenne - 1) × 0.5. Plage 1.5-2.5x. Volatilité haute = coefficient plus bas (plus serré).</template>
                </MetricTooltip>
                <MetricTooltip title="Trade Expiration" direction="top">
                  <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px;">
                    <div style="font-size: 11px; color: #999; margin-bottom: 6px; text-transform: uppercase;">Expiration</div>
                    <div style="font-size: 13px; color: #4ecdc4; font-weight: bold;">{{ analysis.tradingPlan.tradeExpiration || '—' }} <span style="color: #888; font-size: 11px;">min</span></div>
                  </div>
                  <template #definition>Limite de temps maximale avant fermeture automatique du trade (dans le robot Bidi). Basée sur la volatilité et remplace les 300min fixes.</template>
                  <template #usage>Entrée à 14h30 + expiration 180min = fermer avant 16h30. Si TP non atteint à l'expiration, fermer à market. Évite les whipsaws post-peak.</template>
                  <template #scoring>Formula: max(peak_duration, half_life × 2). Ajustée selon ATR. Volatilité haute = expiration courte (120-150min). Volatilité faible = expiration longue (240-270min). Max 300min.</template>
                </MetricTooltip>
              </div>
            </div>

            <!-- Observations & Conseils -->
            <div style="margin-top: 20px; padding: 20px; background: #1a1a2e; border: 1px solid #16213e; border-radius: 8px;">
              <h4>📋 OBSERVATIONS & CONSEILS</h4>
              <div style="margin-top: 15px; display: grid; grid-template-columns: repeat(5, 1fr); gap: 12px;">
                <!-- Analyse Range -->
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px; display: flex; flex-direction: column;">
                  <div style="font-size: 12px; color: #999; margin-bottom: 6px; font-weight: bold;">📊 Range</div>
                  <div style="font-size: 13px; color: #fff; font-weight: bold;">
                    {{ (analysis.slice.stats.range_mean * 10000).toFixed(0) }} pips
                  </div>
                  <div style="font-size: 11px; margin: 6px 0;">
                    <span v-if="analysis.slice.stats.range_mean > 0.0025" style="color: #4ecdc4;">✅ Excellent</span>
                    <span v-else-if="analysis.slice.stats.range_mean > 0.0015" style="color: #ffd700;">⚠️ Bon</span>
                    <span v-else style="color: #ff6b6b;">❌ Faible</span>
                  </div>
                  <div style="font-size: 10px; color: #888; margin-top: auto;">
                    💡 Plus élevé = meilleure opportunité straddle
                  </div>
                </div>

                <!-- Analyse ATR -->
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px; display: flex; flex-direction: column;">
                  <div style="font-size: 12px; color: #999; margin-bottom: 6px; font-weight: bold;">⚡ ATR</div>
                  <div style="font-size: 13px; color: #fff; font-weight: bold;">
                    {{ (analysis.slice.stats.atr_mean * 10000).toFixed(0) }} pips
                  </div>
                  <div style="font-size: 11px; margin: 6px 0;">
                    <span v-if="analysis.slice.stats.atr_mean > 0.0020" style="color: #4ecdc4;">✅ Excellent</span>
                    <span v-else-if="analysis.slice.stats.atr_mean > 0.0010" style="color: #ffd700;">⚠️ Bon</span>
                    <span v-else style="color: #ff6b6b;">❌ Faible</span>
                  </div>
                  <div style="font-size: 10px; color: #888; margin-top: auto;">
                    💡 Volatilité confirmée = SL/TP plus larges
                  </div>
                </div>

                <!-- Analyse Body Range -->
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px; display: flex; flex-direction: column;">
                  <div style="font-size: 12px; color: #999; margin-bottom: 6px; font-weight: bold;">📈 Body Range</div>
                  <div style="font-size: 13px; color: #fff; font-weight: bold;">
                    {{ analysis.slice.stats.body_range_mean.toFixed(1) }}%
                  </div>
                  <div style="font-size: 11px; margin: 6px 0;">
                    <span v-if="analysis.slice.stats.body_range_mean > 45" style="color: #4ecdc4;">✅ Très Pur</span>
                    <span v-else-if="analysis.slice.stats.body_range_mean > 25" style="color: #ffd700;">⚠️ Acceptable</span>
                    <span v-else style="color: #ff6b6b;">❌ Très Bruité</span>
                  </div>
                  <div style="font-size: 10px; color: #888; margin-top: auto;">
                    💡 Élevé = signal pur, peu de bruit
                  </div>
                </div>

                <!-- Analyse Qualité Mouvement (Phase 1.2) -->
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px; display: flex; flex-direction: column;">
                  <div style="font-size: 12px; color: #999; margin-bottom: 6px; font-weight: bold;">💫 Qualité</div>
                  <div style="font-size: 13px; color: #fff; font-weight: bold;">
                    <template v-if="movementQualities[getMovementQualityKey(analysis)]?.quality_score">
                      {{ (movementQualities[getMovementQualityKey(analysis)]?.quality_score || 0).toFixed(1) }}/10
                    </template>
                    <template v-else>
                      —
                    </template>
                  </div>
                  <div style="font-size: 11px; margin: 6px 0;">
                    <template v-if="movementQualities[getMovementQualityKey(analysis)]?.quality_score">
                      <span v-if="(movementQualities[getMovementQualityKey(analysis)]?.quality_score || 0) >= 8" style="color: #4ecdc4;">✅ Excellent</span>
                      <span v-else-if="(movementQualities[getMovementQualityKey(analysis)]?.quality_score || 0) >= 6" style="color: #ffd700;">⚠️ Bon</span>
                      <span v-else style="color: #ff6b6b;">❌ Faible</span>
                    </template>
                    <span v-else style="color: #999;">Calcul...</span>
                  </div>
                  <div style="font-size: 10px; color: #888; margin-top: auto;">
                    💡 Basé sur mouvements directionnel
                  </div>
                </div>

                <!-- Analyse Durée de Volatilité (Phase 1.1) -->
                <div style="padding: 12px; background: rgba(255,255,255,0.05); border-radius: 6px; display: flex; flex-direction: column;">
                  <div style="font-size: 12px; color: #999; margin-bottom: 6px; font-weight: bold;">⏱️ Durée Vol.</div>
                  <div style="font-size: 12px; color: #fff; font-weight: bold;">
                    <template v-if="volatilityDuration?.peak_duration_minutes">
                      {{ volatilityDuration.peak_duration_minutes }}min
                    </template>
                    <template v-else>
                      —
                    </template>
                  </div>
                  <div style="font-size: 11px; margin: 6px 0;">
                    <template v-if="volatilityDuration?.confidence_score">
                      <span v-if="volatilityDuration.confidence_score >= 75" style="color: #4ecdc4;">✅ Haute conf.</span>
                      <span v-else-if="volatilityDuration.confidence_score >= 50" style="color: #ffd700;">⚠️ Moyenne</span>
                      <span v-else style="color: #ff6b6b;">❌ Basse</span>
                    </template>
                    <span v-else style="color: #999;">Calcul...</span>
                  </div>
                  <div style="font-size: 10px; color: #888; margin-top: auto;">
                    💡 Pic {{ volatilityDuration?.peak_duration_minutes || '?' }}min
                  </div>
                </div>

              </div>
            </div>

            <!-- Straddle Performance Metrics (TÂCHE 5) -->
            <div class="straddle-performance-section">
              <h4>📊 Performance Straddle Simulée</h4>
              <div class="performance-grid">
                <!-- Win Rate -->
                <div class="performance-metric">
                  <div class="metric-label">Win Rate</div>
                  <div v-if="winRate" class="metric-display">
                    <span class="metric-value" :style="{ color: winRateColor }">{{ winRate.win_rate_percentage.toFixed(1) }}%</span>
                    <span class="metric-subtext">({{ winRate.wins }}/{{ winRate.total_trades }} trades)</span>
                  </div>
                  <div v-else class="metric-loading">
                    <span>⏳ Calcul...</span>
                  </div>
                </div>

                <!-- Whipsaw Frequency -->
                <div class="performance-metric">
                  <div class="metric-label">Fréquence Whipsaw</div>
                  <div v-if="whipsawAnalysis" class="metric-display">
                    <span class="metric-value" :style="{ color: whipsawAnalysis.risk_color }">{{ whipsawAnalysis.whipsaw_frequency_percentage.toFixed(1) }}%</span>
                    <span class="metric-subtext">({{ whipsawAnalysis.risk_level }})</span>
                  </div>
                  <div v-else class="metric-loading">
                    <span>⏳ Calcul...</span>
                  </div>
                </div>

                <!-- Offset Optimal -->
                <div class="performance-metric">
                  <div class="metric-label">Offset Optimal</div>
                  <div v-if="offsetOptimal" class="metric-display">
                    <span class="metric-value">{{ offsetOptimal.offset_pips.toFixed(1) }} pips</span>
                    <span class="metric-subtext">(P95: {{ offsetOptimal.percentile_95_wicks.toFixed(1) }})</span>
                  </div>
                  <div v-else class="metric-loading">
                    <span>⏳ Calcul...</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <div v-if="!sliceAnalyses || sliceAnalyses.length === 0" class="no-data">
          <p>Aucune donnée disponible pour l'analyse</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button v-if="!isArchiveMode" class="btn-archive" @click="openArchiveModal">💾 Archiver</button>
        <button class="btn-primary" @click="close">Fermer l'analyse</button>
      </div>
    </div>
  </div>

  <!-- Modale d'archivage -->
  <ArchiveModal
    :show="showArchiveModal"
    archive-type="Volatilité brute"
    :period-start="archivePeriodStart"
    :period-end="archivePeriodEnd"
    :symbol="analysisData?.symbol"
    :timeframe="'M1'"
    :data-json="archiveDataJson"
    @close="showArchiveModal = false"
    @saved="handleArchiveSaved"
  />
</template>

<script setup lang="ts">
import { ref, watch, reactive, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AnalysisResult } from '../stores/volatility'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import { analyzeTop3Slices, calculateBidiParameters } from '../utils/straddleAnalysis'
import type { BidiParameters } from '../utils/straddleAnalysis'
import MetricTooltip from './MetricTooltip.vue'
import ArchiveModal from './ArchiveModal.vue'
import { useStraddleAnalysis, type OptimalOffset, type WinRateMetric, type WhipsawMetric } from '../composables/useStraddleAnalysis'

interface Props {
  isOpen: boolean
  analysisResult: AnalysisResult | null
  isArchiveMode?: boolean
}

interface Emits {
  (e: 'close'): void
}

interface MovementQuality {
  id?: number | null
  symbol: string
  event_type: string
  directional_move_rate: number
  whipsaw_rate: number
  avg_pips_moved: number
  success_rate: number
  quality_score: number
  sample_size: number
  created_at: number  // timestamp en secondes (serde ts_seconds)
  updated_at: number  // timestamp en secondes (serde ts_seconds)
}

interface EntryOffsetMetrics {
  minutes_before_event: number
  sample_count: number
  winning_entries: number
  losing_entries: number
  win_rate: number
  avg_pips_gained: number
  avg_pips_lost: number
  max_pips_gained: number
  max_pips_lost: number
  profit_factor: number
}

interface EntryWindowAnalysisResult {
  symbol: string
  event_type: string
  offsets: EntryOffsetMetrics[]
  optimal_offset: number
  optimal_win_rate: number
  analysis_timestamp: number
  total_events_analyzed: number
}

interface VolatilityDuration {
  peak_duration_minutes: number
  volatility_half_life_minutes: number
  recommended_trade_expiration_minutes: number
  confidence_score: number
  sample_size: number
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const analysisData = ref<any>(null)
const sliceAnalyses = ref<SliceAnalysis[] | null>(null)
const movementQualities = ref<Record<string, MovementQuality>>({})
const volatilityDuration = ref<VolatilityDuration | null>(null)
const tradingPlan = ref<any>(null)

// TÂCHE 5 - Straddle Analysis Metrics
const { 
  isLoading,
  offsetOptimal, 
  winRate, 
  whipsawAnalysis,
  analyzeStraddleMetrics,
  winRateColor
} = useStraddleAnalysis()
const entryWindowAnalysis = reactive({
  symbol: '',
  event_type: '',
  offsets: [] as EntryOffsetMetrics[],
  optimal_offset: -5,
  optimal_win_rate: 0,
  analysis_timestamp: 0,
  total_events_analyzed: 0
} as EntryWindowAnalysisResult)

console.log(`🔍 [INIT] entryWindowAnalysis créé:`, entryWindowAnalysis)
console.log(`🔍 [INIT] Type:`, typeof entryWindowAnalysis)
console.log(`🔍 [INIT] Symbol:`, entryWindowAnalysis.symbol)

/**
 * Helper: construit la clé pour accéder une qualité de mouvement
 */
const getMovementQualityKey = (analysis: SliceAnalysis): string => {
  if (!analysisData.value || analysis.slice.stats.events.length === 0) return ''
  const symbol = analysisData.value.symbol || 'UNKNOWN'
  const eventName = analysis.slice.stats.events[0].event_name
  return `${symbol}_${eventName}`
}

// Fonction pour mettre à jour l'analyse
async function updateAnalysis() {
  if (props.analysisResult && props.isOpen) {
    const result = props.analysisResult
    console.log(`🔄 updateAnalysis() - Symbol: ${result.symbol}`)
    analysisData.value = {
      globalMetrics: result.global_metrics,
      symbol: result.symbol,
      confidence: Math.round(result.confidence_score),
      strategy: 'SCALPING STANDARD',
      bestHours: result.best_hours.slice(0, 3).join(', ')
    }

    // Analyser les TOP 3 tranches 15min
    if (result.stats_15min && result.stats_15min.length > 0) {
      console.log('Analyzing', result.stats_15min.length, 'slices')
      sliceAnalyses.value = analyzeTop3Slices(result.stats_15min)
      console.log('Top 3 slices:', sliceAnalyses.value)

      // Calculer et stocker les paramètres Bidi pour le meilleur moment (rank 1)
      if (sliceAnalyses.value && sliceAnalyses.value.length > 0) {
        const bestSlice = sliceAnalyses.value[0] // Top slice
        tradingPlan.value = calculateBidiParameters(bestSlice.slice, sliceAnalyses.value.map(a => a.slice))
        console.log('Trading Plan (avec tradeDurationMinutes):', tradingPlan.value)
        
        // Calculer la volatilité empirique via Tauri command
        try {
          const stats15min = bestSlice.slice.stats
          console.log('📊 Appel Tauri: analyze_volatility_duration avec Stats15Min:', stats15min)
          
          volatilityDuration.value = await invoke('analyze_volatility_duration', {
            stats: {
              hour: stats15min.hour,
              quarter: stats15min.quarter,
              candle_count: stats15min.candle_count,
              atr_mean: stats15min.atr_mean,
              atr_max: stats15min.atr_max,
              volatility_mean: stats15min.volatility_mean,
              range_mean: stats15min.range_mean,
              body_range_mean: stats15min.body_range_mean,
              shadow_ratio_mean: stats15min.shadow_ratio_mean,
              tick_quality_mean: stats15min.tick_quality_mean,
              volume_imbalance_mean: stats15min.volume_imbalance_mean,
              noise_ratio_mean: stats15min.noise_ratio_mean,
              breakout_percentage: stats15min.breakout_percentage,
              events: stats15min.events || []
            }
          })
          console.log('✅ VolatilityDuration reçu du backend:', volatilityDuration.value)
        } catch (error) {
          console.error('❌ Erreur lors de l\'appel Tauri analyze_volatility_duration:', error)
          // Fallback: utiliser des valeurs heuristiques par défaut
          const atr = bestSlice.slice.stats.atr_mean
          const peakDuration = atr > 0.002 ? 120 : atr > 0.0015 ? 150 : atr > 0.001 ? 180 : 240
          const halfLife = atr > 0.002 ? 45 : atr > 0.0015 ? 60 : atr > 0.001 ? 75 : 90
          volatilityDuration.value = {
            peak_duration_minutes: peakDuration,
            volatility_half_life_minutes: halfLife,
            recommended_trade_expiration_minutes: Math.max(peakDuration, halfLife * 2),
            confidence_score: 50, // Confiance basse avec fallback
            sample_size: bestSlice.slice.stats.candle_count
          }
          console.log('⚠️ Utilisation des valeurs heuristiques:', volatilityDuration.value)
        }
      }

      // Charger les qualités de mouvement pour chaque slice (Phase 1.2)
      if (sliceAnalyses.value && sliceAnalyses.value.length > 0) {
        console.log(`💫 Chargement des qualités pour ${sliceAnalyses.value.length} slices...`)
        for (const analysis of sliceAnalyses.value) {
          if (analysis.slice.stats.events && analysis.slice.stats.events.length > 0) {
            const eventName = analysis.slice.stats.events[0].event_name
            console.log(`  → Appel loadMovementQuality(${result.symbol}, ${eventName})`)
            await loadMovementQuality(result.symbol, eventName)
          }
        }
        console.log(`✅ Chargement des qualités terminé`)

        // Charger l'analyse de fenêtre d'entrée (Phase 1.3)
        const firstEvent = sliceAnalyses.value[0].slice.stats.events[0]
        if (firstEvent) {
          console.log(`🪟 Chargement fenêtre d'entrée pour ${result.symbol}/${firstEvent.event_name}`)
          await loadEntryWindowAnalysis(result.symbol, firstEvent.event_name)
        }
      }
    }
  }
}

// Watcher pour analyser quand analysisResult change
watch(
  () => props.analysisResult,
  () => {
    updateAnalysis()
  }
)

// 🔍 DEBUG: Watcher pour entryWindowAnalysis
watch(
  () => entryWindowAnalysis.symbol,
  (newVal, oldVal) => {
    console.log(`👀 [WATCHER] entryWindowAnalysis.symbol changé: "${oldVal}" → "${newVal}"`)
  }
)

watch(
  () => entryWindowAnalysis,
  (newVal, oldVal) => {
    console.log(`👀 [WATCHER] entryWindowAnalysis objet changé:`, newVal)
  },
  { deep: true }
)

// Watcher pour déclencher l'analyse quand le modal s'ouvre
watch(
  () => props.isOpen,
  (isOpen) => {
    if (isOpen) {
      updateAnalysis()
    }
  }
)

onMounted(() => {
  if (props.isOpen && props.analysisResult) {
    updateAnalysis()
  }
})

// TÂCHE 5 - Calculer les métriques Straddle quand les données changent
watch(
  () => sliceAnalyses.value,
  async (newSlices) => {
    if (newSlices && newSlices.length > 0 && props.analysisResult) {
      console.log('🎯 TÂCHE 5: Analyse des métriques Straddle...')
      
      // Récupérer le meilleur slice (rank 1)
      const bestSlice = newSlices[0]
      if (!bestSlice || !bestSlice.slice || !bestSlice.slice.stats) {
        console.warn('⚠️ Pas de slice ou stats disponibles')
        return
      }

      try {
        const symbol = props.analysisResult.symbol || 'EURUSD'
        
        // TODO: Charger les VRAIES 60 candles (1min) depuis la DB pour cette heure
        // Pour l'instant, appel avec tableau vide pour tester la structure
        const emptyCandles: any[] = []

        // Appeler la nouvelle command Tauri avec VRAIES données
        await analyzeStraddleMetrics(
          symbol,
          0, // hour - TODO: récupérer depuis bestSlice
          emptyCandles // TODO: Charger depuis DB
        )

        console.log('✅ TÂCHE 5 Métriques calculées:')
        console.log('   - Offset:', offsetOptimal.value?.offset_pips, 'pips')
        console.log('   - Win Rate:', winRate.value?.win_rate_percentage, '%')
        console.log('   - Whipsaw:', whipsawAnalysis.value?.whipsaw_frequency_percentage, '%')
      } catch (error) {
        console.error('❌ Erreur calcul TÂCHE 5:', error)
      }
    }
  },
  { deep: true }
)

const close = () => {
  emit('close')
}

// Variables pour l'archivage
const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')

function openArchiveModal() {
  if (!props.analysisResult) return
  
  // Calculer la période depuis les données
  const result = props.analysisResult
  
  if (result.period_start && result.period_end) {
    archivePeriodStart.value = result.period_start
    archivePeriodEnd.value = result.period_end
  } else {
    // Fallback si les dates ne sont pas disponibles
    const now = new Date()
    const oneYearAgo = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
    archivePeriodStart.value = oneYearAgo.toISOString()
    archivePeriodEnd.value = now.toISOString()
  }
  
  // Sérialiser les données d'analyse
  archiveDataJson.value = JSON.stringify({
    analysisResult: result,
    sliceAnalyses: sliceAnalyses.value,
    movementQualities: movementQualities.value,
    volatilityDuration: volatilityDuration.value,
    tradingPlan: tradingPlan.value,
    entryWindowAnalysis: entryWindowAnalysis
  })
  
  showArchiveModal.value = true
}

function handleArchiveSaved() {
  console.log('Archive sauvegardée avec succès')
  showArchiveModal.value = false
}

/**
 * Charge la qualité de mouvement pour une paire et événement
 */
const loadMovementQuality = async (symbol: string, eventType: string) => {
  const key = `${symbol}_${eventType}`
  console.log(`🔄 Chargement qualité mouvement: ${key}`)
  
  if (movementQualities.value[key]) {
    console.log(`✅ Qualité en cache: ${key}`)
    return movementQualities.value[key]
  }

  try {
    console.log(`📤 Appel Tauri: analyze_movement_quality(${symbol}, ${eventType})`)
    console.log(`📝 Paramètres envoyés:`, { symbol, eventType })
    const quality: MovementQuality = await invoke('analyze_movement_quality', {
      symbol,
      eventType
    })
    console.log(`✅ Réponse reçue (type: ${typeof quality}):`, quality)
    console.log(`📝 Type réel réponse:`, Object.prototype.toString.call(quality))
    console.log(`📝 Contenu réponse:`, JSON.stringify(quality))
    console.log(`📝 Clés réponse:`, quality ? Object.keys(quality) : 'null')
    
    if (!quality) {
      console.warn(`⚠️ Réponse null ou undefined reçue!`)
      return null
    }
    
    movementQualities.value[key] = quality
    console.log(`✅ Stockée dans Map avec clé: ${key}`)
    console.log(`📊 Movement Quality [${symbol}/${eventType}]:`, quality)
    return quality
  } catch (error) {
    console.error(`❌ Erreur chargement qualité mouvement:`, error)
    console.error(`❌ Stack trace:`, error instanceof Error ? error.stack : 'N/A')
    return null
  }
}

/**
 * Charge l'analyse de fenêtre d'entrée pour une paire et événement (Phase 1.3)
 */
const loadEntryWindowAnalysis = async (symbol: string, eventType: string) => {
  console.log(`\n🪟 [LOAD START] Chargement fenêtre d'entrée: ${symbol} / ${eventType}`)
  
  try {
    console.log(`📤 Appel Tauri: analyze_entry_window(${symbol}, ${eventType})`)
    const result: EntryWindowAnalysisResult = await invoke('analyze_entry_window', {
      symbol,
      eventType
    })
    console.log(`✅ Réponse reçue:`, result)
    console.log(`📊 Result.symbol = "${result.symbol}"`)
    console.log(`📊 Result.event_type = "${result.event_type}"`)
    console.log(`📊 Result.optimal_offset = ${result.optimal_offset}`)
    
    // Avant assignment
    console.log(`📍 [BEFORE] entryWindowAnalysis.symbol = "${entryWindowAnalysis.symbol}"`)
    console.log(`📍 [BEFORE] Type entryWindowAnalysis:`, typeof entryWindowAnalysis)
    console.log(`📍 [BEFORE] entryWindowAnalysis est Proxy?:`, entryWindowAnalysis.toString().includes('Proxy'))
    
    // Assigner chaque propriété
    console.log(`🔄 Assigning symbol="${result.symbol}"...`)
    entryWindowAnalysis.symbol = result.symbol
    console.log(`✓ symbol assigné. Valeur actuelle: "${entryWindowAnalysis.symbol}"`)
    
    console.log(`🔄 Assigning event_type="${result.event_type}"...`)
    entryWindowAnalysis.event_type = result.event_type
    console.log(`✓ event_type assigné. Valeur actuelle: "${entryWindowAnalysis.event_type}"`)
    
    entryWindowAnalysis.offsets = result.offsets
    entryWindowAnalysis.optimal_offset = result.optimal_offset
    entryWindowAnalysis.optimal_win_rate = result.optimal_win_rate
    entryWindowAnalysis.analysis_timestamp = result.analysis_timestamp
    entryWindowAnalysis.total_events_analyzed = result.total_events_analyzed
    
    // Après assignment
    console.log(`📍 [AFTER] entryWindowAnalysis.symbol = "${entryWindowAnalysis.symbol}"`)
    console.log(`📍 [AFTER] entryWindowAnalysis.event_type = "${entryWindowAnalysis.event_type}"`)
    console.log(`📍 [AFTER] entryWindowAnalysis.optimal_offset = ${entryWindowAnalysis.optimal_offset}`)
    console.log(`📍 [AFTER] Objet complet:`, { ...entryWindowAnalysis })
    
    console.log(`✅ LOAD COMPLETE\n`)
    return result
  } catch (error) {
    console.error(`❌ ERREUR:`, error)
    return null
  }
}


// Fonctions utilitaires de rendu
const formatNumber = (value: number, decimals: number): string => {
  return value.toFixed(decimals)
}

/**
 * Calcule l'heure exacte du meilleur moment
 * Prend le startTime du slice (ex: "14:30-14:45") et ajoute l'offset en minutes
 * Note: offset négatif = avant (ex: -5 = 5 min avant)
 */
const calculateExactTime = (): string => {
  console.log('🔍 calculateExactTime() appelée')
  console.log('  - sliceAnalyses:', sliceAnalyses.value)
  console.log('  - sliceAnalyses length:', sliceAnalyses.value?.length)
  console.log('  - optimal_offset:', entryWindowAnalysis.optimal_offset)
  
  // Vérifier que nous avons les données nécessaires
  if (!sliceAnalyses.value || sliceAnalyses.value.length === 0) {
    console.log('  ❌ sliceAnalyses vide, retour "-"')
    return '-'
  }
  
  const firstSlice = sliceAnalyses.value[0]
  console.log('  - firstSlice:', firstSlice)
  
  const timeRange = firstSlice.slice.startTime // Format: "14:30-14:45"
  const offset = entryWindowAnalysis.optimal_offset // En minutes (négatif = avant)
  
  console.log('  - timeRange:', timeRange)
  console.log('  - offset:', offset)
  
  // Extraire l'heure de début (avant le tiret)
  const startTimeStr = timeRange.split('-')[0] // "14:30"
  console.log('  - startTimeStr:', startTimeStr)
  
  // Parser le startTime (format "14:30" avec deux points)
  const timeMatch = startTimeStr.match(/(\d+):(\d+)/)
  if (!timeMatch) {
    console.log('  ❌ Regex ne match pas, retour:', timeRange)
    return timeRange
  }
  
  let hours = parseInt(timeMatch[1], 10)
  let minutes = parseInt(timeMatch[2], 10)
  
  console.log('  - avant calcul: hours=', hours, 'minutes=', minutes)
  
  // Ajouter l'offset (négatif = avant, positif = après)
  minutes += offset
  
  // Gérer le débordement des heures
  while (minutes < 0) {
    minutes += 60
    hours -= 1
  }
  while (minutes >= 60) {
    minutes -= 60
    hours += 1
  }
  
  // Gérer le débordement des jours (0-23 heures)
  if (hours < 0) hours += 24
  if (hours >= 24) hours -= 24
  
  const result = `${hours.toString().padStart(2, '0')}h${minutes.toString().padStart(2, '0')}`
  console.log('  ✅ Résultat:', result)
  
  // Formater le résultat
  return result
}

const getStatusClass = (metrics: any): string => {
  if (!metrics) return 'unknown'
  const confidence = metrics.mean_atr // Utiliser ATR comme proxy
  if (confidence > 0.0015) return 'excellent'
  if (confidence > 0.001) return 'good'
  if (confidence > 0.0005) return 'acceptable'
  return 'poor'
}

const getStatusText = (metrics: any): string => {
  const classes = ['Excellent', 'Good', 'Acceptable', 'Poor']
  return classes[['excellent', 'good', 'acceptable', 'poor'].indexOf(getStatusClass(metrics))]
}

const getMetricClass = (value: number, threshold1: number, threshold2: number): string => {
  if (value >= threshold2) return 'excellent'
  if (value >= threshold1) return 'good'
  return 'poor'
}

const getMetricStatus = (value: number, threshold: number): string => {
  return value >= threshold ? 'ok' : 'low'
}

const getMetricStatusText = (value: number, threshold: number): string => {
  return value >= threshold ? '✅ OK' : '❌ TOO LOW'
}

const getNoiseStatus = (value: number): string => {
  if (value < 2.0) return 'excellent'
  if (value < 3.0) return 'good'
  return 'poor'
}

const getNoiseStatusText = (value: number): string => {
  if (value < 2.0) return '✅ Signal pur'
  if (value < 3.0) return '🔵 Acceptable'
  return '❌ Chaotique'
}

const getImbalanceClass = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return 'excellent'
  if (distance > 0.5) return 'good'
  return 'poor'
}

const getImbalanceStatus = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return 'ok'
  return 'low'
}

const getImbalanceStatusText = (value: number): string => {
  const distance = Math.abs(value - 1.0)
  if (distance > 1.0) return `✅ Tendance marquee`
  if (distance > 0.5) return `🔵 Moderee`
  return `❌ Equilibree`
}

// Direction Strength helpers (replaces Volume Imbalance for Forex)
const getDirectionStrengthClass = (value: number): string => {
  const strengthPercent = value * 100
  if (strengthPercent >= 20) return 'excellent'
  if (strengthPercent >= 10) return 'good'
  if (strengthPercent >= 5) return 'acceptable'
  return 'poor'
}

const getDirectionStrengthStatus = (value: number): string => {
  const strengthPercent = value * 100
  if (strengthPercent >= 20) return 'ok'
  if (strengthPercent >= 5) return 'warning'
  return 'low'
}

const getDirectionStrengthStatusText = (value: number): string => {
  const strengthPercent = value * 100
  if (strengthPercent >= 20) return `✅ Forte direction`
  if (strengthPercent >= 10) return `🔵 Bonne direction`
  if (strengthPercent >= 5) return `🟠 Direction faible`
  return `❌ Peu de direction`
}

const getScoreSeverity = (score: number): string => {
  if (score >= 75) return 'excellent'
  if (score >= 50) return 'good'
  if (score >= 25) return 'acceptable'
  return 'poor'
}

const getRankClass = (rank: number): string => {
  return `rank-${rank}`
}

/**
 * Retourne la classe CSS pour colorer le score de qualité
 */
const getQualityScoreClass = (score: number): string => {
  if (score >= 7.0) return 'high-quality'
  if (score >= 5.0) return 'medium-quality'
  if (score >= 3.0) return 'low-quality'
  return 'avoid-quality'
}

/**
 * Retourne le texte de recommandation basé sur le score
 */
const getQualityRecommendation = (score: number): string => {
  if (score >= 7.0) return 'TRADE ✅'
  if (score >= 5.0) return 'CAUTION 🔵'
  if (score >= 3.0) return 'CAUTION 🔵'
  return 'AVOID ❌'
}
</script>

<style scoped lang="css">
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: #1a1f2e;
  border: 2px solid #2d3748;
  border-radius: 12px;
  width: 100%;
  max-width: 1600px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
  color: #e2e8f0;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border-bottom: 2px solid #2d3748;
  background: linear-gradient(135deg, #1a1f2e 0%, #2d3748 100%);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title .icon {
  font-size: 24px;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #cbd5e0;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.modal-section {
  padding: 24px;
  border-bottom: 1px solid #2d3748;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.section-header .icon {
  font-size: 20px;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Diagnostic Grid */
.diagnostic-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.diagnostic-item {
  background: rgba(45, 55, 72, 0.5);
  border: 1px solid #2d3748;
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diagnostic-item .label {
  font-size: 12px;
  color: #a0aec0;
  text-transform: uppercase;
  font-weight: 600;
}

.diagnostic-item .value,
.diagnostic-item .status-badge {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
}

.status-badge {
  padding: 4px 8px;
  border-radius: 4px;
  width: fit-content;
}

.status-badge.excellent {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.status-badge.good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}

.status-badge.acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.status-badge.poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Slices Container */
.slices-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.slice-card {
  background: rgba(45, 55, 72, 0.4);
  border: 2px solid #2d3748;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s;
}

.slice-card:hover {
  background: rgba(45, 55, 72, 0.6);
  border-color: #4a5568;
}

.slice-card.rank-1 {
  border-color: #fbbf24;
  background: rgba(251, 191, 36, 0.05);
}

.slice-card.rank-2 {
  border-color: #a78bfa;
  background: rgba(167, 139, 250, 0.05);
}

.slice-card.rank-3 {
  border-color: #f97316;
  background: rgba(249, 115, 22, 0.05);
}

/* Slice Header */
.slice-header {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid #2d3748;
  justify-content: space-between;
}

.rank-badge {
  width: 80px;
  height: 80px;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 24px;
  gap: 4px;
}

.rank-badge.rank-1 {
  background: rgba(251, 191, 36, 0.15);
  border: 2px solid #fbbf24;
  color: #fbbf24;
}

.rank-badge.rank-2 {
  background: rgba(167, 139, 250, 0.15);
  border: 2px solid #a78bfa;
  color: #a78bfa;
}

.rank-badge.rank-3 {
  background: rgba(249, 115, 22, 0.15);
  border: 2px solid #f97316;
  color: #f97316;
}

.rank-number {
  font-size: 28px;
}

.rank-medal {
  font-size: 20px;
}

.slice-time {
  flex: 1;
}

.slice-time .time {
  font-size: 18px;
  font-weight: 700;
  color: #fff;
}

.slice-time .score {
  font-size: 14px;
  margin-top: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  width: fit-content;
}

.score-excellent {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.score-good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}

.score-acceptable {
  background: rgba(234, 179, 8, 0.2);
  color: #facc15;
}

.score-poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Metrics Section */
.metrics-section {
  margin-bottom: 20px;
}

.metrics-section h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 12px;
}

.metric-item {
  background: rgba(45, 55, 72, 0.3);
  border: 1px solid #2d3748;
  border-radius: 8px;
  padding: 12px;
}

.metric-name {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: #a0aec0;
  text-transform: uppercase;
  margin-bottom: 6px;
}

.metric-values {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 13px;
}

.metric-values .value15 {
  font-weight: 700;
  min-width: 80px;
  font-family: monospace;
}

.metric-values .valueglobal {
  color: #a0aec0;
  font-family: monospace;
}

.metric-values .separator {
  color: #4a5568;
}

.metric-values .threshold {
  color: #f59e0b;
  font-weight: 600;
}

.metric-values .value15.excellent {
  color: #86efac;
}

.metric-values .value15.good {
  color: #93c5fd;
}

.metric-values .value15.poor {
  color: #fca5a5;
}

.status {
  display: inline-block;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
}

.status.ok {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

.status.low {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* Golden Combos */
.combos-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: 8px;
}

.combos-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #86efac;
}

.combos-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.combo-item {
  background: rgba(45, 55, 72, 0.4);
  border-left: 4px solid #22c55e;
  border-radius: 6px;
  padding: 12px;
}

.combo-item.confidence-jackpot {
  border-left-color: #fbbf24;
  background: rgba(251, 191, 36, 0.08);
}

.combo-item.confidence-excellent {
  border-left-color: #a78bfa;
  background: rgba(167, 139, 250, 0.08);
}

.combo-item.confidence-bon {
  border-left-color: #3b82f6;
  background: rgba(59, 130, 246, 0.08);
}

.combo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.combo-name {
  font-weight: 700;
  color: #fff;
}

.combo-confidence {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.combo-confidence.badge-jackpot {
  background: #fbbf24;
  color: #000;
}

.combo-confidence.badge-excellent {
  background: #a78bfa;
  color: #fff;
}

.combo-confidence.badge-bon {
  background: #3b82f6;
  color: #fff;
}

.combo-description {
  margin: 6px 0;
  font-size: 13px;
  color: #cbd5e0;
  line-height: 1.4;
}

.combo-metrics {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: #a0aec0;
}

.combo-metrics .metric {
  display: flex;
  gap: 4px;
}

/* Pièges */
.traps-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 8px;
}

.traps-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #fca5a5;
}

.traps-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.trap-item {
  background: rgba(45, 55, 72, 0.4);
  border-left: 4px solid #ef4444;
  border-radius: 6px;
  padding: 12px;
}

.trap-item.severity-critique {
  border-left-color: #dc2626;
  background: rgba(220, 38, 38, 0.08);
}

.trap-item.severity-haute {
  border-left-color: #ef4444;
  background: rgba(239, 68, 68, 0.08);
}

.trap-item.severity-moyenne {
  border-left-color: #f97316;
  background: rgba(249, 115, 22, 0.08);
}

.trap-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.trap-name {
  font-weight: 700;
  color: #fff;
}

.trap-severity {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.trap-severity.badge-critique {
  background: #dc2626;
  color: #fff;
}

.trap-severity.badge-haute {
  background: #ef4444;
  color: #fff;
}

.trap-severity.badge-moyenne {
  background: #f97316;
  color: #fff;
}

.trap-description {
  margin: 6px 0;
  font-size: 13px;
  color: #cbd5e0;
  line-height: 1.4;
}

.trap-detail {
  margin: 8px 0;
  font-size: 12px;
  color: #a0aec0;
  background: rgba(0, 0, 0, 0.2);
  padding: 6px 8px;
  border-radius: 4px;
  font-family: monospace;
}

.trap-recommendation {
  margin-top: 8px;
  font-size: 12px;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.1);
  padding: 6px 8px;
  border-radius: 4px;
}

/* Plan Section */
.plan-section {
  padding: 16px;
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 8px;
}

.plan-section h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #93c5fd;
}

.plan-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.plan-item {
  background: rgba(45, 55, 72, 0.4);
  border: 1px solid #2d3748;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.plan-item .label {
  font-size: 11px;
  font-weight: 700;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.plan-item .value {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  font-family: monospace;
}

.plan-item .value.detailed {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
}

.plan-item .value.detailed .secondary {
  font-size: 11px;
  color: #a0aec0;
}

/* Modal Footer */
.modal-footer {
  padding: 20px 24px;
  border-top: 1px solid #2d3748;
  background: rgba(45, 55, 72, 0.3);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-primary {
  padding: 10px 20px;
  background: #3b82f6;
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-archive {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  margin-right: 10px;
}

.btn-archive:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

/* No Data */
.no-data {
  text-align: center;
  padding: 40px;
  color: #a0aec0;
}

/* Bouton Bidi Parameters */
.btn-bidi-params {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: linear-gradient(135deg, #ffc107 0%, #ffb300 100%);
  color: #0d1117;
  border: 1px solid #ffb300;
  border-radius: 6px;
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-bidi-params:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(255, 193, 7, 0.4);
  background: linear-gradient(135deg, #ffb300 0%, #ffa500 100%);
}

.btn-bidi-params:active {
  transform: translateY(0);
}

.btn-bidi-params .btn-icon {
  font-size: 16px;
}

.btn-bidi-params .btn-text {
  font-size: 12px;
}

/* Scrollbar */
.modal-content::-webkit-scrollbar {
  width: 8px;
}

.modal-content::-webkit-scrollbar-track {
  background: rgba(45, 55, 72, 0.3);
}

.modal-content::-webkit-scrollbar-thumb {
  background: #4a5568;
  border-radius: 4px;
}

.modal-content::-webkit-scrollbar-thumb:hover {
  background: #718096;
}

/* Movement Quality Section */
.movement-quality-section {
  background: linear-gradient(135deg, rgba(29, 78, 216, 0.05) 0%, rgba(99, 102, 241, 0.05) 100%);
  border-left: 3px solid #6366f1;
  padding: 14px;
  border-radius: 6px;
  margin-top: 12px;
}

.movement-quality-section h4 {
  color: #e0e7ff;
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 10px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.quality-card {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 10px;
}

.quality-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
}

.quality-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  background: rgba(30, 30, 45, 0.8);
  border-radius: 4px;
  border: 1px solid rgba(75, 85, 99, 0.3);
}

.quality-label {
  font-size: 11px;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  font-weight: 500;
}

.quality-value {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
}

.quality-score {
  font-size: 18px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  text-align: center;
}

.quality-score.high-quality {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.2) 0%, rgba(22, 163, 74, 0.2) 100%);
  color: #86efac;
  border: 1px solid rgba(34, 197, 94, 0.4);
}

.quality-score.medium-quality {
  background: linear-gradient(135deg, rgba(251, 146, 60, 0.2) 0%, rgba(234, 88, 12, 0.2) 100%);
  color: #fed7aa;
  border: 1px solid rgba(251, 146, 60, 0.4);
}

.quality-score.low-quality {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.2) 0%, rgba(220, 38, 38, 0.2) 100%);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.quality-score.avoid-quality {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.2) 0%, rgba(139, 92, 246, 0.2) 100%);
  color: #d8b4fe;
  border: 1px solid rgba(168, 85, 247, 0.4);
}

.quality-recommendation {
  font-size: 11px;
  font-weight: 600;
  color: #fbbf24;
  margin-top: 2px;
}

.quality-loading {
  text-align: center;
  padding: 12px;
  color: #64748b;
  font-size: 12px;
  font-style: italic;
}

/* Straddle Performance Section (TÂCHE 5) */
.straddle-performance-section {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.05) 0%, rgba(168, 85, 247, 0.05) 100%);
  border-left: 3px solid #a855f7;
  padding: 14px;
  border-radius: 6px;
  margin-top: 12px;
}

.straddle-performance-section h4 {
  color: #e9d5ff;
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 10px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.performance-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
}

.performance-metric {
  background: rgba(30, 30, 45, 0.6);
  border: 1px solid rgba(168, 85, 247, 0.2);
  border-radius: 6px;
  padding: 12px;
  text-align: center;
}

.metric-label {
  font-size: 11px;
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  margin-bottom: 8px;
  font-weight: 600;
}

.metric-display {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-value {
  font-size: 18px;
  font-weight: bold;
  line-height: 1;
}

.metric-subtext {
  font-size: 10px;
  color: #94a3b8;
}

.metric-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 40px;
  color: #64748b;
  font-size: 12px;
  font-style: italic;
}
</style>
