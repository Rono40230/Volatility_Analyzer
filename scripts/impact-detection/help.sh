#!/bin/bash
# vibe-coding-help.sh - Aide rapide pour le système vibe-proof

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                      🎯 AIDE SYSTÈME VIBE-PROOF"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

show_menu() {
    echo "Choisir une action :"
    echo ""
    echo "  1) 📖 Lire le guide rapide (QUICK_START_5MIN.md)"
    echo "  2) 🚀 Initialiser le système (première utilisation)"
    echo "  3) ✅ Valider et committer (Phase 2)"
    echo "  4) 📊 Voir le rapport d'impact"
    echo "  5) 🧪 Voir le rapport de régression"
    echo "  6) ℹ️  Voir la documentation complète"
    echo "  7) 📝 Voir les 17 règles (.clinerules)"
    echo "  8) 🔄 Réinitialiser (recommencer Phase 1)"
    echo "  0) ❌ Quitter"
    echo ""
}

while true; do
    show_menu
    read -p "Choix (0-8) : " choice
    
    case $choice in
        1)
            echo ""
            cat QUICK_START_5MIN.md | less
            ;;
        2)
            echo ""
            ./scripts/impact-detection/init-impact-system.sh
            ;;
        3)
            echo ""
            ./scripts/impact-detection/validate-phase2.sh
            ;;
        4)
            echo ""
            if [ -f .git/.snapshots/impact-report.txt ]; then
                cat .git/.snapshots/impact-report.txt
            else
                echo "❌ Aucun rapport d'impact. Exécute Phase 2 d'abord."
            fi
            ;;
        5)
            echo ""
            if [ -f .git/.snapshots/regression-report.txt ]; then
                cat .git/.snapshots/regression-report.txt
            else
                echo "❌ Aucun rapport de régression. Exécute Phase 2 d'abord."
            fi
            ;;
        6)
            echo ""
            cat SYSTEM_COMPLETE_VIBE_PROOF.md | less
            ;;
        7)
            echo ""
            cat .clinerules | tail -100 | head -50
            ;;
        8)
            echo ""
            echo "Réinitialisation..."
            ./scripts/impact-detection/init-impact-system.sh
            ;;
        0)
            echo ""
            echo "Au revoir ! 👋"
            exit 0
            ;;
        *)
            echo "❌ Choix invalide"
            ;;
    esac
    
    echo ""
    read -p "Appuie sur Entrée pour continuer..."
    clear
done
