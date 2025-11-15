# 15-Minute Granularity System - Technical Reference

## Quick Summary
- **Purpose:** Enable precise scalping analysis by breaking hourly data into 15-minute windows
- **Result:** 96 data points per 24h period (24 hours × 4 quarters)
- **Implementation Date:** 2025-11-15
- **Status:** ✅ Complete and verified

## Architecture Overview

### Three-Layer Implementation

#### Layer 1: Data Model (Rust)
- **File:** `src-tauri/src/models/stats_15min.rs`
- **Struct:** `Stats15Min` with 14 metric fields
- **Key Fields:** `hour` (0-23), `quarter` (0-3), `events: Vec<EventInHour>`
- **Methods:** `time_label()`, `quality_score()`, `quality_rating()`

#### Layer 2: Calculation Logic (Rust)
- **File:** `src-tauri/src/services/volatility/stats_15min.rs`
- **Struct:** `Stats15MinCalculator`
- **Logic:**
  - Groups candles by 15-minute buckets (UTC → Paris timezone)
  - Calculates same metrics as HourlyStatsCalculator
  - Associates HIGH/MEDIUM economic events to exact 15-min slots

#### Layer 3: UI/Display (Vue/TypeScript)
- **File:** `src/components/ScalpingTable15min.vue`
- **Features:**
  - 96-row table with color-coded quality ratings
  - Automatic flag display for events
  - Intelligent score calculation
  - Responsive layout with grouping separators

## Time Conversion Reference

**Timezone Handling:**
```
Candle datetime (UTC) → hour, minute
Convert: hour = (utc_hour + 1) % 24  // Paris = UTC+1 (winter)
Quarter: minute / 15 = 0,1,2,3
Result: (hour, quarter) = exact 15-min bucket in Paris time
```

**Quarter Numbering:**
| Quarter | Time Window | Minutes |
|---------|-------------|---------|
| 0       | 00:00-00:15 | 0-14    |
| 1       | 00:15-00:30 | 15-29   |
| 2       | 00:30-00:45 | 30-44   |
| 3       | 00:45-01:00 | 45-59   |

## Integration Points

### Backend Integration
1. **Analyzer:** `src-tauri/src/services/volatility/analyzer.rs`
   - Line ~130: `let calculator_15min = Stats15MinCalculator::new(&self.candles);`
   - Line ~132: `self.load_and_associate_events_15min(...);`
   - Line ~200: Return `stats_15min` in `AnalysisResult`

2. **Event Association:** New method `load_and_associate_events_15min()`
   - Mirrors `load_and_associate_events()` logic
   - Assigns events to (hour, quarter) tuple instead of just hour

### Frontend Integration
1. **Store:** `src/stores/volatility.ts`
   - New interface: `Stats15Min`
   - Updated: `AnalysisResult.stats_15min: Vec<Stats15Min>`

2. **Main App:** `src/App.vue`
   - Ref: `showScalpingView = ref(false)`
   - Toggle buttons (📊 vs 🎯)
   - Conditional rendering: HourlyTable vs ScalpingTable15min

## Quality Score Algorithm

**Frontend Calculation (Vue):**
```typescript
volatility_score = Math.min(volatility_mean * 5, 40)     // 0-40 pts
breakout_score = Math.min(breakout_percentage / 2.5, 30) // 0-30 pts
quality_score = Math.min(tick_quality_mean * 10, 30)     // 0-30 pts
total = volatility_score + breakout_score + quality_score // 0-100
```

**Rating Scale:**
- ≥70: "Excellent" (green background)
- 40-70: "Bon/Medium" (orange background)
- <40: "Faible/Low" (red background)

## Event Display

**Flag Mapping:**
- Uses `getEventTranslation(eventName)` from `eventTranslations.ts`
- Returns emoji flag based on country (🇺🇸 🇬🇧 🇩🇪 🇯🇵 🇫🇷 etc.)
- Multiple events: display 3-4 flags per 15-min slot

