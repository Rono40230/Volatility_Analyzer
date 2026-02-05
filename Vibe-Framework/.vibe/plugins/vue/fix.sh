#!/bin/bash
# Plugin Vue.js - Fix
# Génère un patch via worktree Git au lieu de modifier directement l'arbre courant
VIBE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PATCH_DIR="$VIBE_ROOT/.vibe/patches"
mkdir -p "$PATCH_DIR"

TARGET_DIR="."

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    WORKTREE_DIR=$(mktemp -d)
    git worktree add "$WORKTREE_DIR" HEAD >/dev/null 2>&1 || { rm -rf "$WORKTREE_DIR"; echo "⚠️ git worktree failed, fallback to in-place fixes"; }
    if [ -d "$WORKTREE_DIR" ]; then
        cd "$WORKTREE_DIR/$TARGET_DIR" || exit 0
        if [ -f "package.json" ]; then
            if grep -q "lint" package.json; then
                npm run lint -- --fix > /dev/null 2>&1 || true
            fi
            if grep -q "format" package.json; then
                npm run format > /dev/null 2>&1 || true
            fi
        fi

        # Appliquer les corrections non-destructives (commentaires, script setup)
        for file in src/**/*.vue src/*.vue; do
            if [ -f "$file" ]; then
                sed -E -e 's/^(\s*)(console\.(log|error|warn|debug)\(|debugger|alert\()/\1\/\/ \2/' -i "$file" || true
            fi
        done
        for file in src/**/*.vue src/*.vue; do
            if [ -f "$file" ] && grep -q '<script>' "$file" && ! grep -q '<script setup' "$file"; then
                sed -i 's/<script>/<script setup>/' "$file" || true
            fi
        done

        PATCH_FILE="$PATCH_DIR/vue_fix_$(date '+%Y%m%d_%H%M%S').patch"
        (cd "$WORKTREE_DIR" && git add -A && git diff --binary HEAD) > "$PATCH_FILE" || true
        echo "🔖 Patch Vue généré : $PATCH_FILE"
        META_FILE="${PATCH_FILE}.json"
        MOD_FILES=$(cd "$WORKTREE_DIR" && git diff --name-only HEAD || true)
        STAT=$(cd "$WORKTREE_DIR" && git diff --shortstat HEAD || true)
        HASH=$(sha256sum "$PATCH_FILE" 2>/dev/null | awk '{print $1}' || echo "")
        cat > "$META_FILE" <<JSON
    { "patch": "$(basename "$PATCH_FILE")", "files": "$(echo "$MOD_FILES" | tr '\n' ',' | sed 's/,$//')", "stat": "$(echo $STAT | sed 's/"/\"/g')", "hash": "$HASH", "timestamp": "$(date --iso-8601=seconds)" }
    JSON
        echo "🔖 Meta JSON généré : $META_FILE"
        git worktree remove --force "$WORKTREE_DIR" >/dev/null 2>&1 || rm -rf "$WORKTREE_DIR"
    fi
else
    # Fallback : modifications en place et création d'un diff
    if [ -f "package.json" ]; then
        if grep -q "lint" package.json; then
            npm run lint -- --fix > /dev/null 2>&1 || true
        fi
        if grep -q "format" package.json; then
            npm run format > /dev/null 2>&1 || true
        fi
    fi
    for file in src/**/*.vue src/*.vue; do
        if [ -f "$file" ]; then
            sed -E -e 's/^(\s*)(console\.(log|error|warn|debug)\(|debugger|alert\()/\1\/\/ \2/' -i "$file" || true
        fi
    done
    for file in src/**/*.vue src/*.vue; do
        if [ -f "$file" ] && grep -q '<script>' "$file" && ! grep -q '<script setup' "$file"; then
            sed -i 's/<script>/<script setup>/' "$file" || true
        fi
    done
    PATCH_FILE="$PATCH_DIR/vue_fix_$(date '+%Y%m%d_%H%M%S').diff"
    diff -ruN . . > "$PATCH_FILE" 2>/dev/null || true
    echo "🔖 Patch Vue (fallback) généré : $PATCH_FILE"
    META_FILE="${PATCH_FILE}.json"
    MOD_FILES=$(diff -ruN . . 2>/dev/null | grep '^Only in' || true)
    STAT=$(diff -ruN . . 2>/dev/null | wc -l || true)
    HASH=$(sha256sum "$PATCH_FILE" 2>/dev/null | awk '{print $1}' || echo "")
    cat > "$META_FILE" <<JSON
{ "patch": "$(basename "$PATCH_FILE")", "files": "$(echo "$MOD_FILES" | tr '\n' ',' | sed 's/,$//')", "stat": "$STAT", "hash": "$HASH", "timestamp": "$(date --iso-8601=seconds)" }
JSON
    echo "🔖 Meta JSON (fallback) généré : $META_FILE"
fi
