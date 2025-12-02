# 🚀 Phase 7: Enrichissement Rétrospectif - Plan Complet

**Objectif**: Ajouter 5 nouvelles métriques d'analyse rétrospective pour améliorer l'intelligence du Straddle backtest.

**Contexte**: L'app n'est PAS un système de prédiction temps-réel, mais un analyseur historique. Les 5 nouvelles métriques enrichissent la compréhension des événements passés pour guider le robot Bidi.

---

## 📋 Vue d'Ensemble des 5 Métriques

| # | Métrique | But | Données Clé | Impact UI |
|---|----------|-----|-----------|----------|
| 1 | **Peak Delay** | Quand arrive le mouvement? | `+2.3 min` (delay après annonce) | Tableau Peak Delays |
| 2 | **Whipsaw Root Cause** | Pourquoi whipsaw? | `8% early, 20% late` | Analyse cause |
| 3 | **Entry Timing** | Meilleur moment d'entrée? | `T-5: 52% win, +18p avg` | Matrice profitabilité |
| 4 | **Decay Profile** | Combien de temps volatilité? | `Decay: 2.7p/min, timeout: 25m` | Courbe + recommandations |
| 5 | **Directional Bias** | Asymétrie UP/DOWN? | `+0.70 bias, 35% asymmetry` | Score + warnings |

---

## 🏗️ Architecture Générale

```
PHASE 7 = 5 phases parallèles + intégration finale

Phase 7a: Rust Backend (3 jours)
├─ 7a1: Peak Delay Analyzer (volatility_analyzer.rs)
├─ 7a2: Whipsaw Root Cause (whipsaw_detector.rs)
├─ 7a3: Entry Timing Analyzer (nouveau service)
├─ 7a4: Decay Profile (volatility_duration_analyzer.rs)
└─ 7a5: Directional Bias (nouveau service)

Phase 7b: Database (1 jour)
├─ 7b1: Migrations (add columns + new tables)
└─ 7b2: Seed data (valeurs de base pour tests)

Phase 7c: Frontend Components (2 jours)
├─ 7c1: EventCorrelationView (restructure avec 2 groupes onglets)
├─ 7c2: PeakDelayAnalysis.vue (NEW)
├─ 7c3: DecayProfileView.vue (NEW)
├─ 7c4: EntryTimingProfitability.vue (NEW)
├─ 7c5: DirectionalBiasView.vue (NEW)
└─ 7c6: WhipsawRootCauseView.vue (NEW)

Phase 7d: Formules & Documentation (1 jour)
├─ 7d1: formules.ts (add 5 new formules)
└─ 7d2: Explications littérales (French, no jargon)

Phase 7e: Testing & Validation (1 jour)
├─ 7e1: Unit tests (services)
├─ 7e2: Integration tests (DB + services)
├─ 7e3: UI tests (components render)
└─ 7e4: Regression tests (no breaks)

TOTAL: ~8 jours (peut être parallélisé = 5 jours réels)
```

---

# 📅 PHASES DÉTAILLÉES

---

## PHASE 7a: RUST BACKEND (Services & Calculs)

### 7a1: Peak Delay Analyzer

**Description**: Calculer le délai (en minutes) entre l'annonce d'un événement et le pic de volatilité réel.

**Fichier**: `src-tauri/src/services/volatility_analyzer.rs` (MODIFY)

**Ce qu'il faut faire**:
1. Ajouter fonction `calculate_peak_delay()`
   - Input: `atr_values: &[f64], event_minute: u8`
   - Output: `peak_delay_minutes: i16` (négatif=avant, positif=après)
   - Trouver l'index du max ATR, calculer délai par rapport à l'événement

2. Intégrer dans commande `analyze_symbol`
   - Stocker `peak_delay` dans Stats15Min
   - Agréger par type d'événement dans historique

**Acceptation**:
- ✅ Fonction retourne `Result<i16, VolatilityError>`
- ✅ Pas de `.unwrap()` ou `.expect()`
- ✅ < 300 lignes (total file)
- ✅ Tests: `test_peak_delay_positive()`, `test_peak_delay_negative()`

**Effort**: 2-3 heures

---

### 7a2: Whipsaw Root Cause Analysis