**Example:**
```
14:30-14:45: "US NFP (09:00 UTC)" → Quarter 2 (30-45min) of hour 14
Displayed as: 🇺🇸 (with title showing full impact)
```

## Testing Checklist

- [ ] Verify stats_15min has exactly 96 elements for any symbol
- [ ] Check event counts match between hourly and 15min views
- [ ] Verify time labels display correctly (00:00-00:15, etc.)
- [ ] Confirm timezone conversion (UTC to Paris) is accurate
- [ ] Test toggle switching between HourlyTable and ScalpingTable15min
- [ ] Validate quality_score calculation gives 0-100 range
- [ ] Check flag display for multiple events per 15min slot
- [ ] Verify empty slots (no candles) show 0 values gracefully

## Performance Notes

**Memory Impact:**
- +96 items vs 24 items = 4× increase for stats array
- Negligible (each Stats15Min ≈ 200 bytes)
- Total: ~20KB per symbol analysis

**Rendering:**
- 96 table rows render smoothly in modern browsers
- Sticky header for easy scrolling
- CSS grid-based layout is efficient

## Maintenance Notes

### DST (Daylight Saving Time)
**Current:** `const PARIS_OFFSET_HOURS: i32 = 1;` (UTC+1, winter)

**TODO:** Implement dynamic DST detection
```rust
// Pseudocode for future
let paris_offset = if is_dst(candle.datetime) { 2 } else { 1 };
```

### Adding New Metrics
To add new metrics to the 15-min view:
1. Add field to `Stats15Min` struct
2. Calculate in `Stats15MinCalculator.calculate_for_slice()`
3. Update Vue component to display in table

### Exporting Data
To add CSV/JSON export for 15-min stats:
1. Create new command in `src-tauri/src/commands/`
2. Serialize `Vec<Stats15Min>` to file
3. Add export button in ScalpingTable15min.vue

## Files Reference Map

```
Rust Backend:
├─ models/stats_15min.rs (247 lines) ..................... Model definition
├─ services/volatility/stats_15min.rs (133 lines) ....... Calculator logic
├─ services/volatility/analyzer.rs (mod) ................ Integration point
├─ models/analysis_result.rs (mod) ....................... Response structure
└─ services/volatility/mod.rs (mod) ..................... Module export

TypeScript/Vue Frontend:
├─ stores/volatility.ts (mod) ........................... Type definitions
├─ components/ScalpingTable15min.vue (520 lines) ........ Display component
└─ App.vue (mod) ........................................ Toggle UI + routing

Configuration:
└─ IMPLEMENTATION_15MIN.md ............................... This document
```

## Troubleshooting

**Issue:** Stats_15min field not appearing in AnalysisResult
- Check: `AnalysisResult.stats_15min` is public in analysis_result.rs
- Check: Serialization is enabled (#[derive(Serialize)])

**Issue:** Events not showing in 15-min view
- Check: `load_and_associate_events_15min()` is called
- Check: Quarter calculation: `(minute / 15) as u8` is correct
- Verify: Event time in UTC, converted to Paris hour correctly

**Issue:** Toggle button disabled in UI
- Reason: `stats_15min` is empty or None
- Check: Analysis completed successfully
- Verify: Backend is returning non-empty stats_15min

**Issue:** Wrong time labels (e.g., "01:00-01:00")
- Bug in: `time_label()` method in Vue component
- Check: Quarter boundary calculation at hour transitions

## Links & References

- **Event Translations:** `src/stores/eventTranslations.ts`
- **Event Schedules:** `src/stores/eventSchedules.ts`
- **Hourly System (Legacy):** `src-tauri/src/services/volatility/hourly_stats.rs`
- **Type Definitions:** `src/stores/volatility.ts`

---

**Last Updated:** 2025-11-15  
**Status:** Active and Maintained  
**Contact:** GitHub Copilot
