%define name analyses-historiques
%define version 0.1.0
%define release 1.fedora
%define buildroot %{_tmppath}/%{name}-%{version}-%{release}-buildroot

Name: %{name}
Version: %{version}
Release: %{release}
Summary: Analyseur de volatilité Forex pour scalping avec Tauri 2.0
License: Propriétaire
URL: https://github.com/rono40230/analyses-historiques

# Dépendances pour Tauri/Rust sur Fedora 43+
BuildRequires: cargo >= 1.70
BuildRequires: rustc >= 1.70
BuildRequires: npm >= 9.0
BuildRequires: nodejs >= 18.0
BuildRequires: openssl-devel
BuildRequires: gtk3-devel
BuildRequires: webkit2gtk4.1-devel
BuildRequires: libxcb-devel
BuildRequires: at-spi2-core-devel

Requires: openssl-libs >= 3.0
Requires: gtk3 >= 3.24
Requires: webkit2gtk4.1
Requires: libxcb
Requires: at-spi2-core >= 2.0

%description
Analyseur de volatilité pour le trading Forex. Calcule l'ATR, les profils de mouvement,
et les scores de confiance pour optimiser les stratégies de straddle.

Version légère : données séparées du code, installables optionnellement.

%prep
# Les sources sont déjà compilées par build-rpm.sh
# Pas besoin de faire quoi que ce soit ici

%build
# Les compilations frontend et backend sont faites par build-rpm.sh
# Ce script ne fait rien à ce stade

%install
# Point our builddir to our project directory
SOURCE_DIR=%{_builddir}

# Créer la structure de répertoires
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/libexec/%{name}
mkdir -p %{buildroot}/usr/share/%{name}
mkdir -p %{buildroot}/var/lib/%{name}/data
mkdir -p %{buildroot}/var/lib/%{name}/db
mkdir -p %{buildroot}/var/lib/%{name}/imports
mkdir -p %{buildroot}/etc/%{name}
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/32x32/apps
mkdir -p %{buildroot}/usr/share/icons/hicolor/64x64/apps
mkdir -p %{buildroot}/usr/share/icons/hicolor/128x128/apps

# Copier le binaire compilé
# Le binaire est dans %{_sourcedir} 
install -m 755 %{_sourcedir}/analyses-historiques-volatility %{buildroot}/usr/bin/%{name} || {
  echo "❌ ERREUR: Binaire non trouvé dans %{_sourcedir}"
  exit 1
}

# *IMPORTANT*: Assurer la structure que Tauri cherche
# Tauri avec frontendDist: "../dist" cherche: <binaire>/../dist
# Position du binaire: /usr/libexec/analyses-historiques/analyses-historiques
# Cela cherche: /usr/libexec/dist -> ne fonctionne pas
# Solution: créer un symlink dist dans libexec qui pointe vers /usr/share/analyses-historiques/dist

# Copier les assets statiques (dist/) à /usr/share/analyses-historiques/
# %{_sourcedir} = projet root, dist/ doit y être compilé par build-rpm-simple.sh
if [ ! -d "%{_sourcedir}/dist" ]; then
  echo "❌ ERREUR: dist/ non trouvé dans %{_sourcedir}"
  ls -la %{_sourcedir}/
  exit 1
fi
cp -r "%{_sourcedir}/dist" %{buildroot}/usr/share/%{name}/

# Créer un wrapper qui place le binaire dans le bon contexte de répertoires
mkdir -p %{buildroot}/usr/libexec/%{name}
mv %{buildroot}/usr/bin/%{name} %{buildroot}/usr/libexec/%{name}/

# Créer un symlink dist directement dans /usr/libexec/ 
# Le binaire dans /usr/libexec/analyses-historiques/ cherche ../dist
# Donc on crée /usr/libexec/dist pointant vers /usr/share/analyses-historiques/dist
ln -sf ../share/%{name}/dist %{buildroot}/usr/libexec/dist

# Copier le fichier .desktop
if [ -f "$BUILDDIR/rpm/analyses-historiques.desktop" ]; then
  install -m 644 "$BUILDDIR/rpm/analyses-historiques.desktop" %{buildroot}/usr/share/applications/
elif [ -f "%{_sourcedir}/rpm/analyses-historiques.desktop" ]; then
  install -m 644 "%{_sourcedir}/rpm/analyses-historiques.desktop" %{buildroot}/usr/share/applications/
fi

# Copier les icônes depuis src-tauri/icons
for SIZE in 32x32 64x64 128x128; do
  ICON_PATH=""
  if [ -f "$BUILDDIR/src-tauri/icons/${SIZE}.png" ]; then
    ICON_PATH="$BUILDDIR/src-tauri/icons/${SIZE}.png"
  elif [ -f "%{_sourcedir}/src-tauri/icons/${SIZE}.png" ]; then
    ICON_PATH="%{_sourcedir}/src-tauri/icons/${SIZE}.png"
  fi
  
  if [ -n "$ICON_PATH" ] && [ -f "$ICON_PATH" ]; then
    install -m 644 "$ICON_PATH" %{buildroot}/usr/share/icons/hicolor/${SIZE}/apps/analyses-historiques.png
  fi
done

