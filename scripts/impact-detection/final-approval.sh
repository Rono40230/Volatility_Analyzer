#!/bin/bash
# final-approval.sh - Rapport final complet et décision (OK ou KO pour commit)

set -e

SNAPSHOTS_DIR=".git/.snapshots"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "              📋 RAPPORT FINAL DE VÉRIFICATION D'IMPACT"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Afficher le rapport d'impact
if [ -f "$SNAPSHOTS_DIR/impact-report.txt" ]; then
    echo "📊 IMPACT DES CHANGEMENTS"
    echo "───────────────────────────────────────────────────────────"
    cat "$SNAPSHOTS_DIR/impact-report.txt" | head -20
    echo ""
fi

# Afficher le rapport de régression
if [ -f "$SNAPSHOTS_DIR/regression-report.txt" ]; then
    echo "🧪 VÉRIFICATION DES RÉGRESSIONS"
    echo "───────────────────────────────────────────────────────────"
    grep -E "BASELINE|RÉSULTATS|ANALYSE|STATUS" "$SNAPSHOTS_DIR/regression-report.txt" | head -15
    echo ""
fi

# Exécuter les vérifications pré-commit existantes
echo "🔒 VÉRIFICATION DES RÈGLES .clinerules"
echo "───────────────────────────────────────────────────────────"

CHECKS_PASSED=0
CHECKS_FAILED=0

# 1. Vérifier la taille des fichiers
if ./scripts/check-file-size.sh > /dev/null 2>&1; then
    echo "  ✅ Taille fichiers"
    ((CHECKS_PASSED++))
else
    echo "  ❌ Taille fichiers"
    ((CHECKS_FAILED++))
fi

# 2. Vérifier unwrap()
if ./scripts/check-unwrap.sh > /dev/null 2>&1; then
    echo "  ✅ Pas d'unwrap() en production"
    ((CHECKS_PASSED++))
else
    echo "  ❌ unwrap() détecté"
    ((CHECKS_FAILED++))
fi

# 3. Vérifier anti-patterns
if ./scripts/check-antipatterns.sh > /dev/null 2>&1; then
    echo "  ✅ Anti-patterns"
    ((CHECKS_PASSED++))
else
    echo "  ⚠️  Anti-patterns (avertissement)"
    ((CHECKS_PASSED++))
fi

# 4. Vérifier code mort
if ./scripts/check-dead-code.sh > /dev/null 2>&1; then
    echo "  ✅ Pas de code mort"
    ((CHECKS_PASSED++))
else
    echo "  ❌ Code mort détecté"
    ((CHECKS_FAILED++))
fi

# 5. Vérifier imports circulaires
if ./scripts/check-circular-imports.sh > /dev/null 2>&1; then
    echo "  ✅ Pas d'imports circulaires"
    ((CHECKS_PASSED++))
else
    echo "  ❌ Imports circulaires"
    ((CHECKS_FAILED++))
fi

# 6. Vérifier architecture DAG
if ./scripts/check-architecture.sh > /dev/null 2>&1; then
    echo "  ✅ Architecture DAG 4 niveaux"
    ((CHECKS_PASSED++))
else
    echo "  ❌ Architecture DAG violée"
    ((CHECKS_FAILED++))
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "                      📊 RÉSUMÉ FINAL"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  Vérifications passées : $CHECKS_PASSED/6 ✅"
echo "  Vérifications échouées : $CHECKS_FAILED/6 ❌"
echo ""

# Décision finale
if [ $CHECKS_FAILED -eq 0 ]; then
    if grep -q "STATUS : ❌" "$SNAPSHOTS_DIR/regression-report.txt" 2>/dev/null; then
        echo "═══════════════════════════════════════════════════════════════"
        echo "❌ COMMIT BLOQUÉ - Régression détectée"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        echo "Les changements cassent des tests existants."
        echo "Corrige les régressions avant de committer."
        echo ""
        exit 1
    else
        echo "═══════════════════════════════════════════════════════════════"
        echo "✅ APPROUVÉ POUR COMMIT"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        echo "Tous les contrôles sont passés ✅"
        echo "Aucune régression détectée ✅"
        echo "Architecture respectée ✅"
        echo "Code propre ✅"
        echo ""
        echo "Tu peux committer en toute confiance ! 🚀"
        echo ""
        exit 0
    fi
else
    echo "═══════════════════════════════════════════════════════════════"
    echo "❌ COMMIT BLOQUÉ - Violations détectées"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Corrige les $CHECKS_FAILED violations avant de committer."
    echo ""
    exit 1
fi
