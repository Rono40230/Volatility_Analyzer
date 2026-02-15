# 📦 Paquet RPM Analyses-Historiques - Guide Complet

## ✅ Status: RPM Créé et Prêt

**Fichier RPM généré:**
```
~/rpmbuild/RPMS/x86_64/analyses-historiques-0.1.0-1.fedora43.fc43.x86_64.rpm
Taille: 9.1 KB
```

## 🚀 Installation

### Option 1: Installation directe
```bash
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-0.1.0-1.fedora43.fc43.x86_64.rpm
```

### Option 2: Installation avec dnf
```bash
sudo dnf install ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm
```

### Option 3: Réinstallation (si nécessaire)
```bash
# Supprimer l'ancienne version
sudo rpm -e analyses-historiques

# Installer la nouvelle
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm
```

## 📂 Structure de Répertoires après Installation

| Chemin | Utilisation |
|--------|-------------|
| `/usr/bin/analyses-historiques` | Binaire principal |
| `/usr/bin/analyses-historiques-gui` | Wrapper GUI |
| `/usr/share/analyses-historiques/` | Assets frontend |
| `/var/lib/analyses-historiques/` | Données persistantes |
| `/var/lib/analyses-historiques/data/` | CSV files |
| `/var/lib/analyses-historiques/db/` | Base SQLite |
| `/var/lib/analyses-historiques/imports/` | Historique imports |

## 🛠️ Utilisation

### Afficher l'info
```bash
analyses-historiques
```

### Lancer l'application
```bash
analyses-historiques-gui
```

## 📊 Importer des Données

```bash
# 1. Placer vos fichiers CSV
cp *.csv /var/lib/analyses-historiques/data/

# 2. Lancer l'application
analyses-historiques-gui

# 3. Importer via l'interface
```

## 🔄 Version Actuelle

- **Version:** 0.1.0
- **Release:** 1.fedora43
- **Fedora:** 43+
- **Build Type:** Framework (structure installée, binaire à compiler séparément)

## 🔧 Désinstallation

```bash
sudo rpm -e analyses-historiques
```

## 📦 Reconstruction du RPM

Si vous modifiez le code et voulez reconstruire:

```bash
# Option 1: Utiliser le Makefile
cd rpm
make rpm

# Option 2: Script direct
cd rpm
bash build-rpm.sh

# Option 3: rpmbuild direct
rpmbuild -ba rpm/analyses-historiques.spec
```

## 🐛 Dépannage

### Le RPM n'installe pas
```bash
# Vérifier les dépendances
rpm -q gtk3 webkit2gtk4.1 libxcb

# Vérifier le paquet
rpm -qlp ~/rpmbuild/RPMS/x86_64/analyses-historiques-*.rpm
```

### Les répertoires /var/lib ne sont pas créés
```bash
# Vérifier après installation
ls -la /var/lib/analyses-historiques/

# Les créer manuellement
sudo mkdir -p /var/lib/analyses-historiques/{data,db,imports}
sudo chmod 755 /var/lib/analyses-historiques
```

## 📝 Notes

- Ce RPM est un **framework minimaliste** (9.1 KB)
- Idéal pour distribution sans les 10 GB de données
- Vous pouvez ajouter le binaire Rust compilé ultérieurement
- Les données restent dans `/var/lib/analyses-historiques/` après désinstallation

## 🔗 Ressources

- Source: `/home/rono/Analyse historiques/Analyses-historiques-1/`
- RPM Source: `~/rpmbuild/SOURCES/analyses-historiques-0.1.0.tar.gz`
- Spec File: `~/rpmbuild/SPECS/analyses-historiques.spec`

---

**Date de création:** 12 février 2026
**Maintaineur:** Rono