**Description**: Classifier whipsaws en "early" (avant peak) et "late" (après peak).

**Fichier**: `src-tauri/src/services/volatility/whipsaw_detector.rs` (MODIFY)

**Ce qu'il faut faire**:
1. Ajouter struct `WhipsawAnalysis`
   ```rust
   pub struct WhipsawAnalysis {
       pub whipsaw_freq: f64,        // Total %
       pub whipsaw_early: f64,       // Before peak %
       pub whipsaw_late: f64,        // After peak %
       pub early_avg_loss_pips: f64,
       pub late_avg_loss_pips: f64,
   }
   ```

2. Ajouter fonction `analyze_root_cause()`
   - Input: `whipsaws: &[WhipsawEvent], peak_duration: u16`
   - Compare: `whipsaw.occurred_at < peak_duration` → early
   - Compare: `whipsaw.occurred_at >= peak_duration` → late
   - Calcul moyenne perte par type

3. Retourner `Result<WhipsawAnalysis, VolatilityError>`

**Acceptation**:
- ✅ Classe whipsaws en 2 catégories
- ✅ Calcul moyennes perte par catégorie
- ✅ Tests: `test_whipsaw_early()`, `test_whipsaw_late()`, `test_whipsaw_mixed()`

**Effort**: 2-3 heures

---

### 7a3: Entry Timing Analyzer (NEW SERVICE)

**Description**: Analyser profitabilité par offset d'entrée (T-10, T-5, T-0, T+3 min).

**Fichier**: `src-tauri/src/services/entry_timing_analyzer.rs` (CREATE NEW)

**Ce qu'il faut faire**:
1. Créer struct `EntryTimingRow`
   ```rust
   pub struct EntryTimingRow {
       pub entry_offset_minutes: i8,  // -10, -5, 0, 3
       pub win_rate: f64,             // %
       pub whipsaw_rate: f64,         // %
       pub avg_profit_pips: f64,
       pub sample_size: usize,
   }
   
   pub struct EntryTimingMatrix {
       pub rows: Vec<EntryTimingRow>,
   }
   ```

2. Implémenter fonction `pub fn analyze_entry_timing()`
   - Input: `backtests: &[BacktestResult], event_type: &str`
   - Grouper backtests par `entry_offset`
   - Calculer win_rate, whipsaw_rate, avg_profit pour chaque groupe
   - Retourner `Result<EntryTimingMatrix, VolatilityError>`

3. Exporter fonction dans `src-tauri/src/services/mod.rs`

**Acceptation**:
- ✅ Max 300 lignes
- ✅ Traite 4 offsets distincts
- ✅ Calcul correct des métriques
- ✅ Tests: `test_entry_timing_matrix()`, `test_entry_offset_grouping()`

**Effort**: 2-3 heures

---

### 7a4: Decay Profile

**Description**: Mesurer taux de décroissance ATR après le pic (pips/minute).

**Fichier**: `src-tauri/src/services/volatility_duration_analyzer.rs` (MODIFY)

**Ce qu'il faut faire**:
1. Ajouter struct `DecayProfile`
   ```rust
   pub struct DecayProfile {
       pub peak_atr: f64,
       pub peak_minute: u16,
       pub atr_at_plus_10: f64,
       pub decay_rate_pips_per_min: f64,
       pub decay_speed: DecaySpeed,  // FAST, MEDIUM, SLOW
       pub recommended_timeout: u16,
   }
   
   pub enum DecaySpeed {
       Fast,    // > 3.0 pips/min
       Medium,  // 1.5-3.0 pips/min
       Slow,    // < 1.5 pips/min
   }
   ```

2. Ajouter fonction `pub fn calculate_decay_profile()`
   - Input: `atr_values: &[f64], peak_index: usize`
   - Trouver ATR au peak
   - Trouver ATR à +10 minutes
   - Calculer decay_rate = (peak - at_10) / 10.0
   - Classifier en FAST/MEDIUM/SLOW
   - Recommander timeout: FAST→18min, MEDIUM→25min, SLOW→32min

3. Intégrer dans la commande

**Acceptation**:
- ✅ Correct decay rate calculation
- ✅ Timeout recommendation logique
- ✅ Tests: `test_decay_fast()`, `test_decay_slow()`

