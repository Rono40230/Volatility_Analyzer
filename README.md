# 📊 Analyses Historiques - Analyseur de Volatilité Forex

Application d'analyse de volatilité pour le trading forex, corrélant les mouvements de prix avec les événements économiques.

## 🎯 Fonctionnalités

- **Analyse de volatilité** : Calcul des statistiques horaires et par session de trading
- **Corrélation événements économiques** : Impact des annonces économiques sur la volatilité
- **Import de données** : Support de multiples formats CSV (TradingView, MetaTrader, etc.)
- **Calendrier économique** : Import et gestion d'événements économiques
- **Machine Learning** : Prédiction de volatilité basée sur l'historique
- **Interface moderne** : Vue 3 + Tauri 2.0 pour une expérience desktop native

## 🏗️ Architecture

### Backend (Rust + Tauri 2.0)
```
src-tauri/
├── src/
│   ├── commands/          # Commandes Tauri exposées au frontend
│   │   ├── correlation/   # Analyse de corrélation événements/prix
│   │   ├── calendar_commands.rs
│   │   ├── economic_commands.rs
│   │   ├── pair_data_commands.rs
│   │   ├── session_commands.rs
│   │   └── volatility_commands.rs
│   ├── services/          # Logique métier
│   │   ├── metrics/       # Calculs de métriques
│   │   ├── pair_data/     # Parsers de données
│   │   ├── volatility/    # Analyse de volatilité
│   │   ├── event_correlation.rs
│   │   ├── ml_predictor.rs
│   │   └── ml_trainer.rs
│   ├── models/            # Structures de données
│   └── db/                # Schéma et migrations Diesel
└── migrations/            # Migrations SQL
```

### Frontend (Vue 3 + TypeScript)
```
src/
├── components/
│   ├── AnalysisPanel.vue
│   ├── CalendarView.vue
│   ├── EventCorrelationView.vue
│   ├── SessionAnalysisView.vue
│   └── PairDataImport.vue
└── stores/
    └── volatility.ts      # État Pinia
```

## 📏 Standards de Qualité

Le projet suit des règles strictes définies dans `.clinerules` :

- **Commands** : max 200 lignes
- **Services** : max 300 lignes  
- **Models** : max 150 lignes
- **Aucun** `.unwrap()` ou `.expect()` hors des tests
- Architecture modulaire avec séparation des responsabilités

### Validation
```bash
make check-rules    # Vérification des règles de taille et anti-patterns
make validate       # Tests complets
```

## 🚀 Installation

### Prérequis
- Rust 1.70+
- Node.js 18+
- SQLite 3
- Dépendances système (Linux Fedora) :

```bash
./install_deps_fedora.sh
```

### Build
```bash
# Installation des dépendances
npm install
cd src-tauri && cargo build

# Mode développement
npm run tauri dev

# Build production
npm run tauri build
```

## 📦 Base de données

Le projet utilise **Diesel ORM** avec SQLite :

```bash
# Exécuter les migrations
cd src-tauri
diesel migration run

# Créer une nouvelle migration
diesel migration generate nom_de_la_migration
```

### Schéma
- `candles` : Données OHLCV
- `hourly_stats` : Statistiques horaires de volatilité
- `calendar_events` : Événements économiques
- `correlated_events` : Corrélations événements/volatilité

## 🧪 Tests

```bash
# Tests Rust
cd src-tauri
cargo test

# Tests d'intégration
./run-tests.sh
```

## 📊 Import de données

### Paires de devises (CSV)
Formats supportés :
- TradingView
- MetaTrader 4/5
- Dukascopy
- Format personnalisé

```bash
# Via l'interface : PairDataImport.vue
# Ou via script Python
python convert_csv.py fichier.csv
```

### Calendrier économique
```bash
# Import CSV
python convert_calendar.py sample_economic_events.csv

# Via l'interface : CalendarView.vue
```

## 🛠️ Outils de développement

- `make check-rules` : Validation des .clinerules
- `make validate` : Tests complets
- `./dev-watch.sh` : Mode développement avec auto-reload
- `scripts/check-unwrap.sh` : Détection des .unwrap()/.expect()

## 📈 Récent nettoyage (Nov 2025)

✅ **2221 lignes supprimées** :
- 4 fichiers dupliqués supprimés (1831 lignes)
- 9 fichiers refactorisés (390 lignes optimisées)
- 6 bugs critiques corrigés
- Conformité totale aux .clinerules

## 🔧 Configuration

### .clinerules
Définit les limites de taille et les anti-patterns à éviter.

### Makefile
Commandes disponibles :
- `make check-rules` : Validation complète
- `make validate` : Tests + validation
- `make clean` : Nettoyage des artefacts

## 📝 Licence

[À définir]

## 👤 Auteur

Rono40230

---

**Note** : Pour compiler le projet, assurez-vous d'avoir installé les dépendances système requises (webkit2gtk-4.1, javascriptcoregtk-4.1) via le script `install_deps_fedora.sh`.
