# Makefile pour construction du paquet RPM
# Usage: make rpm

.PHONY: rpm rpm-clean help

help:
	@echo "📦 Cibles disponibles:"
	@echo "  make rpm        - Construire le paquet RPM"
	@echo "  make rpm-clean  - Nettoyer la compilation RPM"
	@echo "  make help       - Afficher cette aide"

rpm:
	@bash rpm/build-rpm.sh

rpm-clean:
	@echo "🗑️  Nettoyage des fichiers RPM..."
	@rm -rf ~/rpmbuild/BUILD/*
	@rm -rf ~/rpmbuild/BUILDROOT/*
	@rm -rf ~/rpmbuild/RPMS/*
	@rm -rf ~/rpmbuild/SRPMS/*
	@echo "✅ Nettoyage effectué"

rpm-install: rpm
	@echo ""
	@echo "📦 Installation du paquet..."
	@sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm

rpm-reinstall: rpm-clean rpm-install
	@echo ""
	@echo "✅ Réinstallation complète"

.DEFAULT_GOAL := help