**Effort**: 2-3 heures

---

### 7a5: Directional Bias Analyzer (NEW SERVICE)

**Description**: Calculer asymétrie UP vs DOWN de winning trades.

**Fichier**: `src-tauri/src/services/directional_bias_analyzer.rs` (CREATE NEW)

**Ce qu'il faut faire**:
1. Créer struct `DirectionalBiasAnalysis`
   ```rust
   pub struct DirectionalBiasAnalysis {
       pub up_wins_count: usize,
       pub down_wins_count: usize,
       pub whipsaw_count: usize,
       pub up_bias: f64,              // Range: -1.0 to +1.0
       pub asymmetry_percent: f64,    // 0-100
       pub classification: BiasType,  // UP_BIASED, DOWN_BIASED, NEUTRAL
       pub sample_size: usize,
       pub recommendation: String,
   }
   
   pub enum BiasType {
       UpBiased,    // bias > 0.3
       DownBiased,  // bias < -0.3
       Neutral,     // -0.3 to 0.3
   }
   ```

2. Implémenter fonction `pub fn analyze_directional_bias()`
   - Input: `backtests: &[BacktestResult], event_type: &str, pair: &str`
   - Count: up_wins (entry_side == BuyStop && outcome == Win)
   - Count: down_wins (entry_side == SellStop && outcome == Win)
   - Calcul: `up_bias = (up_wins - down_wins) / total`
   - Classifier: > 0.3 = UP_BIASED, < -0.3 = DOWN_BIASED, else NEUTRAL
   - Retourner `Result<DirectionalBiasAnalysis, VolatilityError>`

3. Générer recommandation:
   - UP_BIASED → "Straddle has 35% asymmetry → Use for directional UP"
   - DOWN_BIASED → "Straddle has 40% asymmetry → Use for directional DOWN"
   - NEUTRAL → "Straddle is balanced → Safe to use"

**Acceptation**:
- ✅ Correct bias calculation
- ✅ Asymmetry percentage accurate
- ✅ Recommendation messages clear
- ✅ Tests: `test_bias_up_biased()`, `test_bias_neutral()`, `test_bias_down_biased()`

**Effort**: 2-3 heures

---

## PHASE 7b: DATABASE (Migrations & Schema)

### 7b1: Create Migrations

**Fichier**: `src-tauri/src/db/migrations/` (CREATE FILES)

**Migration 1**: `add_peak_delay_to_stats.sql`
```sql
ALTER TABLE stats_15min ADD COLUMN peak_delay_minutes INTEGER DEFAULT NULL;
ALTER TABLE stats_15min ADD COLUMN decay_speed TEXT DEFAULT 'MEDIUM';
```

**Migration 2**: `add_event_peak_delay_history.sql`
```sql
ALTER TABLE calendar_events ADD COLUMN peak_delay_json TEXT DEFAULT '{}';
-- Format: {"NFP": [2.3, 1.8, 2.5], "Jobless": [4.1, 3.9]}
```

**Migration 3**: `create_entry_timing_analysis_table.sql`
```sql
CREATE TABLE entry_timing_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    pair TEXT NOT NULL,
    entry_offset_minutes INTEGER NOT NULL,
    win_rate REAL NOT NULL,
    whipsaw_rate REAL NOT NULL,
    avg_profit_pips REAL NOT NULL,
    sample_size INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_type, pair, entry_offset_minutes)
);
```

**Migration 4**: `create_directional_bias_table.sql`
```sql
CREATE TABLE directional_bias_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    pair TEXT NOT NULL,
    up_wins_count INTEGER NOT NULL,
    down_wins_count INTEGER NOT NULL,
    whipsaw_count INTEGER NOT NULL,
    up_bias REAL NOT NULL,
    asymmetry_percent REAL NOT NULL,
    classification TEXT NOT NULL,
    sample_size INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_type, pair)
);
```

**Effort**: 1-2 heures

---

## PHASE 7c: FRONTEND (UI Components)

### 7c1: Restructure EventCorrelationView.vue

**Fichier**: `src/components/EventCorrelationView.vue` (MODIFY)

