<template>
  <div class="main-container">
    <div class="content-area">
      <div v-if="loading" class="loading"><div class="spinner"></div><p>Analyse en cours...</p></div>
      <div v-if="error" class="error"><h3>❌ Erreur</h3><p>{{ error }}</p></div>
      <div v-if="!selectedSymbol && !loading && !error" class="welcome">
        <div class="welcome-icon">🌍</div><h3>Sélectionnez une paire</h3>
        <div class="welcome-select-container">
          <select v-model="selectedSymbol" @change="analyzeSymbol" class="welcome-symbol-select">
            <option value="">Choisir une paire</option>
            <option v-for="s in symbols" :key="s.symbol" :value="s.symbol">{{ s.symbol }}</option>
          </select>
        </div>
      </div>
      <div v-if="sessionData && !loading">
        <div class="general-info">
          <h3>�� {{ selectedSymbol }} - Analyse complète</h3>
          <div class="info-grid">
            <div class="info-card"><div class="info-icon">📅</div><div class="info-value">{{ sessionData.period }}</div><div class="info-label">Période</div></div>
            <div class="info-card"><div class="info-icon">📊</div><div class="info-value">{{ sessionData.total_candles.toLocaleString() }}</div><div class="info-label">Bougies</div></div>
          </div>
        </div>
        <div class="stats-by-session">
          <h3>📈 Volatilité par session</h3>
          <div class="sessions-grid">
            <div v-for="session in sessionData.sessions" :key="session.session_name" class="session-card">
              <h4>{{ session.session_name }}</h4><div class="stat">Volatilité: {{ session.avg_volatility.toFixed(1) }} pips</div>
              <div class="stat">Max: {{ session.max_volatility.toFixed(1) }} pips</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface SessionData {
  symbol: string
  period: string
  total_candles: number
  sessions: Array<{ session_name: string; avg_volatility: number; max_volatility: number }>
}

const selectedSymbol = ref<string>('')
const sessionData = ref<SessionData | null>(null)
const loading = ref(false)
const error = ref<string>('')
const symbols = ref<Array<{ symbol: string; file_path: string }>>([])

onMounted(async () => {
  try {
    symbols.value = await invoke('load_symbols')
  } catch (err) {
    console.error('Erreur:', err)
  }
})

async function analyzeSymbol() {
  if (!selectedSymbol.value) return
  loading.value = true
  error.value = ''
  try {
    sessionData.value = await invoke('get_session_analysis', { symbol: selectedSymbol.value })
  } catch (err: any) {
    error.value = err.toString()
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.main-container { background: #161b22; border-radius: 16px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); border: 1px solid #30363d; color: #e2e8f0; }
.content-area { padding: 30px; min-height: 400px; }
.loading { text-align: center; padding: 60px 20px; }
.spinner { width: 50px; height: 50px; border: 4px solid #2d3748; border-top: 4px solid #667eea; border-radius: 50%; animation: spin 1s linear infinite; margin: 0 auto 20px; }
@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
.error { padding: 20px; background: #dc2626; border-radius: 8px; }
.welcome { text-align: center; padding: 60px 20px; }
.welcome-icon { font-size: 4em; margin-bottom: 20px; }
.welcome h3 { font-size: 1.8em; color: #e2e8f0; margin-bottom: 15px; }
.welcome-select-container { display: flex; justify-content: center; margin-top: 30px; }
.welcome-symbol-select { padding: 12px 24px; font-size: 1.1em; border-radius: 8px; border: 2px solid #4a5568; background: #ffffff; color: #000000; cursor: pointer; min-width: 350px; }
.general-info { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; margin-bottom: 30px; }
.info-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px; }
.info-card { background: #2d3748; padding: 15px; border-radius: 8px; text-align: center; }
.info-icon { font-size: 2em; margin-bottom: 10px; }
.info-value { font-size: 1.5em; font-weight: bold; color: #e2e8f0; }
.info-label { color: #a0aec0; font-size: 0.9em; margin-top: 5px; }
.stats-by-session { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.sessions-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px; }
.session-card { background: #2d3748; padding: 15px; border-radius: 8px; border-left: 3px solid #667eea; }
.session-card h4 { color: #e2e8f0; margin: 0 0 10px 0; }
.stat { color: #cbd5e0; font-size: 0.9em; margin-bottom: 8px; }
</style>
