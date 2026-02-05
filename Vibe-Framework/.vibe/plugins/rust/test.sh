#!/bin/bash
# Plugin Rust - Test
# Exécute les tests unitaires puis clippy strict (échec si warnings clippy)
TARGET_DIR="."
if [ -f "src-tauri/Cargo.toml" ]; then TARGET_DIR="src-tauri"; fi

echo "🔎 Rust tests in $TARGET_DIR"
cd "$TARGET_DIR" || exit 1

# 1) Unit tests
if ! cargo test --all --color=always; then
    echo "❌ cargo test failed"
    exit 1
fi

# 2) Clippy strict (fail on warnings)
if ! cargo clippy --all -- -D warnings; then
    echo "❌ cargo clippy failed (warnings treated as errors)"
    exit 1
fi

echo "✅ Rust tests & clippy OK"
exit 0
