# Test Plan - Phase 3: Validation Archive Analysis

## 🧪 Test Checklist (PHASE 3)

### Étape 3.1: Unit Tests (Composables)

#### ✅ useArchiveStatistics.ts
- [ ] `loadAllArchives()` successfully loads all 25 archives via Tauri
- [ ] `parseArchiveByType()` correctly routes to appropriate parser
- [ ] `archivesByEvent` computed correctly groups by event type
- [ ] `archivesByPair` computed correctly groups by pair
- [ ] `eventStatistics` computed produces stats for all events
- [ ] `pairStatistics` computed produces stats for all pairs
- [ ] `heatmapData` computed returns valid matrix structure
- [ ] `dynamicAdvice` computed generates 4+ advice items
- [ ] `globalStats` computed aggregates correctly

#### ✅ useArchiveParsers.ts
- [ ] `parseVolatilityArchive()` returns correct structure for Type 1
- [ ] `parseRetrospectiveArchive()` returns correct structure for Type 2
- [ ] `parseHeatmapArchive()` returns normalized array structure
- [ ] All parsers handle edge cases (empty arrays, null values)
- [ ] No unhandled exceptions on malformed data

#### ✅ useArchiveMetrics.ts
- [ ] `calculateEventStatistics()` produces EventStats with all fields
- [ ] `calculateTradabilityScore()` returns 0-100 range
- [ ] Formula validation: (conf*0.4) + (stability*0.3) + (impact*0.3)
- [ ] `calculatePairStatistics()` groups events by sensitivity
- [ ] `getPairRating()` returns correct emoji + text

#### ✅ useArchiveCalculations.ts
- [ ] `extractHeatmapData()` produces pairs/events/impactMatrix
- [ ] `calculateOptimalStraddleParams()` returns valid SL/TP ratios
- [ ] `generateAdvice()` produces 4 distinct recommendation types
- [ ] Gain estimates are realistic (2-3x ATR)

---

### Étape 3.2: Component Tests (Vue)

#### ✅ EventAnalysisBlock.vue
- [ ] Displays all events from `eventStatistics`
- [ ] Sort order is by tradabilityScore descending
- [ ] Color coding matches tradability level (🟢🟡🔴)
- [ ] Metrics display correctly (ATR, Pic, Confiance, Analyses)
- [ ] Straddle Setup calculated as: SL=ATR×1.5, TP=ATR×3
- [ ] Header stats show totalEvents, optimalCount, avgConfidence
- [ ] Empty state displays when no events

#### ✅ PairAnalysisBlock.vue
- [ ] Displays all pairs from `pairStatistics`
- [ ] Grid layout 2 columns on desktop, 1 on mobile
- [ ] Performance color matches rating (TRÈS BON/BON/MOYEN/FAIBLE)
- [ ] Top sensitive event correctly identified per pair
- [ ] ATR average displayed in header
- [ ] Strong pairs count calculated correctly

#### ✅ TimingAnalysisBlock.vue
- [ ] Timeline table shows all events sorted by peakDelay ascending
- [ ] Columns: Événement | Placement (sec) | Durée exit | Gain estimé | Score
- [ ] Fastest/Slowest events highlighted with special styling
- [ ] Average placement calculated and shown in header
- [ ] Color coding in score column matches tradability

#### ✅ AdviceBlock.vue
- [ ] Displays dynamic advice from `dynamicAdvice` array
- [ ] KPI cards show: Risk Level, Win Rate, Config Quality
- [ ] Advice items color-coded by type (success/warning/info/strategy)
- [ ] Risk level matches calculated confidence range
- [ ] Win Rate estimate between 45-75%
- [ ] CTA button \"Lancer Analyse Complète\" visible

#### ✅ GlobalStatsBlock.vue
- [ ] 5 metric cards with correct values from globalStats
- [ ] Quality Score bar animated (0-100%)
- [ ] Data coverage shows: archives/25, events, pairs
- [ ] Two insights display dynamically based on data
- [ ] Color scheme consistent (blue/green/purple/yellow/indigo)

#### ✅ GlobalAnalysisModal.vue
- [ ] Modal opens when isOpen=true
- [ ] Modal closes on click outside or X button
- [ ] All 5 new blocks render without errors
- [ ] Scroll works when content exceeds viewport
- [ ] Header title shows \"✨ IAnalyse Statistique - Archives\"
- [ ] No console errors or warnings

---

### Étape 3.3: Integration Tests

