Name:           analyses-historiques
Version:        0.1.0
Release:        1.fedora43%{?dist}
Summary:        Analyseur de volatilité Forex pour scalping avec Tauri 2.0
License:        Propriétaire
URL:            https://github.com/rono40230/analyses-historiques
Source0:        analyses-historiques-0.1.0.tar.gz

# Build dependencies (Fedora 43+)
BuildRequires:  cargo >= 1.70
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  rustc >= 1.70
BuildRequires:  npm >= 9.0
BuildRequires:  nodejs >= 18.0
BuildRequires:  openssl-devel
BuildRequires:  gtk3-devel
BuildRequires:  webkit2gtk4.1-devel
BuildRequires:  libxcb-devel
BuildRequires:  at-spi2-core-devel
BuildRequires:  glib2-devel
BuildRequires:  libsoup-devel

# Runtime dependencies
Requires:       gtk3 >= 3.24
Requires:       webkit2gtk4.1
Requires:       libxcb
Requires:       at-spi2-core-libs
Requires:       glib2
Requires:       libsoup

%description
Analyseur de volatilité pour le trading Forex. Calcule l'ATR, les profils de mouvement,
et les scores de confiance pour optimiser les stratégies de straddle.

Version légère : données séparées du code, installables optionnellement.

%prep
%setup -q

%build
# Compiler le frontend
npm install
npm ci --prefer-offline || npm install
npm run build

# Compiler Rust/Tauri
cd src-tauri
cargo build --release
cd ..

%install
# Create directory structure
install -d %{buildroot}/usr/bin
install -d %{buildroot}/usr/share/%{name}
install -d %{buildroot}/var/lib/%{name}/data
install -d %{buildroot}/var/lib/%{name}/db
install -d %{buildroot}/var/lib/%{name}/imports
install -d %{buildroot}/etc/%{name}

# Install binary
if [ -f "src-tauri/target/release/analyses-historiques-volatility" ]; then
    install -m 755 "src-tauri/target/release/analyses-historiques-volatility" \
        "%{buildroot}/usr/bin/%{name}"
fi

# Install frontend assets
if [ -d "dist" ]; then
    cp -r dist/* %{buildroot}/usr/share/%{name}/
fi

# Install wrapper script
cat > %{buildroot}/usr/bin/%{name}-gui << 'EOF'
#!/bin/bash
export APP_DATA_DIR="/var/lib/%{name}"
exec "/usr/bin/%{name}" "$@"
EOF
chmod 755 %{buildroot}/usr/bin/%{name}-gui

# Install setup script
if [ -f "rpm/post-install.sh" ]; then
    install -m 755 "rpm/post-install.sh" \
        "%{buildroot}/etc/%{name}/setup-data.sh"
fi

# Install example config
if [ -f "rpm/config.example.toml" ]; then
    install -m 644 "rpm/config.example.toml" \
        "%{buildroot}/etc/%{name}/config.toml.example"
fi

%post
# Create data directories with proper permissions
mkdir -p /var/lib/%{name}/data
mkdir -p /var/lib/%{name}/db
mkdir -p /var/lib/%{name}/imports
chmod 755 /var/lib/%{name}
chmod 755 /var/lib/%{name}/data
chmod 755 /var/lib/%{name}/db
chmod 755 /var/lib/%{name}/imports

%files
%defattr(-,root,root,-)
/usr/bin/%{name}
/usr/bin/%{name}-gui
/usr/share/%{name}/
%config(noreplace) /etc/%{name}/config.toml.example
/etc/%{name}/setup-data.sh
%dir /var/lib/%{name}
%dir /var/lib/%{name}/data
%dir /var/lib/%{name}/db
%dir /var/lib/%{name}/imports

%preun
echo "Désinstallation d'Analyses-Historiques..."
echo "⚠️  Les données dans /var/lib/%{name} seront conservées"

%postun
echo "Application supprimée. Les données restent dans /var/lib/%{name}"

%changelog
* Sun Feb 12 2026 Rono <rono@example.com> - 0.1.0-1.fedora43
- Version initiale du paquet RPM
- Support complet Tauri 2.0 + Vue 3  
- Données persistantes séparées du code
