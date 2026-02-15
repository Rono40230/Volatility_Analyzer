#!/bin/bash
# Script simple pour construire le RPM Analyses-Historiques
# Suppose que le binaire est déjà compilé en release

set -e

echo "🔧 Construction du paquet RPM - Analyses-Historiques"
echo "====================================================="
echo ""

# Vérifier les prérequis
echo "✓ Vérification des prérequis..."
for cmd in rpmbuild; do
    if ! command -v $cmd &> /dev/null; then
        echo "❌ $cmd n'est pas installé"
        exit 1
    fi
done
echo "✅ Tous les prérequis sont présents"
echo ""

# Configuration
SPEC_FILE="analyses-historiques.spec"
BUILDDIR="$HOME/rpmbuild"
RPM_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$RPM_DIR/.."

echo "📁 Configuration:"
echo "   Répertoire RPM: $RPM_DIR"
echo "   Répertoire du projet: $PROJECT_DIR"
echo "   Fichier spec: $SPEC_FILE"
echo ""

# Créer la structure rpmbuild s'il n'existe pas
echo "📦 Création de la structure rpmbuild..."
mkdir -p "$BUILDDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS,BUILDROOT}

# Copier le fichier .spec
echo "📄 Copie du fichier spec..."
cp "$RPM_DIR/$SPEC_FILE" "$BUILDDIR/SPECS/"

# Copier les scripts et configs
echo "📋 Copie des scripts et configurations..."
cp "$RPM_DIR/post-install.sh" "$BUILDDIR/SPECS/"
cp "$RPM_DIR/config.example.toml" "$BUILDDIR/SPECS/"

# Vérifier que le binaire existe
echo ""
echo "📦 Vérification du binaire compilé..."
if [ ! -f "$PROJECT_DIR/src-tauri/target/release/analyses-historiques-volatility" ]; then
    echo "❌ ERREUR: Le binaire n'est pas compilé!"
    echo "   Compilez d'abord: cd $PROJECT_DIR && npm run build && cd src-tauri && cargo build --release"
    exit 1
fi
echo "✅ Binaire trouvé"

# Compiler le frontend (toujours, pour s'assurer que les changements sont pris en compte)
echo ""
echo "📦 Compilation du frontend..."
cd "$PROJECT_DIR"
rm -rf dist .vite
npm run build

# Copier le binaire vers le répertoire source pour que RPM le trouve
echo ""
echo "📋 Copie du binaire..."
cp "$PROJECT_DIR/src-tauri/target/release/analyses-historiques-volatility" "$PROJECT_DIR/"

# Construire le paquet RPM
echo "📦 Construction du paquet RPM..."
cd "$BUILDDIR"

rpmbuild -bb SPECS/$SPEC_FILE \
    --define="_topdir $BUILDDIR" \
    --define="_builddir $PROJECT_DIR" \
    --define="_sourcedir $PROJECT_DIR" \
    --define="_distro fedora" || {
    echo "❌ Erreur lors de la construction du RPM"
    exit 1
}

echo "✅ Paquet RPM construit avec succès !"
echo ""

# Afficher les fichiers RPM générés
echo "📦 Fichiers RPM générés:"
find "$BUILDDIR/RPMS" -name "*.rpm" -exec ls -lh {} \; | awk '{print "   " $5 " " $9}'

echo ""
echo "🚀 Installation du paquet:"
echo "   sudo rpm -ivh --force $(find $BUILDDIR/RPMS -name "*.rpm" | head -1)"
echo ""