#### ✅ Data Flow
- [ ] Tauri command `list_all_archives` returns 25 archives
- [ ] Archives load within < 2 seconds
- [ ] All 25 archives successfully parse (no errors)
- [ ] Parsed data normalizes to NormalizedArchive interface
- [ ] Computed values update reactively when data changes
- [ ] No memory leaks with large datasets

#### ✅ User Interaction
- [ ] Click \"Détails\" button on event (prep for next phase)
- [ ] Hover on pair shows sensitive event highlight
- [ ] Click \"Impact\" button shows heatmap (prep for next phase)
- [ ] Scroll through all 5 blocks works smoothly
- [ ] Modal responsive on mobile (< 768px)

#### ✅ Error Handling
- [ ] Missing archive data handled gracefully (empty state)
- [ ] Corrupted JSON in archives doesn't crash
- [ ] Tauri command timeout handled (error message displayed)
- [ ] Graceful fallback when globalStats is null

#### ✅ Performance
- [ ] Modal load time < 3 seconds
- [ ] No visible lag when scrolling (60fps)
- [ ] Computed values update instantly (< 100ms)
- [ ] No unnecessary re-renders
- [ ] Memory usage < 50MB (even with 25 archives)

---

## 📋 Test Execution Log

### Run 1: Initial Load (PENDING)
```
Timestamp: [TBD]
Environment: Fedora Linux
Browser: [TBD - Firefox/Chrome]

Setup:
1. Open app in dev environment
2. Navigate to GlobalAnalysisModal (click \"IAnalyse Statistique\")
3. Wait for archives to load

Expected:
- 5 blocks render with data
- No console errors
- Headers show correct stats

Actual:
- [TBD after manual testing]
```

### Run 2: Data Validation (PENDING)
```
Verify:
- EventAnalysisBlock shows ≥15 events
- PairAnalysisBlock shows ≥5 pairs  
- TimingAnalysisBlock table has all events
- AdviceBlock shows ≥2 recommendations
- GlobalStatsBlock shows 25 archives loaded

Actual:
- [TBD after manual testing]
```

### Run 3: Responsive & Accessibility (PENDING)
```
Test on:
- Desktop (1920x1080)
- Tablet (768x1024)
- Mobile (375x667)

Expected:
- All blocks responsive
- Text readable
- Buttons clickable

Actual:
- [TBD after manual testing]
```

---

## 🎯 Success Criteria (All Must Pass)

| Criterion | Status | Notes |
|-----------|--------|-------|
| All 5 blocks render without errors | ⏳ PENDING | Awaiting manual test |
| eventStatistics calculates ≥15 events | ⏳ PENDING | |
| pairStatistics shows ≥5 pairs | ⏳ PENDING | |
| Tradability scores 0-100 range | ⏳ PENDING | |
| Colors match predefined scheme | ⏳ PENDING | |
| No console.log/warn/error | ⏳ PENDING | |
| Responsive on all screen sizes | ⏳ PENDING | |
| No TypeScript compilation errors | ⏳ PENDING | |
| All Tauri commands execute | ⏳ PENDING | |
| Modal opens/closes smoothly | ⏳ PENDING | |

---

## 📝 Manual Test Script (Interactive)

```bash
# 1. Start dev environment
npm run tauri dev

# 2. In browser DevTools console, verify no errors:
# - No red text in console
# - No 404 network errors

# 3. Click on GlobalAnalysisModal trigger (button/menu)

# 4. Visual verification (5 min)
- 🟢 EventAnalysisBlock appears with events
- 🟡 PairAnalysisBlock appears with grid
- 🔴 TimingAnalysisBlock appears with table
- 🟣 AdviceBlock appears with recommendations
- 🟠 GlobalStatsBlock appears with metrics

# 5. Interaction checks (3 min)
- Scroll through blocks
- Hover on items
- Check responsive on mobile view (F12 → device toolbar)

# 6. Data validation (2 min)
- Count events shown (should be ≥15)
- Count pairs shown (should be ≥5)
- Verify at least 1 \"OPTIMAL\" event (score ≥80)

# 7. Close modal and reopen
- Should reload archives
- Should maintain same data
```

---

## 🚀 Sign-Off

- **Phase 1 (Infrastructure)**: ✅ COMPLETED & COMMITTED
- **Phase 2 (Display)**: ✅ COMPLETED & COMMITTED
- **Phase 3 (Testing)**: 🟡 IN PROGRESS (Manual testing required)

**Ready for manual QA**: YES - All code deployed, awaiting visual validation
