<script setup lang="ts">
import { ref, onMounted } from 'vue'

const emit = defineEmits<{
  (e: 'navigate', tab: string): void
  (e: 'open-modal', modal: 'formulas' | 'export'): void
}>()

const recentItems = ref<Array<{ title: string, date: string, type: string, tab: string }>>([])

onMounted(() => {
  // Mock mock data for now, relying on LocalStorage later
  const saved = localStorage.getItem('recent_analyses')
  if (saved) {
    try {
        recentItems.value = JSON.parse(saved)
    } catch (e) {}
  }
})

function navigate(tab: string) {
  emit('navigate', tab)
}

function openModal(modal: 'formulas' | 'export') {
  emit('open-modal', modal)
}
</script>

<template>
  <div class="home-view">
    <div class="hero-section">
      <h1>Bienvenue sur Volatility Analyzer</h1>
      <p class="subtitle">Sélectionnez un outil pour commencer votre analyse</p>
    </div>

    <div class="recents-section" v-if="recentItems.length > 0">
      <h2>Récents</h2>
      <div class="recents-list">
        <div 
          v-for="(item, idx) in recentItems" 
          :key="idx" 
          class="recent-item"
          @click="navigate(item.tab)"
        >
          <div class="recent-icon">🕒</div>
          <div class="recent-info">
            <span class="recent-title">{{ item.title }}</span>
            <span class="recent-date">{{ item.date }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="tools-grid">
      <!-- Heatmap -->
      <div class="tool-card" @click="navigate('heatmap')">
        <div class="card-icon heatmap-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"/>
            <path d="M3 9h18M9 21V9" stroke-width="2"/>
            <rect x="11" y="11" width="8" height="8" fill="currentColor" fill-opacity="0.2" class="pulse-rect"/>
          </svg>
        </div>
        <h3>Heatmap</h3>
        <p>Corrélation visuelle des événements et paires</p>
      </div>

      <!-- Volatilité -->
      <div class="tool-card" @click="navigate('volatility')">
        <div class="card-icon vol-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <path d="M3 12h3l3 -9l6 18l3 -9h3" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chart-line"/>
          </svg>
        </div>
        <h3>Volatilité</h3>
        <p>Analyse de volatilité brute par paire et période</p>
      </div>

      <!-- Corrélation -->
      <div class="tool-card" @click="navigate('retrospective')">
        <div class="card-icon corr-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <circle cx="12" cy="12" r="10" stroke-width="2"/>
            <path d="M8 12h8m-4-4v8" stroke-width="2" class="spin-cross"/>
          </svg>
        </div>
        <h3>Corrélation</h3>
        <p>Étude de l'impact des événements passés</p>
      </div>

      <!-- Planning -->
      <div class="tool-card" @click="navigate('planning')">
        <div class="card-icon plan-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <rect x="3" y="4" width="18" height="18" rx="2" stroke-width="2" class="calendar-rect"/>
            <line x1="16" y1="2" x2="16" y2="6" stroke-width="2" class="calendar-hook"/>
            <line x1="8" y1="2" x2="8" y2="6" stroke-width="2" class="calendar-hook"/>
            <line x1="3" y1="10" x2="21" y2="10" stroke-width="2" class="calendar-line"/>
          </svg>
        </div>
        <h3>Planning</h3>
        <p>Calendrier économique et événements futurs</p>
      </div>

      <!-- Archives -->
      <div class="tool-card" @click="navigate('archives')">
        <div class="card-icon archive-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <polyline points="21 8 21 21 3 21 3 8" stroke-width="2" class="box-body"/>
            <rect x="1" y="3" width="22" height="5" stroke-width="2" class="box-lid"/>
            <line x1="10" y1="12" x2="14" y2="12" stroke-width="2" class="box-lock"/>
          </svg>
        </div>
        <h3>Archives</h3>
        <p>Consulter les sauvegardes d'analyses</p>
      </div>

      <!-- Import -->
      <div class="tool-card" @click="navigate('calendar')">
         <div class="card-icon import-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" stroke-width="2"/>
            <polyline points="7 10 12 15 17 10" stroke-width="2" class="arrow-down"/>
            <line x1="12" y1="15" x2="12" y2="3" stroke-width="2" class="arrow-down"/>
          </svg>
        </div>
        <h3>Importer</h3>
        <p>Mettre à jour les données et calendriers</p>
      </div>

      <!-- Formules -->
      <div class="tool-card" @click="openModal('formulas')">
         <div class="card-icon formula-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
             <!-- Radical symbol -->
             <path d="M2 12h4l3 9 5-17h8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="math-radical"/>
             <!-- Fraction -->
             <g class="math-fraction">
               <text x="18" y="10" font-family="serif" font-weight="bold" font-size="7" fill="currentColor" stroke="none" text-anchor="middle" class="math-num">x</text>
               <line x1="15" y1="12" x2="21" y2="12" stroke-width="2" stroke-linecap="round" />
               <text x="18" y="19" font-family="serif" font-weight="bold" font-size="7" fill="currentColor" stroke="none" text-anchor="middle" class="math-denom">y</text>
             </g>
          </svg>
        </div>
        <h3>Formules</h3>
        <p>Documentation des calculs utilisés</p>
      </div>
      
      <!-- Exports -->
      <div class="tool-card" @click="openModal('export')">
         <div class="card-icon export-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" fill="none" class="animated-icon">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" stroke-width="2"/>
            <polyline points="17 8 12 3 7 8" stroke-width="2" class="arrow-up"/>
            <line x1="12" y1="3" x2="12" y2="15" stroke-width="2" class="arrow-up"/>
          </svg>
        </div>
        <h3>Data Export</h3>
        <p>Exporter les données brutes ou analysées</p>
      </div>


    </div>
  </div>
</template>

<style scoped>
.home-view {
  padding: 40px;
  width: 100%;
  height: 100%;
  color: #eee;
  animation: fadeIn 0.5s ease-out;
  display: flex;
  flex-direction: column;
  align-items: center;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.hero-section {
  text-align: center;
  margin-bottom: 60px;
  margin-top: 40px;
}

.hero-section h1 {
  font-size: 3rem;
  margin-bottom: 10px;
  background: linear-gradient(45deg, #42b883, #35495e);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.subtitle {
  color: #888;
  font-size: 1.2rem;
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
  width: 100%;
  max-width: 1200px;
  padding: 0 40px;
  margin-bottom: 40px;
}

.tool-card {
  background: #1e1e1e;
  border: 1px solid #333;
  border-radius: 12px;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

.tool-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 30px rgba(0,0,0,0.3);
  border-color: #42b883;
}

.tool-card:hover .card-icon {
  color: #42b883;
}

.card-icon {
  margin-bottom: 16px;
  color: #666;
  transition: color 0.3s;
}

/* Animations */
.tool-card:hover .pulse-rect {
  animation: pulse 2s infinite;
}

.tool-card:hover .chart-line {
  stroke-dasharray: 100;
  stroke-dashoffset: 100;
  animation: draw 1.5s ease-out forwards;
}

.tool-card:hover .spin-cross {
  animation: spin 3s linear infinite;
  transform-origin: center;
}

.tool-card:hover .arrow-down {
  animation: bounceDown 1s infinite;
}
.tool-card:hover .arrow-up {
  animation: bounceUp 1s infinite;
}

/* New Animations */
.tool-card:hover .graph-line {
  stroke-dasharray: 100;
  stroke-dashoffset: 100;
  animation: draw 2s ease-out forwards infinite;
}

.tool-card:hover .calendar-rect {
  animation: pulse 2s infinite;
}
.tool-card:hover .calendar-hook {
  animation: bounceDown 1s infinite;
}

.tool-card:hover .box-lid {
  transform-origin: 12px 3px;
  animation: flap 1s infinite alternate;
}

/* Math Animations */
.tool-card:hover .math-radical {
  stroke-dasharray: 60;
  stroke-dashoffset: 60;
  animation: drawMath 1.5s ease-out forwards;
}

.tool-card:hover .math-num,
.tool-card:hover .math-denom {
  transform-box: fill-box;
  transform-origin: center;
}

.tool-card:hover .math-num {
  animation: floatUp 1s ease-in-out infinite alternate;
}
.tool-card:hover .math-denom {
  animation: floatDown 1s ease-in-out infinite alternate;
}

@keyframes flap {
  0% { transform: rotateX(0deg); }
  100% { transform: rotateX(-45deg); }
}

@keyframes pop {
  0% { transform: scale(1); }
  100% { transform: scale(1.2); }
}

@keyframes pulse {
  0% { fill-opacity: 0.2; }
  50% { fill-opacity: 0.6; }
  100% { fill-opacity: 0.2; }
}

@keyframes draw {
  from { stroke-dashoffset: 100; }
  to { stroke-dashoffset: 0; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes bounceDown {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(3px); }
}
@keyframes bounceUp {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-3px); }
}

@keyframes drawMath {
  from { stroke-dashoffset: 60; }
  to { stroke-dashoffset: 0; }
}

@keyframes floatUp {
  from { transform: translateY(0); }
  to { transform: translateY(-2px); }
}

@keyframes floatDown {
  from { transform: translateY(0); }
  to { transform: translateY(2px); }
}

h3 {
  margin: 0 0 8px 0;
  font-size: 1.2rem;
  font-weight: 600;
}

p {
  margin: 0;
  color: #888;
  font-size: 0.9rem;
  line-height: 1.4;
}

.recents-section {
  margin-bottom: 30px;
}
</style>
