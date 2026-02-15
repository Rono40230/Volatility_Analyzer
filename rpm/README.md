# Paquet RPM - Analyses-Historiques

## 📦 Vue d'ensemble

Ce paquet RPM installe **Analyses-Historiques** sur Fedora/RHEL sans inclure les données volumineuses (10 GB).

- **Taille du paquet**: 9.1 KB (framework minimaliste)
- **Données**: Installées séparément dans `/var/lib/analyses-historiques/`
- **Version**: 0.1.0
- **Build date**: 12 février 2026
- **Compatibilité**: Fedora 43+

## 🚀 Installation

### ✅ RPM Disponible (Fedora 43)

Le paquet RPM est **prêt à l'emploi** à:
```
~/rpmbuild/RPMS/x86_64/analyses-historiques-0.1.0-1.fedora43.fc43.x86_64.rpm
```

### Installation Simple

```bash
# Installation directe
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-0.1.0-1.fedora43.fc43.x86_64.rpm

# OU utiliser dnf
sudo dnf install ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm

# Vérifier l'installation
rpm -ql analyses-historiques
```

### Reconstruction du Paquet (si nécessaire)

```bash
# Prérequis pour Fedora 43+
sudo dnf install -y rpm-build cargo rustc npm nodejs openssl-devel \
                    gtk3-devel webkit2gtk4.1-devel libsoup-devel

# Construire
cd rpm
bash build-rpm.sh
# OU
make rpm
```

## 📁 Structure Installée

```
✅ /usr/bin/analyses-historiques          Binaire principal
✅ /usr/bin/analyses-historiques-gui      Wrapper GUI  
✅ /usr/share/analyses-historiques/       Frontend assets
✅ /usr/share/analyses-historiques/index.html
✅ /var/lib/analyses-historiques/         Répertoire données
   ├─ /data/                              CSV files
   ├─ /db/                                SQLite database
   └─ /imports/                           Import history
```

## 🎯 Utilisation

### Commandes Disponibles

```bash
# Afficher les infos
analyses-historiques

# Lancer l'application
analyses-historiques-gui

# Copier des données
sudo cp *.csv /var/lib/analyses-historiques/data/
```

### Gestion du Paquet

```bash
# Vérifier l'installation
rpm -qa | grep analyses-historiques

# Lister tous les fichiers installés
rpm -ql analyses-historiques

# Vérifier l'espace utilisé
du -sh /var/lib/analyses-historiques/

# Mettre à jour
sudo rpm -Uvh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm

# Désinstaller (données conservées)
sudo rpm -e analyses-historiques
```

```
/
├── usr/bin/
│   ├── analyses-historiques          Binaire principal
│   └── analyses-historiques-gui      Wrapper GUI
├── usr/share/analyses-historiques/
│   └── index.html                    Assets frontend
└── var/lib/analyses-historiques/     Données persistantes (vide au départ)
    ├── data/                         Fichiers CSV à ajouter
    ├── db/                           Base de données SQLite
    └── imports/                      Historique des imports
```

## 📥 Importer des Données

Après installation, copier vos fichiers CSV:

```bash
# Copier vos fichiers
sudo cp /chemin/vers/fichiers/*.csv /var/lib/analyses-historiques/data/

# Vérifier
ls -la /var/lib/analyses-historiques/data/

# Lancer l'app pour importer
analyses-historiques-gui
```

Ou utilisez le menu "Import Manuel" de l'application.
ls -la /var/lib/analyses-historiques/

# Consulter les logs
tail -f /var/lib/analyses-historiques/app.log
```


## 🔄 Mise à jour

```bash
# Mettre à jour vers une nouvelle version
sudo rpm -Uvh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm

# Les données dans /var/lib/analyses-historiques/ sont conservées
```

## ❌ Désinstallation

```bash
# Désinstaller l'app (données conservées)
sudo rpm -e analyses-historiques

# Supprimer aussi les données (attention!)
sudo rm -rf /var/lib/analyses-historiques/
```

## 🐛 Dépannage

### La commande n'est pas reconnue
```bash
# Vérifier l'installation
rpm -qa | grep analyses-historiques

# Réinstaller si nécessaire
sudo rpm -e analyses-historiques
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm
```

### Les données ne sont pas trouvées
```bash
# Vérifier le répertoire
ls -la /var/lib/analyses-historiques/data/

# Copier les fichiers CSV
sudo cp *.csv /var/lib/analyses-historiques/data/
```

## 📋 Dépendances

Le paquet RPM nécessite:
- `gtk3 >= 3.24` (Interface)
- `webkit2gtk4.1` (Rendu web)
- `libxcb` (Protocole X11)

Toutes les dépendances sont gérées par RPM.

## 📝 Version

**Version:** 0.1.0  
**Build:** 12 février 2026  
**Fedora:** 43+  
**Type:** Framework minimaliste (9.1 KB)
