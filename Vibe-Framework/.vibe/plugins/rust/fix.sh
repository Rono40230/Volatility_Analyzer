#!/bin/bash
# Plugin Rust - Fix
# Génère un patch contenant les modifications de formatting/lint via un worktree Git
VIBE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PATCH_DIR="$VIBE_ROOT/.vibe/patches"
mkdir -p "$PATCH_DIR"

if [ -f "Cargo.toml" ] || [ -f "src-tauri/Cargo.toml" ]; then
    TARGET_DIR="."
    if [ -f "src-tauri/Cargo.toml" ]; then TARGET_DIR="src-tauri"; fi

    # Utiliser un worktree temporaire pour ne pas modifier l'arbre de travail courant
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        WORKTREE_DIR=$(mktemp -d)
        git worktree add "$WORKTREE_DIR" HEAD >/dev/null 2>&1 || { rm -rf "$WORKTREE_DIR"; echo "⚠️ git worktree failed, fallback to in-place fixes"; }
        if [ -d "$WORKTREE_DIR" ]; then
            (cd "$WORKTREE_DIR/$TARGET_DIR" && cargo fmt --quiet || true)
            (cd "$WORKTREE_DIR/$TARGET_DIR" && cargo clippy --fix --allow-dirty --allow-staged --quiet -- -D warnings 2>/dev/null || true)
            # Générer la patch depuis le worktree contre HEAD
            PATCH_FILE="$PATCH_DIR/rust_fix_$(date '+%Y%m%d_%H%M%S').patch"
            (cd "$WORKTREE_DIR" && git add -A && git diff --binary HEAD) > "$PATCH_FILE" || true
            echo "🔖 Patch Rust généré : $PATCH_FILE"
            # Générer un résumé JSON pour le patch
            META_FILE="${PATCH_FILE}.json"
            MOD_FILES=$(cd "$WORKTREE_DIR" && git diff --name-only HEAD || true)
            STAT=$(cd "$WORKTREE_DIR" && git diff --shortstat HEAD || true)
            HASH=$(sha256sum "$PATCH_FILE" 2>/dev/null | awk '{print $1}' || echo "")
            cat > "$META_FILE" <<JSON
{ "patch": "$(basename "$PATCH_FILE")", "files": "$(echo "$MOD_FILES" | tr '\n' ',' | sed 's/,$//')", "stat": "$(echo $STAT | sed 's/"/\"/g')", "hash": "$HASH", "timestamp": "$(date --iso-8601=seconds)" }
JSON
            echo "🔖 Meta JSON généré : $META_FILE"
            # Nettoyer worktree
            git worktree remove --force "$WORKTREE_DIR" >/dev/null 2>&1 || rm -rf "$WORKTREE_DIR"
        fi
    else
        # Pas de git -> appliquer en local et générer diff simple
        (cd "$TARGET_DIR" && cargo fmt --quiet || true)
        (cd "$TARGET_DIR" && cargo clippy --fix --allow-dirty --allow-staged --quiet -- -D warnings 2>/dev/null || true)
        PATCH_FILE="$PATCH_DIR/rust_fix_$(date '+%Y%m%d_%H%M%S').diff"
        diff -ruN . "$TARGET_DIR" > "$PATCH_FILE" 2>/dev/null || true
        echo "🔖 Patch Rust (fallback) généré : $PATCH_FILE"
        META_FILE="${PATCH_FILE}.json"
        MOD_FILES=$(diff -ruN . "$TARGET_DIR" 2>/dev/null | grep '^Only in' || true)
        STAT=$(diff -ruN . "$TARGET_DIR" 2>/dev/null | wc -l || true)
        HASH=$(sha256sum "$PATCH_FILE" 2>/dev/null | awk '{print $1}' || echo "")
        cat > "$META_FILE" <<JSON
    { "patch": "$(basename "$PATCH_FILE")", "files": "$(echo "$MOD_FILES" | tr '\n' ',' | sed 's/,$//')", "stat": "$STAT", "hash": "$HASH", "timestamp": "$(date --iso-8601=seconds)" }
    JSON
        echo "🔖 Meta JSON (fallback) généré : $META_FILE"
    fi
fi