**Ce qu'il faut faire**:
1. Ajouter nouveau state `analysisTab`
   ```typescript
   const analysisTab = ref<'none' | 'peak-delays' | 'decay' | 'entry-timing' | 'directional' | 'whipsaw'>('none')
   ```

2. Restructurer template:
   - **GROUPE 1** (EXISTANT): Mode selector (📅 Par Événement | 💱 Par Paire | 🔥 Heatmap)
   - **GROUPE 2** (NOUVEAU): Analysis tabs (⏰ Peak Delays | 📉 Decay | 📊 Entry | 🎯 Bias | ⚡ Whipsaw)

3. Ajouter conditional rendering:
   ```vue
   <!-- Existing modes -->
   <EventCorrelationByEvent v-if="analysisTab === 'none' && viewMode === 'by-event'" />
   
   <!-- New analyses -->
   <PeakDelayAnalysis v-if="analysisTab === 'peak-delays'" />
   <DecayProfileView v-if="analysisTab === 'decay'" />
   <!-- etc... -->
   ```

4. Importer les 5 nouveaux composants

**Acceptation**:
- ✅ Both tab groups clearly separated
- ✅ Conditional rendering works correctly
- ✅ State management clean

**Effort**: 1-2 heures

---

### 7c2: PeakDelayAnalysis.vue (NEW)

**Purpose**: Afficher tableau des Peak Delays par événement.

**Structure**:
```vue
<template>
  <div class="peak-delay-container">
    <h3>⏰ Peak Delay Analysis</h3>
    <p class="description">When do events peak after announcement?</p>
    
    <div class="filters">
      <select v-model="selectedPair">
        <option value="">All Pairs</option>
        <option>EURUSD</option>
        <!-- etc -->
      </select>
    </div>
    
    <table class="analysis-table">
      <thead>
        <tr>
          <th>Event Type</th>
          <th>Peak Delay</th>
          <th>Samples</th>
          <th>Consistency</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in data" :key="row.eventType">
          <td>{{ row.eventType }}</td>
          <td>{{ row.peakDelay }}±{{ row.stdDev }} min</td>
          <td>{{ row.samples }}</td>
          <td><ProgressBar :value="row.consistency" /></td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface PeakDelayRow {
  eventType: string
  peakDelay: number
  stdDev: number
  samples: number
  consistency: number
}

const selectedPair = ref('')
const data = ref<PeakDelayRow[]>([])

onMounted(async () => {
  // Load from Tauri command
  data.value = await invoke('get_peak_delay_analysis', { pair: selectedPair.value })
})
</script>

<style scoped>
.peak-delay-container { /* styling */ }
.analysis-table { /* table styling */ }
</style>
```

**Acceptance**:
- ✅ Tableau des Peak Delays
- ✅ Filtrable par pair
- ✅ Affiche écart-type (consistency)
- ✅ < 250 lignes

**Effort**: 1-2 heures

---

### 7c3: DecayProfileView.vue (NEW)

**Purpose**: Afficher courbe de décroissance ATR + recommandations.

**Structure**:
```vue
<template>
  <div class="decay-profile-container">
    <h3>📉 Volatility Decay Profile</h3>
    
    <div class="decay-chart">
      <!-- Graphique ATR curve (Canvas ou Chart.js) -->
      <canvas ref="decayChart"></canvas>
    </div>
    
    <div class="decay-stats">
      <div class="stat">
        <label>Peak ATR:</label>
        <value>{{ profile.peakAtr }} pips</value>
      </div>
      <div class="stat">
        <label>Decay Rate:</label>
        <value>{{ profile.decayRate }} pips/min</value>
      </div>
      <div class="stat">
        <label>Decay Speed:</label>
        <badge :type="profile.decaySpeed">{{ profile.decaySpeed }}</badge>
      </div>
      <div class="stat">
        <label>Recommended Timeout:</label>
        <value>{{ profile.timeout }} minutes</value>
      </div>
    </div>
    
    <div class="insight-box">
      <p>{{ profile.insight }}</p>
    </div>
  </div>
</template>
```

**Acceptance**:
- ✅ Chart renders correctly
- ✅ Stats displayed clearly
- ✅ Insight message helpful
- ✅ < 250 lignes

