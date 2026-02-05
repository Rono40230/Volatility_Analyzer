#!/bin/bash
# Script de lancement avec correctifs pour l'environnement Fedora 43 / Rust 1.89

# Augmentation drastique de la taille de la pile pour éviter les SIGSEGV du compilateur
export RUST_MIN_STACK=1073741824

# Fixes graphiques pour éviter l'écran blanc (WebView Linux)
export GDK_BACKEND=x11
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1 


export CARGO_BUILD_JOBS=1

# Nettoyage préventif si nécessaire (décommenter si le build échoue encore)
# cargo clean --manifest-path src-tauri/Cargo.toml

echo "🚀 Démarrage de l'application..."
echo "ℹ️  Stack Size configurée à : $RUST_MIN_STACK"

# Lancement du mode dev Tauri
npm run tauri dev
