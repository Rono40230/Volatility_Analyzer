#!/bin/bash
# validate-tests.sh - Exécution tests et capture résultats

EXIT_CODE=0

echo "🧪 Exécution des tests..."

# Exécuter les tests depuis le répertoire src-tauri
TEST_OUTPUT=$(cd src-tauri && cargo test --release 2>&1)
TEST_COUNT=$(echo "$TEST_OUTPUT" | grep -oE "test .* ok" | wc -l)
TEST_FAILURES=$(echo "$TEST_OUTPUT" | grep -oE "test .* FAILED" | wc -l)

echo "$TEST_OUTPUT"

if [ "$TEST_FAILURES" -eq 0 ]; then
    echo ""
    echo "✅ Tests: $TEST_COUNT tests passants"
    EXIT_CODE=0
else
    echo ""
    echo "❌ Tests: $TEST_FAILURES tests échoués"
    EXIT_CODE=1
fi

exit $EXIT_CODE