**Effort**: 2-3 heures

---

### 7c4: EntryTimingProfitability.vue (NEW)

**Purpose**: Matrice de profitabilité par offset d'entrée.

**Structure**:
```vue
<template>
  <div class="entry-timing-container">
    <h3>📊 Entry Window Profitability</h3>
    
    <div class="event-selector">
      <select v-model="selectedEventType">
        <option value="">Select Event Type</option>
        <!-- NFP, Jobless, CPI, etc -->
      </select>
      <select v-model="selectedPair">
        <option value="">All Pairs</option>
      </select>
    </div>
    
    <table class="profitability-matrix">
      <thead>
        <tr>
          <th>Entry Time</th>
          <th>Win Rate</th>
          <th>Whipsaw %</th>
          <th>Avg P&L</th>
          <th>Samples</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in matrixRows" :key="row.offset" :class="row.isBest && 'best-row'">
          <td>{{ formatOffset(row.offset) }}</td>
          <td><ProgressBar :value="row.winRate" /></td>
          <td>{{ row.whipsawRate }}%</td>
          <td :class="row.profit > 0 ? 'profit' : 'loss'">{{ row.profit }}p</td>
          <td>{{ row.samples }}</td>
        </tr>
      </tbody>
    </table>
    
    <div class="best-entry-highlight">
      <h4>⭐ Best Entry: {{ bestEntry }}</h4>
      <p>{{ bestEntryReason }}</p>
    </div>
  </div>
</template>
```

**Acceptance**:
- ✅ Matrice affiche 4 offsets (T-10, T-5, T-0, T+3)
- ✅ Highlight meilleure ligne
- ✅ Recommandation claire
- ✅ < 250 lignes

**Effort**: 2-3 heures

---

### 7c5: DirectionalBiasView.vue (NEW)

**Purpose**: Afficher asymétrie UP/DOWN avec warnings.

**Structure**:
```vue
<template>
  <div class="directional-bias-container">
    <h3>🎯 Directional Bias Analysis</h3>
    
    <div class="event-selector">
      <select v-model="selectedEvent">
        <option value="">Select Event Type</option>
      </select>
      <select v-model="selectedPair">
        <option value="">All Pairs</option>
      </select>
    </div>
    
    <div class="bias-visualization">
      <!-- Graphique: UP wins vs DOWN wins -->
      <div class="bias-bar">
        <div class="up-side" :style="{ width: upPercent + '%' }">
          <span>{{ upWins }}% Buy</span>
        </div>
        <div class="down-side" :style="{ width: downPercent + '%' }">
          <span>{{ downWins }}% Sell</span>
        </div>
      </div>
    </div>
    
    <div class="bias-metrics">
      <div class="metric">
        <label>Bias Score:</label>
        <value :class="biasClass">{{ biasScore.toFixed(2) }}</value>
      </div>
      <div class="metric">
        <label>Asymmetry:</label>
        <value>{{ asymmetryPercent }}% ⚠️</value>
      </div>
      <div class="metric">
        <label>Classification:</label>
        <badge :type="classification">{{ classification }}</badge>
      </div>
    </div>
    
    <div class="recommendation-box" :class="`warning-${classification.toLowerCase()}`">
      <h4>⚠️ Recommendation:</h4>
      <p>{{ recommendation }}</p>
    </div>
  </div>
</template>
```

**Acceptance**:
- ✅ Visual bias representation
- ✅ Asymmetry clearly shown
- ✅ Warning appropriately colored
- ✅ < 250 lignes

**Effort**: 2-3 heures

---

### 7c6: WhipsawRootCauseView.vue (NEW)

**Purpose**: Analyser whipsaw early vs late.

