#!/bin/bash
# init-impact-system.sh - Initialise le système d'impact detection
# À exécuter au DÉBUT de la Phase 1 (avant de faire les changements)

set -e

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "       🚀 INITIALISATION DU SYSTÈME D'IMPACT DETECTION"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Nettoyer les snapshots précédents
SNAPSHOTS_DIR=".git/.snapshots"
if [ -d "$SNAPSHOTS_DIR" ]; then
    echo "🧹 Nettoyage des snapshots précédents..."
    rm -f "$SNAPSHOTS_DIR"/*.json "$SNAPSHOTS_DIR"/*.txt "$SNAPSHOTS_DIR"/*.log
fi

mkdir -p "$SNAPSHOTS_DIR"

# Prendre un snapshot initial (baseline)
echo "📸 Prise de snapshot initial (baseline)..."
./scripts/impact-detection/snapshot-dependencies.sh

echo ""
echo "✅ Système initialisé et prêt pour la Phase 1"
echo ""
echo "Tu peux maintenant :"
echo "  1. Faire tes changements et améliorations (Phase 1)"
echo "  2. L'IA teste chaque modification"
echo "  3. Quand tu dis 'valide tout', je vérifierai l'impact (Phase 2)"
echo ""

exit 0
