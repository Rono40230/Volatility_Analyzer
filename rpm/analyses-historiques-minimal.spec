Name:           analyses-historiques
Version:        0.1.0
Release:        1.fedora43%{?dist}
Summary:        Analyseur de volatilité Forex - Application native Tauri
License:        Propriétaire
URL:            https://github.com/rono40230/analyses-historiques

# Runtime dependencies for Tauri native app
Requires:       gtk3 >= 3.24
Requires:       webkit2gtk4.1
Requires:       libxcb
Requires:       libsoup3

%description
Analyseur de volatilité pour le trading Forex utilisant Tauri 2.0.
Cette version inclut l'application desktop native compilée.

%prep
# No prep needed

%build
# No build needed

%install
# Create directory structure
install -d %{buildroot}/usr/bin
install -d %{buildroot}/usr/share/pixmaps
install -d %{buildroot}/usr/share/applications
install -d %{buildroot}/var/lib/%{name}/{data,db,imports}
install -d %{buildroot}/etc/%{name}

# Copy the Tauri binary (native app) directly
install -m 755 "/home/rono/Analyse historiques/Analyses-historiques-1/src-tauri/target/release/analyses-historiques-volatility" \
    %{buildroot}/usr/bin/%{name}

# Copy frontend assets (required by Tauri)
install -d %{buildroot}/usr/share/%{name}/dist
cp -r "/home/rono/Analyse historiques/Analyses-historiques-1/dist/"* \
    %{buildroot}/usr/share/%{name}/dist/ 2>/dev/null || true

# Install icon
install -m 644 "/home/rono/Analyse historiques/Analyses-historiques-1/src-tauri/icons/128x128.png" \
    %{buildroot}/usr/share/pixmaps/%{name}.png

# Install .desktop file for application menu
cat > %{buildroot}/usr/share/applications/%{name}.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Analyses-Historiques
Comment=Analyseur de volatilité Forex avec Tauri 2.0
Exec=/usr/bin/analyses-historiques
Icon=analyses-historiques
Categories=Office;Finance;Utility;
Terminal=false
StartupNotify=true
Version=1.0
EOF
chmod 644 %{buildroot}/usr/share/applications/%{name}.desktop

%post
#!/bin/bash
# Create data directories
mkdir -p /var/lib/%{name}/{data,db,imports}
chmod 755 /var/lib/%{name}
chmod 755 /var/lib/%{name}/data
chmod 755 /var/lib/%{name}/db
chmod 755 /var/lib/%{name}/imports

echo ""
echo "=========================================="
echo "✅ Analyses-Historiques Installed"
echo "=========================================="
echo ""
echo "� Application: Analyses-Historiques (Native Tauri App)"
echo "📂 Data directory: /var/lib/analyses-historiques/"
echo "🚀 Launch: analyses-historiques (or from application menu)"
echo ""

%files
%attr(755,root,root) /usr/bin/%{name}
/usr/share/%{name}/
/usr/share/applications/%{name}.desktop
/usr/share/pixmaps/%{name}.png
%dir /var/lib/%{name}
%dir /var/lib/%{name}/data
%dir /var/lib/%{name}/db
%dir /var/lib/%{name}/imports

%changelog
* Sun Feb 12 2026 Rono <rono@example.com> - 0.1.0-1.fedora43
- Native Tauri desktop application package