**Structure**:
```vue
<template>
  <div class="whipsaw-cause-container">
    <h3>⚡ Whipsaw Root Cause Analysis</h3>
    
    <div class="whipsaw-breakdown">
      <div class="whipsaw-stat">
        <h4>Total Whipsaws</h4>
        <value>{{ totalWhipsaw }}%</value>
      </div>
      
      <div class="whipsaw-section early">
        <h4>Early Whipsaws (Before Peak)</h4>
        <p class="percentage">{{ earlyWhipsaw }}%</p>
        <p class="explanation">Avg Loss: {{ earlyAvgLoss }}p</p>
        <p class="cause">🎲 Bad luck (early entry, peak not yet)</p>
      </div>
      
      <div class="whipsaw-section late">
        <h4>Late Whipsaws (After Peak)</h4>
        <p class="percentage">{{ lateWhipsaw }}%</p>
        <p class="explanation">Avg Loss: {{ lateAvgLoss }}p</p>
        <p class="cause">⚠️ SL too tight (didn't capture momentum)</p>
      </div>
    </div>
    
    <div class="recommendation-box">
      <h4>💡 Recommendation:</h4>
      <p v-if="lateWhipsawHigh">
        "High late-whipsaw detected. Increase SL by +{{ slIncrease }}p"
      </p>
      <p v-else>
        "Whipsaw pattern is balanced. Current SL is appropriate."
      </p>
    </div>
  </div>
</template>
```

**Acceptance**:
- ✅ Early/Late distinction clear
- ✅ Average loss calculated
- ✅ Actionable recommendation
- ✅ < 250 lignes

**Effort**: 2-3 heures

---

## PHASE 7d: FORMULES & DOCUMENTATION

### 7d1: Update formules.ts

**Fichier**: `src/data/formules.ts` (MODIFY)

**Ce qu'il faut faire**:
1. Ajouter 5 nouvelles formules dans l'export

