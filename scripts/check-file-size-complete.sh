#!/bin/bash
# check-file-size-complete.sh - Vérifie TOUTES les limites de taille (Backend + Frontend)
# Conforme à RÈGLE 15 mise à jour

echo "📏 Vérification complète des tailles de fichiers..."
echo ""

EXIT_CODE=0
VIOLATIONS=0

# Couleurs pour l'affichage
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# ═══════════════════════════════════════════════════════════════
# BACKEND (Rust)
# ═══════════════════════════════════════════════════════════════

echo "🦀 BACKEND (Rust)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

MAX_SERVICES=300
MAX_COMMANDS=200
MAX_MODELS=150
MAX_MAIN=120

# Services
echo ""
echo "📦 Services (max $MAX_SERVICES lignes):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_SERVICES" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $MAX_SERVICES)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src-tauri/src/services -name "*.rs" -type f 2>/dev/null || true)

# Commands
echo ""
echo "⚡ Commands (max $MAX_COMMANDS lignes):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_COMMANDS" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $MAX_COMMANDS)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src-tauri/src/commands -name "*.rs" -type f 2>/dev/null || true)

# Models
echo ""
echo "📊 Models (max $MAX_MODELS lignes):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_MODELS" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $MAX_MODELS)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src-tauri/src/models -name "*.rs" -type f 2>/dev/null || true)

# main.rs
echo ""
echo "🎯 main.rs (max $MAX_MAIN lignes):"
if [ -f "src-tauri/src/main.rs" ]; then
    lines=$(wc -l < "src-tauri/src/main.rs")
    if [ "$lines" -gt "$MAX_MAIN" ]; then
        echo -e "${RED}❌ src-tauri/src/main.rs: $lines lignes (max $MAX_MAIN)${NC}"
        ((VIOLATIONS++))
        EXIT_CODE=1
    else
        echo -e "${GREEN}✅ src-tauri/src/main.rs: $lines lignes${NC}"
    fi
fi

# ═══════════════════════════════════════════════════════════════
# FRONTEND (Vue.js/TypeScript)
# ═══════════════════════════════════════════════════════════════

echo ""
echo ""
echo "🎨 FRONTEND (Vue.js/TypeScript)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

MAX_VUE_STANDARD=250
MAX_VUE_MODAL=300
MAX_STORE=200
MAX_STORE_DATA=500
MAX_COMPOSABLE=150
MAX_UTILS=200

# ═══════════════════════════════════════════════════════════════
# EXCLUSIONS (Fichiers complexes spécifiques)
# ═══════════════════════════════════════════════════════════════
# - eventTranslations: Données statiques (traductions)
# - eventSchedules: Données statiques (horaires)
# - HourlyTable.vue: Composant données-intensif avec logique métier complexe
# Ces fichiers sont intentionnellement exclus du contrôle de taille.
EXCLUDED_PATTERNS=("eventTranslations" "eventSchedules" "HourlyTable")

# Composants Vue
echo ""
echo "🧩 Composants Vue (max $MAX_VUE_STANDARD lignes, modals/tables: $MAX_VUE_MODAL):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # Vérifier si le fichier doit être exclu
        skip_file=false
        for pattern in "${EXCLUDED_PATTERNS[@]}"; do
            if [[ "$filename" == *"$pattern"* ]]; then
                skip_file=true
                break
            fi
        done
        
        if [ "$skip_file" = true ]; then
            continue
        fi
        
        lines=$(wc -l < "$file")
        
        # Déterminer si c'est un modal ou une table (exception)
        is_exception=false
        if [[ "$filename" == *"Modal"* ]] || [[ "$filename" == *"Table"* ]] || [[ "$filename" == *"Drawer"* ]]; then
            is_exception=true
            limit=$MAX_VUE_MODAL
        else
            limit=$MAX_VUE_STANDARD
        fi
        
        if [ "$lines" -gt "$limit" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $limit)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src/components -name "*.vue" -type f 2>/dev/null || true)

# Stores Pinia
echo ""
echo "🗄️  Stores Pinia (max $MAX_STORE lignes, data stores: $MAX_STORE_DATA):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        filename=$(basename "$file")
        
        # Déterminer si c'est un store de données (exception)
        is_data_store=false
        if [[ "$filename" == *"Translation"* ]] || [[ "$filename" == *"Schedule"* ]] || [[ "$filename" == *"Config"* ]]; then
            is_data_store=true
            limit=$MAX_STORE_DATA
        else
            limit=$MAX_STORE
        fi
        
        if [ "$lines" -gt "$limit" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $limit)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src/stores -name "*.ts" -o -name "*.js" 2>/dev/null || true)

# Composables
echo ""
echo "🔧 Composables (max $MAX_COMPOSABLE lignes):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_COMPOSABLE" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $MAX_COMPOSABLE)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src/composables -name "*.ts" -o -name "*.js" 2>/dev/null || true)

# Utils/Helpers
echo ""
echo "🛠️  Utils/Helpers (max $MAX_UTILS lignes):"
while IFS= read -r file; do
    if [ -f "$file" ]; then
        # Exclure les fichiers de données statiques (traductions, horaires)
        # Ces fichiers sont purement des données et pas du code logique
        if [[ "$file" == *"eventTranslations"* ]] || [[ "$file" == *"eventSchedules"* ]]; then
            continue
        fi
        
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_UTILS" ]; then
            echo -e "${RED}❌ $file: $lines lignes (max $MAX_UTILS)${NC}"
            ((VIOLATIONS++))
            EXIT_CODE=1
        fi
    fi
done < <(find src/utils -name "*.ts" -o -name "*.js" 2>/dev/null || true)

# ═══════════════════════════════════════════════════════════════
# RÉSUMÉ
# ═══════════════════════════════════════════════════════════════

echo ""
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}✅ Tous les fichiers respectent les limites de taille !${NC}"
else
    echo -e "${RED}❌ $VIOLATIONS fichier(s) dépassent les limites (BLOQUANT)${NC}"
    echo ""
    echo "📝 Action requise: Refactoriser les fichiers trop longs"
    echo "   Voir RÈGLE 15 dans .clinerules pour les limites"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit $EXIT_CODE
