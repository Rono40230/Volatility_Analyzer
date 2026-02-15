# 📊 Analyses Historiques - Volatility Analyzer

## 🎯 À quoi sert cette application ?

**Analyses Historiques** est un outil d'aide à la décision conçu spécifiquement pour le **News Trading** (trading d'annonces économiques) sur le Forex et les Indices.

Son objectif unique est de repérer les forts mouvements de volatilité propices au scalping en straddle (2 jambes ouvertes symétriquement au même point d'entrée)

Elle répond à la question fondamentale : *"Comment cet actif réagit-il habituellement à cet événement économique précis ?"*

---

## ❓ À quelles questions répond-elle ?

L'application permet de répondre précisément aux questions suivantes avant chaque annonce économique :

1.  **Faut-il trader cet événement ?**
    *   *Est-ce que ça bouge assez ?* (Volatilité suffisante)
    *   *Est-ce que c'est propre ?* (Ratio de bruit faible, peu de mèches)
    *   *Est-ce que c'est dangereux ?* (Risque de "Whipsaw" / faux départ)

2.  **Quelle est la meilleure paire ?**
    *   *Sur quel actif l'impact est-il le plus fort et le plus directionnel ?* (Comparaison EURUSD vs GBPUSD vs GOLD...)


---

## � Gestion Intelligente des Actifs & Coûts

L'application ne traite pas le Bitcoin comme l'Euro-Dollar. Elle intègre une intelligence financière pour adapter les calculs à chaque classe d'actif :

### 1. Détection Automatique des Classes d'Actifs
L'application reconnaît automatiquement le type d'actif importé :
*   **Forex Majeur** (ex: EURUSD) : Calcul en Pips (0.0001).
*   **Forex JPY & Exotiques** (ex: USDJPY, USDHUF) : Calcul adapté (0.01).
*   **Or & Métaux** (ex: XAUUSD) : Calcul standardisé (0.1$ = 1 pip).
*   **Indices** (ex: DAX, US30, NAS100) : Calcul en Points.
*   **Cryptos** (ex: BTCUSD, ETHUSD, DOGE...) : Calcul en Points (1$ = 1 point).
*   **Matières Premières** (ex: WTI, BRENT, NGAS) : Calcul spécifique (0.01 ou 0.001).

### 2. Coûts de Trading Réalistes (Spread + Slippage)
Pour que les paramètres (Offset, SL) soient utilisables dans la vraie vie, l'application inclut automatiquement les coûts de trading moyens dans ses calculs :
*   **Crypto** : Spread large + Slippage élevé inclus.
*   **Indices** : Spread variable selon l'indice.
*   **Forex** : Spread serré.
*   *Exemple :* Un Offset calculé pour le BTC inclura automatiquement une marge de sécurité (~60 points) bien supérieure à celle de l'EURUSD (~3.5 pips).

---

## �🛠️ Workflow par Onglet

L'application est organisée en 6 onglets principaux suivant le flux de travail logique d'un trader.

### 1. 📅 Planning (Feuille de Route Hebdomadaire)
*C'est votre tableau de bord opérationnel pour la semaine à venir.*
*   **Fonctionnement :** Affiche le calendrier économique de la semaine en cours (synchronisé depuis Forex Factory ou importé manuellement).
*   **Cartes Événements :** Chaque événement futur est affiché avec :
    *   Un badge d'impact (High/Medium).
    *   Un badge indiquant le nombre d'occurrences historiques disponibles (📚).
*   **Workflow d'Analyse :**
    1.  Sélectionnez la paire à trader directement sur la carte de l'événement.
    2.  Cliquez sur le bouton **"📊 Analyser"**.
    3.  Une fenêtre s'ouvre avec l'analyse historique complète et les paramètres optimaux (Offset, SL, TP).
    4.  Si une stratégie "Simultanée" (Double Straddle) est possible, les paramètres additionnels s'affichent.

### 2. 📥 Calendrier (Import Hub)
*Le centre de gestion de vos données.*
*   **Import Calendrier :** Deux modes d'importation :
    *   **Historique Général :** Importez un gros fichier CSV (ex: 2018-2024) pour nourrir les statistiques.
    *   **Planning Hebdo :** Cochez la case "Planning Hebdo" pour importer le fichier de la semaine en cours. Cela remplace automatiquement l'ancien planning sans toucher à votre historique général.
*   **Import Paires :** Importez vos données OHLC (M1) pour permettre les calculs de volatilité.
*   **Outils de Nettoyage :**
    *   **Événements Orphelins :** Détecte et supprime les événements liés à des devises que vous ne tradez pas.
    *   **Événements Rares :** Nettoie les événements qui n'apparaissent qu'une seule fois dans l'histoire pour ne pas polluer les stats.
    *   **Nettoyage CSV :** Outil dédié pour reformater les CSV européens (point-virgule) en format standard.

### 3. 🔥 Heatmap de Corrélation
*Pour identifier les opportunités en un coup d'œil.*
*   **Fonctionnement :** Affiche une matrice visuelle (Événements x Paires).
*   **Lecture :**
    *   🟥 **Rouge** : Impact violent, forte volatilité.
    *   🟩 **Vert** : Impact faible ou nul.
*   **Action :** Cliquez sur une case pour lancer une analyse détaillée.

### 4. 📊 Volatilité (Analyse Technique)
*Pour analyser le comportement structurel d'une paire.*
*   **Fonctionnement :** Sélectionnez une paire (ex: EURUSD). L'appli analyse chaque heure de la journée.
*   **Résultat :**
    *   **Tableau Horaire :** Volatilité moyenne, bruit, mouvements pour chaque heure.
    *   **Analyse Bidi :** Cliquez sur une heure pour voir l'analyse bidirectionnelle détaillée (probabilités de mouvement haussier vs baissier).
    *   **Métriques Avancées :**
        *   **Peak Delay :** Temps moyen avant d'atteindre le point haut/bas de l'impulsion.
        *   **Decay Profile :** Vitesse à laquelle la volatilité retombe après l'annonce.
        *   **Whipsaw Detection :** Probabilité de faux départ (mèche inverse avant le vrai mouvement).

### 5. 🧪 Backtest & Archives
*Pour valider et sauvegarder.*
*   **Archives :** Retrouvez toutes vos analyses sauvegardées depuis l'onglet Planning ou Volatilité.
*   **Backtest :** Rejouez les événements passés tick par tick avec vos paramètres (Offset, SL, TP) pour vérifier la robustesse de la stratégie (Win Rate, Drawdown, Equity Curve).

### 6. 🖨️ Exports
*Pour générer vos fiches de trading.*
*   Générez des rapports PDF professionnels incluant :
    *   Les paramètres de trading (Entrée, SL, TP).
    *   Les statistiques de volatilité.
    *   Le classement des meilleures opportunités.

---

## 📥 Importation de Données

Pour fonctionner, l'application a besoin de deux types de données :

1.  **Données de Prix (Bougies M1) :**
    *   Fichiers CSV exportés depuis MT4/MT5 ou Dukascopy.
    *   Format attendu : Date, Open, High, Low, Close, Volume.
    *   *Astuce :* L'application nettoie et convertit automatiquement les formats courants.

2.  **Calendriers Économiques (CSV) :**
    *   **Source recommandée :** Forex Factory.
    *   **Format :** Date, Time, Currency, Impact, Event Name.
    *   *Workflow :* Importez un gros historique une fois pour toutes, puis mettez à jour le "Planning Hebdo" chaque semaine.

---

## 🚀 Installation & Démarrage

### Prérequis
- **Node.js** (v18+)
- **Rust** (v1.70+)
- **Tauri CLI**

### Commandes
```bash
# Installation des dépendances
npm install

# Lancement en mode développement
npm run tauri dev

# Compilation pour production
npm run tauri build
```

---

## 🛡️ Confidentialité
Cette application fonctionne **100% en local**. Aucune donnée (ni vos CSV, ni vos analyses) n'est envoyée sur un serveur externe. Tout est stocké dans une base de données SQLite sur votre machine.