```typescript
export const formules: Record<string, Formule> = {
  // ... existing formules ...
  
  // === NEW RETROSPECTIVE METRICS ===
  
  peak_delay: {
    id: 'peak_delay',
    titre: 'Peak Delay (Minutes)',
    categorieId: 'retrospectif',
    definition: 'Délai en minutes entre l\'annonce d\'un événement et le pic de volatilité réel.',
    explication_litterale: 'Cette formule mesure QUAND arrive le vrai mouvement. Si Peak Delay = +2.3 min, ça signifie qu\'après l\'annonce, il faut attendre 2.3 minutes pour voir le mouvement maximum. Utile pour savoir: "Quand est-ce que je dois être attentif?"',
    formule: 'Peak_Delay = Time(max_ATR) - Time(event_announcement)',
    inputs: ['Time of announcement', 'ATR timeseries'],
    output: { type: 'integer', range: '-5 to +15', unite: 'minutes' },
    exemple: 'NFP annoncé à 13:30:00 → Peak ATR à 13:32:18 → Delay = +2.3 min',
    notes: [
      'Négatif = pic avant l\'annonce (rare)',
      'Positif = pic après l\'annonce (habituel)',
      'Variance importante selon paires et types d\'événements'
    ]
  },

  whipsaw_root_cause: {
    id: 'whipsaw_root_cause',
    titre: 'Whipsaw Root Cause',
    categorieId: 'retrospectif',
    definition: 'Analyse des whipsaws: combien avant peak vs après peak.',
    explication_litterale: 'Cette formule sépare les faux déclenchements en deux: ceux qui arrivent AVANT le pic (mauvaise chance) et ceux qui arrivent APRÈS (mauvais SL). Si beaucoup de whipsaws "late", tu dois agrandir ton SL. Si beaucoup de "early", c\'est juste de la malchance.',
    formule: 'Whipsaw_Early% = (whipsaws_before_peak / total_whipsaws) × 100\nWhipsaw_Late% = (whipsaws_after_peak / total_whipsaws) × 100',
    inputs: ['Whipsaw events', 'Peak duration'],
    output: { type: 'float', range: '0-100', unite: '%' },
    exemple: '8% early (before peak), 20% late (after peak) → SL issue → Increase SL',
    notes: [
      'Early whipsaw = avant le pic = malchance du timing',
      'Late whipsaw = après le pic = SL trop serré',
      'Ajuster SL si late% > 15%'
    ]
  },

  entry_timing_profitability: {
    id: 'entry_timing_profitability',
    titre: 'Entry Timing Profitability',
    categorieId: 'retrospectif',
    definition: 'Profitabilité stratifiée par offset d\'entrée (T-10, T-5, T-0, T+3).',
    explication_litterale: 'Cette formule te montre: "Si j\'étais entré 5 minutes avant l\'annonce, quel aurait été mon win rate?" Compare 4 moments d\'entrée différents pour trouver le meilleur. Le moment idéal change selon l\'événement.',
    formule: 'For each entry_offset in [-10, -5, 0, +3]:\n  Win_Rate(offset) = (wins / total) × 100\n  P&L(offset) = sum(profits) / total',
    inputs: ['Backtest results', 'Entry time offsets'],
    output: { type: 'matrix', range: '4 rows × 5 cols', unite: 'win%, P&L' },
    exemple: 'T-5 min: 52% win, +18p avg ← BEST\nT-0 min: 50% win, +8p avg\nT+3 min: 45% win, -5p avg',
    notes: [
      'Meilleur offset varie par type d\'événement',
      'NFP optimal = T-5 min',
      'Jobless optimal = T-3 min'
    ]
  },

  volatility_decay_profile: {
    id: 'volatility_decay_profile',
    titre: 'Volatility Decay Profile',
    categorieId: 'retrospectif',
    definition: 'Taux de décroissance de la volatilité après le pic (pips/minute).',
    explication_litterale: 'Cette formule mesure: "Comment vite la volatilité baisse après le mouvement?" Si la volatilité baisse très vite (3 pips/minute), le mouvement est court, donc timeout court. Si elle baisse lentement (0.8 pips/minute), le mouvement dure longtemps, donc timeout long.',
    formule: 'Decay_Rate = (Peak_ATR - ATR_at_T+10) / 10 min\nDecay_Speed = FAST (>3) | MEDIUM (1.5-3) | SLOW (<1.5)',
    inputs: ['ATR timeseries', 'Peak ATR value'],
    output: { type: 'float', range: '0.5 to 5.0', unite: 'pips/minute' },
    exemple: 'Peak 45p → 18p at T+10 → Decay = 2.7p/min = MEDIUM → Timeout = 25 min',
    notes: [
      'FAST decay = mouvement court = short timeout (18 min)',
      'MEDIUM decay = équilibré = medium timeout (25 min)',
      'SLOW decay = mouvement long = long timeout (32 min)'
    ]
  },

  directional_bias_score: {
    id: 'directional_bias_score',
    titre: 'Directional Bias Score',
    categorieId: 'retrospectif',
    definition: 'Asymétrie UP vs DOWN des gagnants: mesure la tendance inhérente.',
    explication_litterale: 'Cette formule regarde: "Les achats gagnent-ils plus que les ventes pour cet événement?" Si oui = événement biaisé HAUT. Si non = biaisé BAS. Si égal = neutre. Un Straddle fonctionne mieux sur événements neutres.',
    formule: 'UP_Bias = (Up_Wins - Down_Wins) / Total_Wins\nAsymmetry% = |UP_Bias| × 100\nClassification: |Bias| > 0.3 = BIASED, else NEUTRAL',
    inputs: ['Backtest results (buy/sell side)'],
    output: { type: 'enum', range: '{UP_BIASED, DOWN_BIASED, NEUTRAL}', unite: 'classification' },
    exemple: 'NFP: 67% buy wins, 33% sell wins → Bias = +0.7 → UP_BIASED',
    notes: [
      'Straddle fonctionne mal sur événements biaisés',
      'Meilleur sur événements NEUTRAL',
      'Si biaisé, utiliser pour stratégies directionnelles'
    ]
  }
}
```

2. Ajouter nouvelle catégorie dans `categories` array:
```typescript
{
  id: 'retrospectif',
  titre: 'Analyse Rétrospective',
  emoji: '📊',
  description: 'Métriques d\'analyse rétrospective pour backtesting',
  formules: ['peak_delay', 'whipsaw_root_cause', 'entry_timing_profitability', 'volatility_decay_profile', 'directional_bias_score']
}
```

**Acceptance**:
- ✅ 5 formules new avec définitions complètes
- ✅ Explications littérales en français simple (no jargon)
- ✅ Catégorie nouvelles bien nommée
- ✅ Exemples réalistes

**Effort**: 1-2 heures

---

## PHASE 7e: TESTING & VALIDATION

### 7e1: Rust Unit Tests

