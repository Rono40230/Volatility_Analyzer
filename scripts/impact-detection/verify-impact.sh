#!/bin/bash
# verify-impact.sh - Vérifie l'impact des changements
# Compare snapshot vs état courant, détecte changements inattendus

set -e

SNAPSHOTS_DIR=".git/.snapshots"
SNAPSHOT_FILE=$(ls -t "$SNAPSHOTS_DIR"/pre-phase2-state-*.json 2>/dev/null | head -1)

if [ -z "$SNAPSHOT_FILE" ]; then
    echo "❌ Aucun snapshot trouvé. Exécute d'abord : ./scripts/impact-detection/snapshot-dependencies.sh"
    exit 1
fi

echo "🔍 Vérification de l'impact des changements..."
echo ""

# Extraire les données du snapshot
SNAPSHOT_FILES=$(jq -r '.file_hashes[]' "$SNAPSHOT_FILE" 2>/dev/null | cut -d: -f1)
SNAPSHOT_TIME=$(jq -r '.timestamp' "$SNAPSHOT_FILE")

# Fichiers actuels
CURRENT_FILES=$(find src-tauri/src -name "*.rs" -type f | sort)

# Comparer les fichiers
MODIFIED_FILES=""
UNCHANGED_FILES=""
DELETED_FILES=""
NEW_FILES=""

echo "📋 Comparaison des fichiers..."
echo ""

# Vérifier chaque fichier du snapshot
while IFS= read -r snapshot_file; do
    [ -z "$snapshot_file" ] && continue
    
    if [ ! -f "$snapshot_file" ]; then
        DELETED_FILES="$DELETED_FILES
$snapshot_file"
    else
        # Vérifier si le hash a changé
        SNAPSHOT_HASH=$(jq -r '.file_hashes[]' "$SNAPSHOT_FILE" | grep "^$snapshot_file:" | cut -d: -f2)
        CURRENT_HASH=$(md5sum "$snapshot_file" | cut -d" " -f1)
        
        if [ "$SNAPSHOT_HASH" != "$CURRENT_HASH" ]; then
            MODIFIED_FILES="$MODIFIED_FILES
$snapshot_file"
        else
            UNCHANGED_FILES="$UNCHANGED_FILES
$snapshot_file"
        fi
    fi
done <<< "$SNAPSHOT_FILES"

# Chercher les nouveaux fichiers
while IFS= read -r current_file; do
    [ -z "$current_file" ] && continue
    
    if ! echo "$SNAPSHOT_FILES" | grep -q "^$current_file$"; then
        NEW_FILES="$NEW_FILES
$current_file"
    fi
done <<< "$CURRENT_FILES"

# Compter les fichiers modifiés
MODIFIED_COUNT=$(echo "$MODIFIED_FILES" | grep -c . || echo 0)
UNCHANGED_COUNT=$(echo "$UNCHANGED_FILES" | grep -c . || echo 0)
DELETED_COUNT=$(echo "$DELETED_FILES" | grep -c . || echo 0)
NEW_COUNT=$(echo "$NEW_FILES" | grep -c . || echo 0)

# Afficher le rapport
echo "✅ FICHIERS INCHANGÉS : $UNCHANGED_COUNT"
echo ""

if [ $MODIFIED_COUNT -gt 0 ]; then
    echo "📝 FICHIERS MODIFIÉS : $MODIFIED_COUNT (ATTENDU)"
    echo "$MODIFIED_FILES" | grep . | sed 's/^/   /'
    echo ""
fi

if [ $NEW_COUNT -gt 0 ]; then
    echo "✨ NOUVEAUX FICHIERS : $NEW_COUNT (ATTENDU)"
    echo "$NEW_FILES" | grep . | sed 's/^/   /'
    echo ""
fi

if [ $DELETED_COUNT -gt 0 ]; then
    echo "🗑️  FICHIERS SUPPRIMÉS : $DELETED_COUNT (ATTENDU)"
    echo "$DELETED_FILES" | grep . | sed 's/^/   /'
    echo ""
fi

# Vérifier les modules affectés (transitifs)
echo "🔗 Vérification des dépendances transitives..."
echo ""

# Les fichiers modifiés
AFFECTED_MODULES=$(echo "$MODIFIED_FILES" | grep . | xargs -I {} sh -c 'basename {} .rs' | sort -u)

# Trouver tous les modules qui DÉPENDENT des modules modifiés
DEPENDENT_MODULES=""
while IFS= read -r module; do
    [ -z "$module" ] && continue
    # Chercher les fichiers qui importent ce module
    DEPENDENTS=$(grep -r "use crate::.*::$module" src-tauri/src --include="*.rs" | cut -d: -f1 | xargs -I {} basename {} .rs | sort -u)
    DEPENDENT_MODULES="$DEPENDENT_MODULES
$DEPENDENTS"
done <<< "$AFFECTED_MODULES"

DEPENDENT_COUNT=$(echo "$DEPENDENT_MODULES" | grep -c . || echo 0)

if [ $DEPENDENT_COUNT -gt 0 ]; then
    echo "📊 Modules affectés transitifs (testables) : $DEPENDENT_COUNT"
    echo "$DEPENDENT_MODULES" | grep . | sort -u | sed 's/^/   /'
    echo ""
fi

echo "═══════════════════════════════════════════"
echo "📊 RÉSUMÉ DE L'IMPACT"
echo "═══════════════════════════════════════════"
echo "Fichiers inchangés   : $UNCHANGED_COUNT ✅"
echo "Fichiers modifiés    : $MODIFIED_COUNT (à tester)"
echo "Fichiers nouveaux    : $NEW_COUNT (à tester)"
echo "Fichiers supprimés   : $DELETED_COUNT"
echo "Modules transitifs   : $DEPENDENT_COUNT (à vérifier)"
echo "═══════════════════════════════════════════"
echo ""

# Sauvegarder le rapport d'impact
IMPACT_FILE="$SNAPSHOTS_DIR/impact-report.txt"
{
    echo "RAPPORT D'IMPACT - $(date)"
    echo "═══════════════════════════════════════════"
    echo ""
    echo "FICHIERS MODIFIÉS : $MODIFIED_COUNT"
    echo "$MODIFIED_FILES" | grep . || echo "Aucun"
    echo ""
    echo "FICHIERS NOUVEAUX : $NEW_COUNT"
    echo "$NEW_FILES" | grep . || echo "Aucun"
    echo ""
    echo "MODULES AFFECTÉS TRANSITIFS : $DEPENDENT_COUNT"
    echo "$DEPENDENT_MODULES" | grep . | sort -u || echo "Aucun"
    echo ""
} > "$IMPACT_FILE"

echo "✅ Rapport d'impact sauvegardé : $IMPACT_FILE"
exit 0
