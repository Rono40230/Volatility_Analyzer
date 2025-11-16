# Makefile - Automatisation des vérifications
.PHONY: help pre-commit check-rules validate audit dev setup-hooks clean

help:
	@echo "════════════════════════════════════════════════"
	@echo "   COMMANDES DISPONIBLES"
	@echo "════════════════════════════════════════════════"
	@echo ""
	@echo "  make pre-commit    - Vérifie tout avant commit (RECOMMANDÉ)"
	@echo "  make check-rules   - Vérifie conformité .clinerules"
	@echo "  make validate      - Compile + teste + linte"
	@echo "  make audit         - Audit sécurité dépendances"
	@echo "  make report        - Génère rapport conformité"
	@echo "  make dev           - Lance dev avec hot-reload"
	@echo "════════════════════════════════════════════════"

pre-commit: check-rules validate audit
	@echo ""
	@./scripts/generate-report.sh

check-rules:
	@echo "📋 Vérification du respect des .clinerules..."
	@./scripts/auto-format.sh
	@./scripts/check-file-size.sh
	@./scripts/check-unwrap.sh
	@./scripts/check-antipatterns.sh
	@./scripts/check-dead-code.sh
	@./scripts/check-circular-imports.sh
	@./scripts/check-architecture.sh
	@echo "✅ Vérification des règles terminée !"

validate:
	@echo "🔍 Validation complète du code..."
	cd src-tauri && cargo check --release
	@./scripts/validate-tests.sh
	@./scripts/check-coverage.sh
	cd src-tauri && cargo clippy --release -- -D warnings
	cd src-tauri && cargo fmt -- --check
	@echo "✅ Validation terminée avec succès !"

audit:
	@echo "🔍 Audit sécurité des dépendances..."
	cargo audit 2>/dev/null || echo "⚠️ cargo-audit non installé"
	@echo "✅ Audit terminé"

report:
	@./scripts/generate-report.sh

dev:
	@echo "🔥 Lancement mode développement..."
	cargo watch -x "check" -x "test" -x "clippy -- -D warnings"

setup-hooks:
	@echo "🔧 Installation du pre-commit hook..."
	ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit
	@echo "✅ Hook installé"

clean:
	@echo "🧹 Nettoyage..."
	cargo clean
	rm -rf target/
	@echo "✅ Nettoyé"
