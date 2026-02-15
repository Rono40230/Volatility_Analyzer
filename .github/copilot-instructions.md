# 🤖 Copilot Instructions - Analyses Historiques

**Context**: Volatility Analyzer for Forex, index, crypto and métaux (Tauri 2.0 + Vue 3 + Rust).

## 🏗️ Architecture
- **Frontend**: Vue 3 (Composition API `<script setup>`), TypeScript, Pinia.
- **Backend**: Rust (Tauri), Diesel (SQLite), Polars.
- **Data Flow**: UI → Store → `invoke('command')` → Service → DB/Calc → Result.

## 📏 Critical Rules (Strict Enforcement)
1.  **Error Handling**:
    -   **Rust**: Return `Result<T, VolatilityError>`. Use `?`. **NO `unwrap()`**.
    -   **Vue**: `try/catch` around `invoke()`. No `console.log()`.
2.  **File Size Limits** (Split if exceeded):
    -   Rust Service: < 300 lines
    -   Rust Command: < 200 lines
    -   Vue Component: < 250 lines
3.  **DAG Architecture**:
    -   Models (L1) ← DB (L2) ← Services (L3) ← Commands (L4).
    -   **Never** import between services at the same level.

## 🛠️ Developer Workflows
-   **Validation (Phase 1)**: Code & Test locally (`cargo test`). Accumulate changes.
-   **Commit (Phase 2)**: Run `./scripts/impact-detection/validate-phase2.sh` when user approves.
-   **Quality Checks**:
    -   `./scripts/check-file-size.sh`
    -   `cargo clippy -- -D warnings`

## 📊 Domain: Straddle Strategy
-   **Goal**: Optimize **Entry**, **TP/SL**, **Duration** based on historical volatility.
-   **Golden Rule**: If a metric doesn't help parameterize a Straddle, **don't implement it**.
-   **Key Metrics**: ATR, Noise Ratio (>3.0 is bad), Event Impact, Volatility Decay.

## 📝 Code Patterns

### Rust Command
```rust
#[tauri::command]
pub async fn analyze_symbol(symbol: String) -> Result<AnalysisResult> {
    if symbol.is_empty() { return Err(VolatilityError::ValidationError("...".into())); }
    let service = VolatilityService::new();
    Ok(service.analyze(symbol).await?)
}
```

### Vue Component
```vue
<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const loading = ref(false)
const result = ref(null)

async function loadData() {
  try {
    loading.value = true
    result.value = await invoke('analyze_symbol', { symbol: 'EURUSD' })
  } catch (e) {
    // Handle error
  } finally {
    loading.value = false
  }
}
</script>
```
