#!/bin/bash
# check-eslint.sh - Audit ESLint pour détecter code mort et violations qualité
# Intégré dans le système d'audit global

echo "🔍 AUDIT ESLINT (Code mort & Qualité)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

EXIT_CODE=0

# Vérifier si ESLint est installé
if ! command -v npx &> /dev/null; then
    echo "❌ npm/npx non disponible"
    echo "   Installer Node.js pour utiliser ESLint"
    exit 1
fi

if [ ! -f "node_modules/.bin/eslint" ]; then
    echo "⚠️  ESLint non installé"
    echo "   Exécuter: npm install"
    exit 1
fi

# Lancer ESLint
echo "📋 Analyse du code frontend avec ESLint..."
echo ""

# Créer un fichier temporaire pour le rapport
REPORT_FILE=$(mktemp)

# Lancer ESLint avec format JSON pour parsing
npx eslint src/ --ext .vue,.ts,.js --format json --output-file "$REPORT_FILE" 2>/dev/null || true

# Parser le rapport JSON
if [ -f "$REPORT_FILE" ]; then
    # Compter les erreurs et warnings
    ERRORS=$(cat "$REPORT_FILE" | grep -o '"errorCount":[0-9]*' | cut -d: -f2 | awk '{s+=$1} END {print s}')
    WARNINGS=$(cat "$REPORT_FILE" | grep -o '"warningCount":[0-9]*' | cut -d: -f2 | awk '{s+=$1} END {print s}')
    
    # Valeurs par défaut si vide
    ERRORS=${ERRORS:-0}
    WARNINGS=${WARNINGS:-0}
    
    # Afficher le résumé
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 RÉSUMÉ ESLINT"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
        echo "✅ Aucune violation détectée !"
        echo ""
    else
        if [ "$ERRORS" -gt 0 ]; then
            echo "❌ Erreurs : $ERRORS"
            EXIT_CODE=1
        fi
        
        if [ "$WARNINGS" -gt 0 ]; then
            echo "⚠️  Warnings : $WARNINGS"
        fi
        echo ""
        
        # Afficher le détail avec format lisible
        echo "📝 Détails des violations :"
        echo ""
        npx eslint src/ --ext .vue,.ts,.js --format stylish | head -100
        
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "💡 Pour corriger automatiquement ce qui est possible :"
        echo "   npx eslint src/ --ext .vue,.ts,.js --fix"
        echo ""
        echo "📖 Pour voir le rapport complet :"
        echo "   npx eslint src/ --ext .vue,.ts,.js"
        echo ""
    fi
    
    rm -f "$REPORT_FILE"
else
    echo "❌ Erreur lors de l'exécution d'ESLint"
    EXIT_CODE=1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit $EXIT_CODE
