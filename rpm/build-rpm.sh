#!/bin/bash
# Script de construction du paquet RPM
# Usage: bash build-rpm.sh

set -e

echo "🔧 Construction du paquet RPM - Analyses-Historiques"
echo "====================================================="
echo ""

# Vérifier les prérequis
echo "✓ Vérification des prérequis..."
for cmd in cargo npm rpmbuild; do
    if ! command -v $cmd &> /dev/null; then
        echo "❌ $cmd n'est pas installé"
        echo "   Fedora: sudo dnf install rpm-build nodejs npm cargo"
        exit 1
    fi
done
echo "✅ Tous les prérequis sont présents"
echo ""

# Configuration
SPEC_FILE="analyses-historiques.spec"
BUILDDIR="$HOME/rpmbuild"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)/.."

echo "📁 Configuration:"
echo "   Répertoire du projet: $PROJECT_DIR"
echo "   Répertoire rpmbuild: $BUILDDIR"
echo "   Fichier spec: $SPEC_FILE"
echo ""

# Créer la structure rpmbuild s'il n'existe pas
echo "📦 Création de la structure rpmbuild..."
mkdir -p "$BUILDDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS,BUILDROOT}

# Copier le fichier .spec
echo "📄 Copie du fichier spec..."
cp "$SPEC_FILE" "$BUILDDIR/SPECS/"

# Copier les scripts et configs
echo "📋 Copie des scripts et configurations..."
cp post-install.sh "$BUILDDIR/SPECS/"
cp config.example.toml "$BUILDDIR/SPECS/"

# Compiler l'application
echo ""
echo "🔨 Compilation du projet..."
echo "   (Cette étape peut prendre 5-10 minutes...)"

cd "$PROJECT_DIR"

# Installer les dépendances npm
echo "1️⃣  Installation des dépendances npm..."
npm install --legacy-peer-deps > /dev/null 2>&1 || npm install > /dev/null 2>&1

# Compiler le frontend
echo "2️⃣  Compilation du frontend Vue..."
npm run build

# Créer une configuration Tauri PRODUCTION (sans devUrl, avec chemins assets complets)
echo "3️⃣  Configuration de tauri.conf.json pour mode PRODUCTION RPM..."
cp src-tauri/tauri.conf.json src-tauri/tauri.conf.json.dev

# Créer le fichier production - devUrl à vide pour forcer Tauri à utiliser frontendDist
cat > src-tauri/tauri.conf.json.prod << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "tauri-app",
  "version": "0.1.0",
  "identifier": "com.rono.tauri-app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Volatility Analyzer",
        "width": 1400,
        "height": 900,
        "decorations": false
      }
    ],
    "security": {
      "csp": null,
      "dangerousDisableAssetCspModification": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
EOF

cp src-tauri/tauri.conf.json.prod src-tauri/tauri.conf.json

# Nettoyer le cache Cargo pour forcer recompilation
echo "4️⃣  Nettoyage du cache Cargo..."
cd src-tauri
cargo clean > /dev/null 2>&1

# Compiler le backend Rust en RELEASE avec nouveau tauri.conf.json
echo "5️⃣  Compilation du backend Rust (RELEASE)..."
cargo build --release 2>&1 | grep -E "Compiling|Finished|error"
BUILD_RESULT=$?

# Restaurer tauri.conf.json original
mv tauri.conf.json.dev tauri.conf.json
rm -f tauri.conf.json.prod
cd ..

if [ $BUILD_RESULT -ne 0 ]; then
    echo "❌ Erreur lors de la compilation du backend"
    exit 1
fi

echo "✅ Compilation réussie"
echo ""

# Copier le binaire compilé vers le répertoire du projet (pour que RPM le trouve)
echo "📋 Copie du binaire compilé..."
cp "$PROJECT_DIR/src-tauri/target/release/analyses-historiques-volatility" "$PROJECT_DIR/" || {
    echo "❌ Erreur: Binaire compilé non trouvé"
    exit 1
}

# Construire le paquet RPM
echo "📦 Construction du paquet RPM..."
cd "$BUILDDIR"

rpmbuild -bb SPECS/$SPEC_FILE \
    --define="_topdir $BUILDDIR" \
    --define="_builddir $PROJECT_DIR" \
    --define="_sourcedir $PROJECT_DIR" \
    --define="_distro fedora" || {
    echo "❌ Erreur lors de la construction du RPM"
    echo "   Vérifiez les logs: cat $BUILDDIR/BUILD/*/build.log"
    exit 1
}

echo "✅ Paquet RPM construit avec succès !"
echo ""

# Afficher les fichiers RPM générés
echo "📦 Fichiers RPM générés:"
find "$BUILDDIR/RPMS" -name "*.rpm" -exec ls -lh {} \; | awk '{print "   " $5 " " $9}'

echo ""
echo "🚀 Installation du paquet:"
echo "   sudo rpm -ivh $(find $BUILDDIR/RPMS -name "*.rpm" | head -1)"
echo ""
echo "📖 Documentation: cat rpm/README.md"
echo ""

