#!/bin/bash
# validate-phase2.sh - Orchestre la Phase 2 complète
# À exécuter quand l'utilisateur dit "valide tout et commit"

set -e

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "              🎯 PHASE 2 : VALIDATION D'IMPACT COMPLÈTE"
echo "═══════════════════════════════════════════════════════════════"
echo ""

SNAPSHOTS_DIR=".git/.snapshots"

# Vérifier qu'un snapshot existe
if [ ! -f "$SNAPSHOTS_DIR/pre-phase2-state-"* ]; then
    echo "❌ Aucun snapshot trouvé"
    echo "Exécute d'abord : ./scripts/impact-detection/init-impact-system.sh"
    exit 1
fi

echo "📍 Étape 1 : Vérification de l'impact des changements..."
echo ""
if ! ./scripts/impact-detection/verify-impact.sh; then
    echo "❌ Erreur lors de la vérification d'impact"
    exit 1
fi

echo ""
echo "───────────────────────────────────────────────────────────"
echo ""
echo "📍 Étape 2 : Détection des régressions..."
echo ""
if ! ./scripts/impact-detection/regression-detector.sh; then
    echo "❌ Régression détectée - Commit bloqué"
    exit 1
fi

echo ""
echo "───────────────────────────────────────────────────────────"
echo ""
echo "📍 Étape 3 : Rapport final et décision..."
echo ""
if ! ./scripts/impact-detection/final-approval.sh; then
    echo "❌ Validation échouée - Commit bloqué"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ PHASE 2 RÉUSSIE - PRÊT POUR COMMIT"
echo "═══════════════════════════════════════════════════════════════"
echo ""

exit 0