**Pour chaque service créé/modifié:**
- ✅ `test_peak_delay_positive()`, `test_peak_delay_negative()`
- ✅ `test_whipsaw_early()`, `test_whipsaw_late()`, `test_whipsaw_mixed()`
- ✅ `test_entry_timing_matrix()`, `test_entry_offset_grouping()`
- ✅ `test_decay_fast()`, `test_decay_slow()`
- ✅ `test_bias_up_biased()`, `test_bias_neutral()`, `test_bias_down_biased()`

**Command**: `cargo test`
- ✅ Tous les tests passent
- ✅ 0 compiler warnings
- ✅ Coverage > 80%

**Effort**: 2-3 heures

---

### 7e2: Frontend Component Tests

**Pour chaque composant créé:**
- ✅ Component renders without error
- ✅ Props passed correctly
- ✅ Event emitters work
- ✅ Conditional rendering correct

**Effort**: 1-2 heures

---

### 7e3: Integration Tests

- ✅ Database migrations apply without error
- ✅ Tauri commands execute correctly
- ✅ Frontend calls backend correctly
- ✅ Data flows end-to-end

**Effort**: 1-2 heures

---

### 7e4: Regression Tests

- ✅ Existing tabs still work (volatility, archives)
- ✅ MetricsAnalysisModal still works
- ✅ FormulasModal not broken
- ✅ No performance degradation

**Commands**:
```bash
make check          # All quality gates
cargo test          # All Rust tests
npm run build       # TypeScript compilation
```

**Acceptance**:
- ✅ All tests pass
- ✅ 0 unwrap() violations
- ✅ File sizes compliant
- ✅ No dead code

**Effort**: 1-2 heures

---

## 📊 TIMELINE RÉSUMÉE

| Phase | Durée | Status |
|-------|-------|--------|
| 7a: Rust Backend | 3 jours | ⬜ TODO |
| 7b: Database | 1 jour | ⬜ TODO |
| 7c: Frontend | 2-3 jours | ⬜ TODO |
| 7d: Formules | 1 jour | ⬜ TODO |
| 7e: Testing | 1-2 jours | ⬜ TODO |
| **TOTAL** | **~8-9 jours** | ⬜ TODO |
| **Parallélisé** | **~5-6 jours** | ⬜ TODO |

---

## 🎯 ORDRE D'EXÉCUTION RECOMMANDÉ

### **Option A: Séquentiel (Sûr, Simple)**
1. Phase 7b (Database) ← Dépendance commune
2. Phase 7a (Rust Backend) ← Services
3. Phase 7c (Frontend) ← UI dépend de services
4. Phase 7d (Formules) ← Documentation
5. Phase 7e (Testing) ← Validation

### **Option B: Parallélisé (Rapide)**
1. **Jour 1**: 7b (Database) + 7d (Formules) en parallèle
2. **Jour 2-3**: 7a (Rust Backend) 5 services en parallèle
3. **Jour 4-5**: 7c (Frontend) 6 components en parallèle
4. **Jour 5-6**: 7e (Testing) intégration + validation

**Recommandation**: Option B (parallélisé = 5-6 jours réels)

---

## ✅ CHECKLIST DE COMPLÉTION

- [ ] 7a1: Peak Delay Analyzer implémenté
- [ ] 7a2: Whipsaw Root Cause implémenté
- [ ] 7a3: Entry Timing Analyzer implémenté
- [ ] 7a4: Decay Profile implémenté
- [ ] 7a5: Directional Bias Analyzer implémenté
- [ ] 7b1: Database migrations exécutées
- [ ] 7c1: EventCorrelationView restructuré
- [ ] 7c2-7c6: 5 nouveaux composants créés
- [ ] 7d1: 5 formules ajoutées à formules.ts
- [ ] 7e1-7e4: Tests passent, no regressions
- [ ] Documentation complète dans /docs
- [ ] GitHub: Phase 7 branch mergé à main
- [ ] Commit avec message détaillé

---

## 🚀 PRÊT À COMMENCER?

**Prochaine action**: Choisir l'ordre (A ou B) et starter Phase 7b ou 7a.

**Questions avant de commencer?**
- Budget temps?
- Parallélisation possible?
- Priorité (feature rapidement vs qualité)?
