# Makefile 2.0 - Compatible Antigravity
# Ce fichier fait le pont entre vos habitudes (make) et le nouveau système (scripts)

.PHONY: help dev check validate all pre-commit

# Affiche l'aide
help:
	@echo "════════════════════════════════════════════════"
	@echo "   COMMANDES DU PROJET (Antigravity Powered)"
	@echo "════════════════════════════════════════════════"
	@echo ""
	@echo "  make dev          - Lance l'app (Frontend + Backend)"
	@echo "  make check        - Vérifie la qualité (Taille, Unwrap, etc.)"
	@echo "  make validate     - Lance la validation complète (Phase 2)"
	@echo "  make pre-commit   - Vérifie tout avant commit (RECOMMANDÉ)"
	@echo ""
	@echo "════════════════════════════════════════════════"

# Lance le développement (Vue + Tauri)
dev:
	@echo "🔥 Lancement de l'environnement de dev..."
	npm run tauri dev

# Vérifie la qualité (Appelle le nouveau script)
check:
	@./scripts/impact-detection/check-quality.sh

# Validation complète (Appelle le script Phase 2)
validate:
	@./scripts/impact-detection/validate-phase2.sh

# Pre-commit checks (Alias pour check)
pre-commit: check

# Alias pour check
check-rules: check
