#!/bin/bash
# snapshot-dependencies.sh - Prend un snapshot de l'état du code avant Phase 2
# Sauvegarde : liste des fichiers, structure des dépendances, baseline des tests

set -e

SNAPSHOTS_DIR=".git/.snapshots"
TIMESTAMP=$(date +%s)
SNAPSHOT_FILE="$SNAPSHOTS_DIR/pre-phase2-state-$TIMESTAMP.json"
BASELINE_FILE="$SNAPSHOTS_DIR/test-baseline-$TIMESTAMP.json"

mkdir -p "$SNAPSHOTS_DIR"

echo "📸 Prise de snapshot de l'état du code..."
echo ""

# 1. Sauvegarder la liste des fichiers Rust
echo "  1️⃣  Fichiers Rust..."
RUST_FILES=$(find src-tauri/src -name "*.rs" -type f | sort)
FILE_COUNT=$(echo "$RUST_FILES" | wc -l)
FILE_HASHES=$(echo "$RUST_FILES" | xargs -I {} sh -c 'echo "{}:$(md5sum {} | cut -d" " -f1)"' | sort)

# 2. Sauvegarder les dépendances entre modules
echo "  2️⃣  Structure des dépendances..."
DEP_GRAPH=$(find src-tauri/src -name "*.rs" -type f -print0 | xargs -0 grep -h "^use crate::" | sort -u)

# 3. Baseline des tests AVANT changements
echo "  3️⃣  Baseline des tests..."
TEST_BASELINE=$(cd src-tauri && cargo test --release 2>&1 | grep -E "^test .* (ok|FAILED)" || true)
TEST_PASSED=$(echo "$TEST_BASELINE" | grep -c " ok$" 2>/dev/null || echo "0")
TEST_FAILED=$(echo "$TEST_BASELINE" | grep -c " FAILED$" 2>/dev/null || echo "0")

# S'assurer que ce sont des nombres simples (sans newlines)
TEST_PASSED=$(echo "$TEST_PASSED" | tr -d '\n')
TEST_FAILED=$(echo "$TEST_FAILED" | tr -d '\n')

# 4. Sauvegarder les métriques de code
echo "  4️⃣  Métriques du code..."
TOTAL_LINES=$(find src-tauri/src -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}')
TOTAL_FILES=$(find src-tauri/src -name "*.rs" -type f | wc -l)

# 5. Créer le snapshot JSON (utiliser jq pour éviter les problèmes de bash/newlines)
jq -n \
  --arg timestamp "$TIMESTAMP" \
  --arg date "$(date)" \
  --arg file_count "$FILE_COUNT" \
  --arg total_files "$TOTAL_FILES" \
  --arg total_lines "$TOTAL_LINES" \
  --arg test_passed "$TEST_PASSED" \
  --arg test_failed "$TEST_FAILED" \
  --arg test_baseline "$TEST_BASELINE" \
  --arg file_hashes "$FILE_HASHES" \
  --arg dep_graph "$DEP_GRAPH" \
  '{
    "timestamp": $timestamp,
    "date": $date,
    "files": {
      "count": ($file_count | tonumber),
      "total_rust_files": ($total_files | tonumber),
      "total_lines": ($total_lines | tonumber)
    },
    "tests": {
      "passed": ($test_passed | tonumber),
      "failed": ($test_failed | tonumber),
      "baseline": $test_baseline
    },
    "file_hashes": ($file_hashes | split("\n") | map(select(length > 0))),
    "dependencies": ($dep_graph | split("\n") | map(select(length > 0)))
  }' > "$SNAPSHOT_FILE"

echo ""
echo "✅ Snapshot créé : $SNAPSHOT_FILE"
echo "  - Fichiers Rust : $FILE_COUNT"
echo "  - Tests baseline : $TEST_PASSED passés, $TEST_FAILED échoués"
echo "  - Lignes totales : $TOTAL_LINES"
echo ""

# Nettoyer les anciens snapshots (garder que le dernier)
echo "🧹 Nettoyage des anciens snapshots..."
ls -t "$SNAPSHOTS_DIR"/pre-phase2-state-*.json 2>/dev/null | tail -n +2 | xargs -r rm

echo "✅ Snapshot prêt pour Phase 2"
exit 0