# Créer un script wrapper pour l'app avec variables d'environnement nécessaires et exécution en contexte
cat > %{buildroot}/usr/bin/%{name}-gui << 'WRAPPER_EOF'
#!/bin/bash
# Wrapper de lancement pour Analyses-Historiques
# Exécute le binaire depuis son répertoire libexec où ../dist existe
LOG_FILE="${HOME}/.local/share/analyses-historiques-launcher.log"

# Configurer les répertoires de données
export APP_DATA_DIR="/var/lib/%{name}"

# Assurer que les répertoires de données existent
mkdir -p "$APP_DATA_DIR/data" 2>/dev/null || true
mkdir -p "$APP_DATA_DIR/db" 2>/dev/null || true
mkdir -p "$APP_DATA_DIR/imports" 2>/dev/null || true

# Variables d'environnement pour Tauri sur Linux
export GTK_IM_MODULE=fcitx
export RUST_LOG=info

# Variables Wayland (pour Fedora 41+)
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Log
mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null
echo "[$(date)] Lancement de %{name} avec APP_DATA_DIR=$APP_DATA_DIR" >> "$LOG_FILE" 2>&1

# Exécuter le binaire depuis son répertoire libexec
exec /usr/libexec/%{name}/%{name} "$@" 2>> "$LOG_FILE"
WRAPPER_EOF
chmod 755 %{buildroot}/usr/bin/%{name}-gui

# Copier les scripts de setup
if [ -f "$BUILDDIR/rpm/post-install.sh" ]; then
  install -m 755 "$BUILDDIR/rpm/post-install.sh" %{buildroot}/etc/%{name}/setup-data.sh
elif [ -f "%{_sourcedir}/rpm/post-install.sh" ]; then
  install -m 755 "%{_sourcedir}/rpm/post-install.sh" %{buildroot}/etc/%{name}/setup-data.sh
fi

# Copier le fichier de configuration d'exemple
if [ -f "$BUILDDIR/rpm/config.example.toml" ]; then
  install -m 644 "$BUILDDIR/rpm/config.example.toml" %{buildroot}/etc/%{name}/config.toml.example
elif [ -f "%{_sourcedir}/rpm/config.example.toml" ]; then
  install -m 644 "%{_sourcedir}/rpm/config.example.toml" %{buildroot}/etc/%{name}/config.toml.example
fi

%post
# Mise à jour du cache des icônes
update-desktop-database -q /usr/share/applications/ 2>/dev/null || true
gtk-update-icon-cache /usr/share/icons/hicolor/ 2>/dev/null || true

# Création des répertoires de données et permissions
mkdir -p /var/lib/%{name}/data
mkdir -p /var/lib/%{name}/db
mkdir -p /var/lib/%{name}/imports
chmod 755 /var/lib/%{name}
chmod 755 /var/lib/%{name}/data
chmod 755 /var/lib/%{name}/db
chmod 755 /var/lib/%{name}/imports

echo "=========================================="
echo "Installation d'Analyses-Historiques"
echo "=========================================="
echo ""
echo "✅ Application installée dans : /usr/bin/%{name}"
echo "📂 Données seront stockées dans : /var/lib/%{name}"
echo ""
echo "⚠️  PROCHAINES ÉTAPES:"
echo "1. Placez vos fichiers CSV dans : /var/lib/%{name}/data/"
echo "   OU lancez : /etc/%{name}/setup-data.sh"
echo ""
echo "2. Lancez l'application :"
echo "   $ %{name}-gui"
echo ""
echo "3. Documentation complète :"
echo "   https://github.com/rono40230/analyses-historiques"
echo ""

%preun
# Cleanup avant désinstallation
echo "Désinstallation d'Analyses-Historiques..."
echo "⚠️  Les données dans /var/lib/%{name} seront conservées"

%postun
# Nettoyage du cache des icônes et applications
update-desktop-database -q /usr/share/applications/ 2>/dev/null || true
gtk-update-icon-cache /usr/share/icons/hicolor/ 2>/dev/null || true

# Actions après désinstallation
echo "Application supprimée. Les données restent dans /var/lib/%{name}"

%files
%defattr(-,root,root)
/usr/libexec/%{name}/%{name}
/usr/libexec/dist
/usr/bin/%{name}-gui
/usr/share/%{name}/
/usr/share/applications/analyses-historiques.desktop
/usr/share/icons/hicolor/32x32/apps/analyses-historiques.png
/usr/share/icons/hicolor/64x64/apps/analyses-historiques.png
/usr/share/icons/hicolor/128x128/apps/analyses-historiques.png
%config(noreplace) /etc/%{name}/config.toml.example
/etc/%{name}/setup-data.sh
%dir %attr(755,root,root) /var/lib/%{name}
%dir %attr(755,root,root) /var/lib/%{name}/data
%dir %attr(755,root,root) /var/lib/%{name}/db
%dir %attr(755,root,root) /var/lib/%{name}/imports

%changelog
* Wed Feb 12 2026 Rono <rono@example.com> - 0.1.0-1.fedora
- Version initiale du paquet RPM
- Support complet Tauri 2.0 + Vue 3
- Données persistantes séparées du code

%clean
rm -rf %{buildroot}
