#!/bin/bash
# check-french-naming.sh - Vérifie que les fonctions sont nommées en français (RÈGLE 2)

set -e

echo "🇫🇷 Vérification du nommage français des fonctions..."
echo ""

EXIT_CODE=0
VIOLATIONS=0

# Liste des mots anglais courants à détecter dans les noms de fonctions
ENGLISH_WORDS=(
    "calculate" "compute" "analyze" "process" "convert" "parse" "format"
    "extract" "detect" "normalize" "import" "export" "clean" "validate"
    "check" "verify" "count" "add" "remove" "delete" "update" "create"
    "get" "set" "fetch" "load" "save" "store" "find" "search"
)

# Exceptions autorisées (traits Rust standards, méthodes obligatoires)
EXCEPTIONS=(
    "fn default()"
    "fn from("
    "fn fmt("
    "fn clone("
    "fn new("
    "fn build("
)

# Fonction pour vérifier si une ligne est une exception
is_exception() {
    local line="$1"
    for exception in "${EXCEPTIONS[@]}"; do
        if [[ "$line" == *"$exception"* ]]; then
            return 0
        fi
    done
    return 1
}

# Rechercher les fonctions avec des mots anglais
echo "🔍 Recherche de fonctions avec noms anglais..."
echo ""

for word in "${ENGLISH_WORDS[@]}"; do
    # Rechercher les fonctions contenant ce mot anglais
    while IFS= read -r line; do
        # Ignorer les lignes de test
        if [[ "$line" == *"#[test]"* ]] || [[ "$line" == *"mod tests"* ]]; then
            continue
        fi
        
        # Vérifier si c'est une exception autorisée
        if is_exception "$line"; then
            continue
        fi
        
        # Extraire le nom du fichier et la ligne
        file=$(echo "$line" | cut -d: -f1)
        content=$(echo "$line" | cut -d: -f2-)
        
        echo "❌ $file"
        echo "   $content"
        echo ""
        
        ((VIOLATIONS++))
        EXIT_CODE=1
        
    done < <(grep -rn "fn .*${word}" src-tauri/src --include="*.rs" 2>/dev/null || true)
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $VIOLATIONS -eq 0 ]; then
    echo "✅ Aucune violation détectée - Toutes les fonctions sont en français !"
else
    echo "❌ $VIOLATIONS violations détectées"
    echo ""
    echo "📝 RAPPEL RÈGLE 2 (CRITIQUE):"
    echo "   Toutes les fonctions DOIVENT être nommées en FRANÇAIS"
    echo ""
    echo "   Exemples de corrections:"
    echo "   ❌ calculate_average() → ✅ calculer_moyenne()"
    echo "   ❌ parse_csv_file()    → ✅ parser_fichier_csv()"
    echo "   ❌ convert_to_json()   → ✅ convertir_en_json()"
    echo ""
    echo "   Exceptions autorisées:"
    echo "   - Traits Rust standards: default(), from(), fmt(), clone()"
    echo "   - Constructeurs: new()"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit $EXIT_CODE
