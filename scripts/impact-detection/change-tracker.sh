#!/bin/bash
# change-tracker.sh - Enregistre les changements accumulés (Phase 1)
# Utile pour savoir combien de changements ont été accumulés

SNAPSHOTS_DIR=".git/.snapshots"
CHANGE_LOG="$SNAPSHOTS_DIR/change-log.txt"

mkdir -p "$SNAPSHOTS_DIR"

# Si c'est la première accumulation, créer le fichier
if [ ! -f "$CHANGE_LOG" ]; then
    echo "ACCUMULATION COMMENCÉE - $(date)" > "$CHANGE_LOG"
    echo "" >> "$CHANGE_LOG"
fi

# Ajouter un nouveau changement au log
CHANGE_COUNT=$(grep -c "^CHANGEMENT" "$CHANGE_LOG" || echo 0)
CHANGE_NUM=$((CHANGE_COUNT + 1))

{
    echo "CHANGEMENT #$CHANGE_NUM - $(date)"
    echo "  Message : $1"
    echo "  Fichiers modifiés : $(git diff --name-only 2>/dev/null | wc -l)"
    echo ""
} >> "$CHANGE_LOG"

echo "📝 Changement #$CHANGE_NUM enregistré"

# Afficher le statut
echo ""
echo "📊 Accumulation en cours :"
echo "  Changements accumulés : $CHANGE_NUM"
echo "  Fichiers modifiés au total : $(git diff --name-only 2>/dev/null | wc -l)"
echo "  Fichiers non-committtés : $(git status --short 2>/dev/null | wc -l)"
echo ""

exit 0
