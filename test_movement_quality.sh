#!/bin/bash
# Test script pour vérifier que le Tauri command analyze_movement_quality fonctionne

echo "🧪 Test: Movement Quality Analysis"
echo "===================================="
echo ""

# Attendre que l'app soit prête (si elle se lance)
sleep 5

# Afficher les logs de la console
echo "📋 Logs disponibles:"
echo "   Vérifiez la console DevTools (F12) pour voir les messages:"
echo "   🔄 Chargement qualité mouvement: SYMBOL_EVENT"
echo "   📤 Appel Tauri: analyze_movement_quality(SYMBOL, EVENT)"
echo "   ✅ Réponse reçue: { quality_score: X, ... }"
echo ""
echo "⚠️  Si vous voyez une erreur, vérifiez:"
echo "   1. La base de données pairs.db existe et a des données"
echo "   2. Le symbol et event_type existent"
echo "   3. Les logs côté backend (console Tauri)"
echo ""

