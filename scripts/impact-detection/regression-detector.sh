#!/bin/bash
# regression-detector.sh - Détecte les régressions
# Teste tous les modules, compare avec le baseline, alerte sur les cassures

set -e

SNAPSHOTS_DIR=".git/.snapshots"
SNAPSHOT_FILE=$(ls -t "$SNAPSHOTS_DIR"/pre-phase2-state-*.json 2>/dev/null | head -1)

if [ -z "$SNAPSHOT_FILE" ]; then
    echo "❌ Aucun snapshot trouvé."
    exit 1
fi

echo "🧪 Détection des régressions..."
echo ""

# Récupérer le baseline des tests
BASELINE_PASSED=$(jq -r '.tests.passed' "$SNAPSHOT_FILE")
BASELINE_FAILED=$(jq -r '.tests.failed' "$SNAPSHOT_FILE")

echo "📊 Baseline (avant changements) :"
echo "  - Tests passés : $BASELINE_PASSED"
echo "  - Tests échoués : $BASELINE_FAILED"
echo ""

# Lancer les tests maintenant
echo "🏃 Exécution des tests..."
TEST_OUTPUT=$(cd src-tauri && cargo test --release 2>&1 || true)

# Compter les tests
CURRENT_PASSED=$(echo "$TEST_OUTPUT" | grep -c "^test .* ok$" || echo "0")
CURRENT_FAILED=$(echo "$TEST_OUTPUT" | grep -c "^test .* FAILED$" || echo "0")

echo ""
echo "📊 Résultats actuels :"
echo "  - Tests passés : $CURRENT_PASSED"
echo "  - Tests échoués : $CURRENT_FAILED"
echo ""

# Analyser les différences
NEWLY_BROKEN=$((CURRENT_FAILED - BASELINE_FAILED))
NEWLY_FIXED=$((BASELINE_FAILED - CURRENT_FAILED))

echo "═══════════════════════════════════════════"
echo "🔍 ANALYSE DES RÉGRESSIONS"
echo "═══════════════════════════════════════════"

if [ $NEWLY_BROKEN -gt 0 ]; then
    echo "❌ $NEWLY_BROKEN test(s) cassé(s) ! (RÉGRESSION)"
    echo ""
    echo "$TEST_OUTPUT" | grep "^test .* FAILED$" | head -5
    echo ""
    REGRESSION=1
elif [ $NEWLY_FIXED -gt 0 ]; then
    echo "✅ $NEWLY_FIXED test(s) réparé(s) ! (Amélioration)"
    echo ""
    REGRESSION=0
elif [ $CURRENT_FAILED -gt 0 ]; then
    echo "⚠️  $CURRENT_FAILED test(s) échoué(s) au départ (ignorés)"
    echo "Pas de nouvelle régression détectée ✅"
    echo ""
    REGRESSION=0
else
    echo "✅ Tous les tests passent !"
    echo ""
    REGRESSION=0
fi

echo "═══════════════════════════════════════════"

# Sauvegarder le rapport de régression
REGRESSION_FILE="$SNAPSHOTS_DIR/regression-report.txt"
{
    echo "RAPPORT DE RÉGRESSION - $(date)"
    echo "═══════════════════════════════════════════"
    echo ""
    echo "BASELINE (avant changements)"
    echo "  Tests passés : $BASELINE_PASSED"
    echo "  Tests échoués : $BASELINE_FAILED"
    echo ""
    echo "RÉSULTATS ACTUELS"
    echo "  Tests passés : $CURRENT_PASSED"
    echo "  Tests échoués : $CURRENT_FAILED"
    echo ""
    echo "ANALYSE"
    echo "  Nouveaux cassés : $NEWLY_BROKEN"
    echo "  Nouveaux réparés : $NEWLY_FIXED"
    echo ""
    if [ $REGRESSION -eq 1 ]; then
        echo "STATUS : ❌ RÉGRESSION DÉTECTÉE"
    else
        echo "STATUS : ✅ AUCUNE RÉGRESSION"
    fi
    echo ""
    echo "DÉTAILS DES TESTS"
    echo "$TEST_OUTPUT"
} > "$REGRESSION_FILE"

echo "✅ Rapport de régression sauvegardé : $REGRESSION_FILE"
echo ""

if [ $REGRESSION -eq 1 ]; then
    echo "❌ RÉGRESSION DÉTECTÉE - Commit bloqué"
    exit 1
else
    echo "✅ Aucune régression"
    exit 0
fi
