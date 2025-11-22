# 📋 PROJET : Analyses Historiques - Volatility Analyzer for Straddle

## 🎯 OBJECTIF PRINCIPAL

Créer une **application desktop d'analyse de volatilité historique** permettant de **paramétrer automatiquement un robot de trading Straddle** (robot "Bidi") en identifiant:

1. **QUAND trader** → Heures de forte volatilité exploitable
2. **COMMENT trader** → Offset, SL, TP, Trailing Stop optimisés
3. **COMBIEN de temps** → Durée de maintien position basée sur decay volatilité
4. **QUELS événements** → Quelles annonces économiques vraiment impactent la volatilité

---

## 🎯 OBJECTIFS SECONDAIRES

### 1. Intelligence Prédictive
- Apprendre des patterns historiques de volatilité
- Prédire volatilité avant annonce économique
- Recommander setup optimal (Optimal/Good/Cautious/Risky)

### 2. Gestion Multi-Paires
- Analyser corrélation: Événement × Paire × Volatilité
- Identifier paires "complices" (react bien au même event)
- Classifier events par impact paire

### 3. Optimalisation Continue
- Tracker perfs réelles vs prédictions
- Ajuster paramètres dynamiquement
- Archive analyses pour ML training

### 4. Facilité d'Utilisation
- Interface intuitive pour trader (pas de coder)
- Export automatique paramètres vers Bidi
- Visualisations claires (heatmaps, scores, recommendations)


## ⚙️ CONTRAINTES TECHNIQUES

### 1. Performance
- ✅ Import CSV 1-10 ans historique < 30 secondes
- ✅ Calcul analyse < 5 secondes
- ✅ Affichage heatmap 50×20 events/pairs < 1 seconde
- ✅ RAM usage < 500 MB même avec 10 ans données

### 2. Fiabilité
- ✅ Gestion erreurs gracieuse (pas de crash)
- ✅ Validation données avant traitement
- ✅ Logs détaillés pour debugging
- ✅ Recovery auto après erreur mineure

### 3. Compatibilité
- ✅ Linux (Fedora, Ubuntu)
- ✅ Formats CSV variés (MetaTrader, TradingView, Dukascopy)

### 4. Code Quality
- ✅ Respect `.clinerules` (fichier taille, pas unwrap, architecture DAG)
- ✅ 80%+ test coverage (105 tests minimum)
- ✅ 0 clippy warnings
- ✅ Zéro code mort

---

## 📋 OBLIGATIONS STRATÉGIQUES

### Pour le Straddle
1. **ATR Basé** → SL/TP et offset = f(ATR local)
2. **Noise Aware** → Filtrer events avec Noise Ratio > 3.0
3. **Body% Filter** → Ignorer heures avec directionnalité < 20%
4. **Event Correlated** → Volatilité haute doit = event HIGH
5. **Duration Adaptive** → Trade duration = f(ATR + event_type)

### Pour le Bidi Robot
1. **Export API** → Données toujours en JSON standardisé
2. **Confidence Score** → Chaque recommandation avec score 0-100
3. **Risk Percent = 1.0** → Immuable, pas de modificateur
4. **Trailing Stop Dynamic** → Coefficient 1.5-2.5 basé volatilité
5. **Event Time Exact** → À la seconde (H:MM:SS)

### Pour l'Utilisateur
1. **Transparence** → Chaque nombre = expliqué/justifié
2. **Facilité** → Zéro configuration = defaults smart
3. **Visibilité** → Dashboard clair avec alertes
4. **Validation** → Avant import, valider données
5. **Historique** → Archive analyses pour comparaison

---

## 🔒 SÉCURITÉ & CONFIDENTIALITÉ

### Données Utilisateur
- ✅ Pas de données personnelles collectées
- ✅ Données locales (pas de cloud)
- ✅ Pas de tracking ou telemetry
- ✅ Historique local en SQLite

### Validation Entrées
- ✅ CSV parsé strictement (pas d'injection)
- ✅ Datetimes validés (pas de parsing arbitraire)
- ✅ Nombres rangés (min/max checks)
- ✅ Pas d'exécution de code utilisateur

---

## 📈 MÉTRIQUES DE SUCCÈS

### Business Metrics
- **Accuracy**: Volatilité prédite vs réelle < 10% error
- **Win Rate**: Setup Straddle gagne > 60% des cas
- **Profitability**: Gain moyen > 2R (Risk:Reward)
- **Adoption**: Bidi robot utilise données pour > 80% des trades

### Technical Metrics
- **Uptime**: > 99.5%
- **Performance**: Analyse < 5sec, Import < 30sec
- **Stability**: 0 crashes en 100 heures utilisation
- **Coverage**: Tests > 80% du code

---

## 🚫 ANTI-OBJECTIFS (Ce qu'on NE DOIT PAS faire)

1. ❌ Fournir des signaux de trading directs
   - L'app = analyse historique, pas predictor temps réel

2. ❌ Simuler performances futures
   - "Gain garanti 50R" = Mensonge

3. ❌ Remplacer la décision humaine
   - Utilisateur reste responsable

5. ❌ Permettre backtesting
   - Autre app; trop complexe pour MVP
