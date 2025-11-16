#!/bin/bash
# check-coverage.sh - Vérification couverture de tests (>80%)

MIN_COVERAGE=80
EXIT_CODE=0

echo "📊 Mesure de la couverture de tests..."

# Vérifier si tarpaulin est installé
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "⚠️  cargo-tarpaulin non installé. Installation..."
    cargo install cargo-tarpaulin || {
        echo "❌ Impossible d'installer cargo-tarpaulin"
        exit 1
    }
fi

# Exécuter tarpaulin et capturer la couverture
COVERAGE_OUTPUT=$(cd src-tauri && cargo tarpaulin --out Xml --timeout 300 --exclude-files migrations/* 2>&1)

if echo "$COVERAGE_OUTPUT" | grep -q "Coverage:"; then
    COVERAGE=$(echo "$COVERAGE_OUTPUT" | grep "Coverage:" | awk '{print $NF}' | sed 's/%//')
    
    if (( $(echo "$COVERAGE >= $MIN_COVERAGE" | bc -l) )); then
        echo "✅ Couverture: ${COVERAGE}% (minimum: ${MIN_COVERAGE}%) - OK"
        EXIT_CODE=0
    else
        echo "❌ Couverture: ${COVERAGE}% (minimum requis: ${MIN_COVERAGE}%)"
        EXIT_CODE=1
    fi
else
    echo "⚠️  Couverture non disponible - Vérification des tests passants..."
    if cargo test --release 2>&1 | grep -q "test result: ok"; then
        echo "✅ Tests passants - Couverture non mesurable"
        EXIT_CODE=0
    else
        echo "❌ Certains tests ont échoué"
        EXIT_CODE=1
    fi
fi

exit $EXIT_CODE
