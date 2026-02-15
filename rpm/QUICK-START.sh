#!/bin/bash
# Quick reference pour le paquet RPM

cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║         Analyses-Historiques - Paquet RPM Fedora              ║
║                      Quick Start Guide                        ║
╚═══════════════════════════════════════════════════════════════╝

📦 CONTENU CRÉÉ:
  rpm/
  ├── analyses-historiques.spec     (définition du paquet)
  ├── build-rpm.sh                  (script de construction)
  ├── post-install.sh               (script post-installation)
  ├── config.example.toml           (configuration exemple)
  ├── Makefile                      (automatisation)
  ├── README.md                     (documentation complète)
  └── QUICK-START.sh                (ce fichier)

═══════════════════════════════════════════════════════════════

🚀 DÉMARRAGE RAPIDE:

  # Option 1: Avec make (plus facile)
  $ cd rpm && make rpm

  # Option 2: Script direct
  $ cd rpm && bash build-rpm.sh

═══════════════════════════════════════════════════════════════

⏱️  DURÉE ESTIMÉE:
  - Première build: 10-15 minutes
  - Builds ultérieurs: 3-5 minutes
  - Taille du RPM: ~100 MB

═══════════════════════════════════════════════════════════════

✅ APRÈS LA CONSTRUCTION:

  # 1. Installer le paquet
  $ sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm

  # 2. Setup des données
  $ sudo /etc/analyses-historiques/setup-data.sh

  # 3. Lancer l'app
  $ analyses-historiques-gui

═══════════════════════════════════════════════════════════════

💾 DOSSIERS CRÉÉS:
  /usr/bin/analyses-historiques       (binaire)
  /usr/share/analyses-historiques/    (assets frontend)
  /etc/analyses-historiques/          (configuration)
  /var/lib/analyses-historiques/      (données - VIDE)
    ├── data/                         (CSV à importer)
    ├── db/                           (base SQLite)
    └── imports/                      (historique)

═══════════════════════════════════════════════════════════════

📚 DOCUMENTATION:
  - Détaillé:   cat rpm/README.md
  - Spec RPM:   cat rpm/analyses-historiques.spec
  - Script:     cat rpm/build-rpm.sh

═══════════════════════════════════════════════════════════════

⚙️  PRÉREQUIS POUR BUILD (Fedora 43+):
  $ sudo dnf install -y rpm-build cargo rustc npm nodejs openssl-devel \
                        gtk3-devel webkitgtk6-devel

═══════════════════════════════════════════════════════════════

❓ QUESTIONS FRÉQUENTES:

Q: Puis-je passer des arguments à la construction?
A: Non directement. Modifiez rpm/analyses-historiques.spec

Q: Comment inclure mes données dans le RPM?
A: Modifiez la section %install du .spec pour inclure vos CSV
   (déconseillé pour 10 GB!)

Q: Comment upgrader vers une nouvelle version?
A: sudo rpm -Uvh nouveau-paquet.rpm (données conservées)

Q: Où sont les logs?
A: cat /var/lib/analyses-historiques/app.log

═══════════════════════════════════════════════════════════════

🚨 DÉPANNAGE:

Erreur "rpmbuild not found":
  $ sudo dnf install rpm-build

Erreur "cargo not found":
  $ sudo dnf install cargo rustc

Erreur de compilation Rust:
  $ cd src-tauri && cargo clean && cargo build --release

═══════════════════════════════════════════════════════════════
EOF
