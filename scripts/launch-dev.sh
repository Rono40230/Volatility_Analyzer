#!/bin/bash
# =============================================================================
# Script de lancement DEV fiable
# Résout les problèmes :
#   1. EPIPE esbuild (race condition Vite/Tauri)
#   2. Crash Wayland GDK Error 71
# =============================================================================

set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "🧹 Nettoyage des processus précédents..."
pkill -f "analyses-historiques" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
kill $(lsof -t -i:1420) 2>/dev/null || true
sleep 1

echo "🚀 Démarrage de Vite dev server..."
npx vite &
VITE_PID=$!

# Attendre que Vite soit prêt
for i in $(seq 1 20); do
  if curl -s http://localhost:1420/ > /dev/null 2>&1; then
    echo "✅ Vite prêt (${i}s)"
    break
  fi
  sleep 0.5
done

echo "🔥 Pré-chauffage du cache esbuild..."
for f in src/main.ts src/App.vue src/stores/volatility.ts src/stores/analysisStore.ts src/components/AnalysisPanel.vue src/components/HourlyTable.vue src/views/HomeView.vue; do
  curl -s "http://localhost:1420/$f" > /dev/null 2>&1 || true
done
echo "✅ Cache pré-chauffé"

echo "🖥️  Lancement de Tauri (Wayland natif + DMABUF désactivé)..."
cd src-tauri
GDK_BACKEND=wayland WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run &
TAURI_PID=$!

echo ""
echo "════════════════════════════════════════════"
echo "  App lancée ! PID Vite=$VITE_PID, PID Tauri=$TAURI_PID"
echo "  Ctrl+C pour tout arrêter"
echo "════════════════════════════════════════════"

# Attendre la fin de Tauri
wait $TAURI_PID 2>/dev/null

# Quand Tauri se ferme, tuer Vite aussi
echo "🛑 Arrêt de Vite..."
kill $VITE_PID 2>/dev/null || true
echo "✅ Terminé"
