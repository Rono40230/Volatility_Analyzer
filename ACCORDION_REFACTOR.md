# 📋 Changement Architecture: Accordion Horaire ↔ 15 Minutes

## Avant ❌
- 2 composants séparés: HourlyTable + ScalpingTable15min
- 2 onglets/toggles pour basculer entre les vues
- Duplication de logique Vue
- L'utilisateur doit cliquer sur un bouton pour changer

## Après ✅
- 1 composant unique: HourlyTableWithScalping
- 24 heures affichées par défaut (compact)
- Cliquer sur une heure → se déploie pour voir les 4 tranches de 15min
- Tout dans un seul tableau hiérarchique
- Meilleure UX: simple et détaillé au même endroit

---

## 🎯 Comment ça marche

```
Tableau Principal (24 heures)
│
├─ Heure 00:00 [▶] ← Cliquer pour déployer
│  └─ Accord se développe :
│     ├─ 00:00-00:15 (Quarter 0)
│     ├─ 00:15-00:30 (Quarter 1)
│     ├─ 00:30-00:45 (Quarter 2)
│     └─ 00:45-01:00 (Quarter 3)
│
├─ Heure 01:00 [▶]
│  └─ (collapsed by default)
│
└─ Heure 23:00 [▶]
```

---

## 📊 Affichage

### Niveau Horaire (Default View)
```
Heure    │ ATR  │ Vol %  │ Body % │ Quality │ ... │ Événements
─────────┼──────┼────────┼────────┼─────────┼─────┼──────────
00:00 ▶  │ 1234 │ 12.5%  │ 45.2%  │ 75      │ ... │ 🇺🇸🇯🇵
01:00 ▶  │ 1156 │ 10.2%  │ 42.1%  │ 62      │ ... │ 🇬🇧
```

### Niveau 15-Minute (Expanded)
```
Quand tu cliques sur "00:00 ▶", apparaît:

Tranche        │ ATR  │ Vol % │ Body % │ Quality │ Events
───────────────┼──────┼───────┼────────┼─────────┼────────
00:00-00:15    │ 1100 │ 11%   │ 42%    │ 70      │ 🇺🇸
00:15-00:30    │ 1050 │ 9%    │ 40%    │ 58      │ -
00:30-00:45    │ 1250 │ 14%   │ 48%    │ 82      │ 🇯🇵
00:45-01:00    │ 1000 │ 8%    │ 38%    │ 52      │ -
```

---

## 🔧 Changements Techniques

### Fichiers Impactés

#### ✨ Nouveau
- `src/components/HourlyTableWithScalping.vue` (650 lignes)
  - Combine HourlyTable + ScalpingTable15min
  - Gestion expand/collapse avec state `expandedHours`
  - Deux niveaux de tables imbriquées

#### 🗑️ Obsolètes (mais conservés pour référence)
- `src/components/HourlyTable.vue` (encore là, non utilisé)
- `src/components/ScalpingTable15min.vue` (encore là, non utilisé)

#### 🔧 Modifiés
- `src/App.vue`
  - Remplacé: `import HourlyTable + ScalpingTable15min`
  - Par: `import HourlyTableWithScalping`
  - Supprimé: ref `showScalpingView`
  - Supprimé: div `.view-toggle` et boutons
  - Supprimé: logique v-if/v-else-if toggle
  - Nouveau: seul appel à HourlyTableWithScalping

---

## 🎨 Interface

### Bouton Expand/Collapse
- Flèche `▶` pour chaque heure
- Animation rotation 90° quand déployé ▼
- Couleur #00d4ff (cyan)
- Hover effect: couleur + cursor pointer

### Color-Coding 15min
- 🟢 Excellente (≥70): Fond vert léger
- 🟠 Bonne (40-70): Fond orange léger
- 🔴 Faible (<40): Fond rouge léger

### Séparation Visuelle
- Heure parent: fond #1a1a2e + bordure cyan
- Accordion: fond #0f2a3e + padding
- Table 15min: fond #1a1a2e, colonnes réduites

---

## 💾 Props du Composant

```typescript
HourlyTableWithScalping {
  hourlyStats: HourlyStats[]      // 24 heures
  stats15min: Stats15Min[]          // 96 tranches
  bestHours?: number[]              // Optionnel
}
```

---

## ⌨️ Interactions Utilisateur

1. **Chargement** → Affiche 24 heures
2. **Cliquer heure** → Déploie ses 4 tranches de 15min
3. **Lire détails** → Voir volatilité précise per 15min
4. **Cliquer heure déployée** → Se replie
5. **Multi-select** → Peut déployer plusieurs heures simultanément

---

## 🚀 Avantages

✅ **UX Améliorée**
- Plus simple visuellement (pas de toggle buttons)
- Intuitive (like file explorer accordion)
- Compacte au démarrage (24 lignes)

✅ **Performance**
- Pas de refonte complète du DOM
- Les 15min chargés à la demande (virtuellement)
- Smooth animations

✅ **Maintenance**
- 1 seul composant au lieu de 2
- Logique centralisée
- Pas de duplication

✅ **Scalabilité**
- Facile d'ajouter d'autres niveaux de détail
- Même architecture peut supporter 5-minute granularity

---

## 🔍 Détails Techniques

### State Management
```typescript
const expandedHours = ref<number[]>([])  // Heures déployées
```

### Fonctions Clés
```typescript
toggleExpand(hour)         // Ajoute/retire hour du state
getQuartersForHour(hour)   // Récupère 4 tranches pour une heure
calculateQualityScore()    // Score 0-100 pour chaque tranche
```

### Event Handling
- Clic sur `expand-btn` → `toggleExpand(hour)`
- Propagation bloquée pour éviter déployer la row parente

---

## 📱 Responsive Design

**Desktop:** Tableau full-width, tous les détails visibles
**Tablet:** Horizontal scroll si besoin, accordion reste compact
**Mobile:** Peut avoir besoin de scroll horizontal

---

## ✨ Prochaines Étapes (Optionnel)

1. **Expand All / Collapse All** buttons
2. **Sticky first column** pour heures visibles lors du scroll
3. **Copy to clipboard** pour export rapide
4. **Filtrer par qualité** (show only high-quality slots)
5. **Dark mode** (already done)

---

**Status:** ✅ Implémentation Complète  
**Date:** 2025-11-15  
**Type:** Refactoring UX (no backend changes)
