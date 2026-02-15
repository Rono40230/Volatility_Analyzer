#!/bin/bash
# Script de setup optionnel pour les données

set -e

APP_DATA_DIR="${APP_DATA_DIR:-/var/lib/analyses-historiques}"
DOWNLOAD_URL="https://example.com/data"  # À remplacer par votre URL

echo "🔧 Setup des données pour Analyses-Historiques"
echo "=============================================="
echo ""

# Vérifier si les données existent déjà
if [ -d "$APP_DATA_DIR/data" ] && [ "$(ls -A $APP_DATA_DIR/data)" ]; then
    echo "✅ Les données semblent déjà présentes dans $APP_DATA_DIR/data"
    echo ""
    read -p "Voulez-vous continuer quand même ? (y/n) " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 0
fi

echo "📁 Préparation des répertoires..."
mkdir -p "$APP_DATA_DIR/data"
mkdir -p "$APP_DATA_DIR/db"
mkdir -p "$APP_DATA_DIR/imports"

echo "✅ Répertoires créés"
echo ""

# Option 1: Importer depuis un dossier local
echo "Choisissez une option :"
echo "1. Importer depuis un dossier local (CSV Dukascopy)"
echo "2. Télécharger les données (URL)"
echo "3. Passer pour l'instant"
echo ""
read -p "Votre choix (1-3) : " choice

case $choice in
    1)
        echo ""
        read -p "Chemin du dossier contenant les CSV : " csv_folder
        if [ -d "$csv_folder" ]; then
            echo "📂 Copie des fichiers CSV..."
            cp -v "$csv_folder"/*.csv "$APP_DATA_DIR/data/" 2>/dev/null || {
                echo "⚠️  Aucun fichier CSV trouvé dans $csv_folder"
            }
            echo "✅ Fichiers copiés"
        else
            echo "❌ Le dossier $csv_folder n'existe pas"
        fi
        ;;
    2)
        echo ""
        echo "⚠️  Téléchargement non configuré pour cette version"
        echo "    Configurez DOWNLOAD_URL dans ce script"
        ;;
    3)
        echo "👋 Vous pourrez copier les données manuellement dans :"
        echo "   $APP_DATA_DIR/data/"
        ;;
esac

echo ""
echo "✅ Setup terminé !"
echo ""
echo "📖 Documentation :"
echo "   - Placer les CSV dans : $APP_DATA_DIR/data/"
echo "   - Lancer l'app : analyses-historiques-gui"
echo ""
