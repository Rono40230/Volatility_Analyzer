Name:           analyses-historiques
Version:        0.1.0
Release:        1.fedora43%{?dist}
Summary:        Analyseur de volatilité Forex avec Tauri 2.0
License:        Propriétaire
URL:            https://github.com/rono40230/analyses-historiques

# Minimal dependencies - runtime only
Requires:       gtk3 >= 3.24
Requires:       webkit2gtk4.1
Requires:       libxcb

%description
Analyseur de volatilité pour le trading Forex.
Version légère sans données volumineuses.

%prep
# No sources needed - using pre-built binary

%build
# No build required - using pre-built binary

%install
# Create directory structure
install -d %{buildroot}/usr/bin
install -d %{buildroot}/usr/share/%{name}
install -d %{buildroot}/var/lib/%{name}/{data,db,imports}
install -d %{buildroot}/etc/%{name}

# Copy binary if it exists
if [ -f "/home/rono/Analyse historiques/Analyses-historiques-1/src-tauri/target/release/analyses-historiques-volatility" ]; then
    install -m 755 "/home/rono/Analyse historiques/Analyses-historiques-1/src-tauri/target/release/analyses-historiques-volatility" \
        "%{buildroot}/usr/bin/%{name}"
    
    # Create wrapper
    cat > %{buildroot}/usr/bin/%{name}-gui << 'EOF'
#!/bin/bash
export APP_DATA_DIR="/var/lib/%{name}"
exec "/usr/bin/%{name}" "$@"
EOF
    chmod 755 %{buildroot}/usr/bin/%{name}-gui
fi

# Copy assets if available
if [ -d "/home/rono/Analyse historiques/Analyses-historiques-1/dist" ]; then
    cp -r "/home/rono/Analyse historiques/Analyses-historiques-1/dist"/* %{buildroot}/usr/share/%{name}/ 2>/dev/null || true
fi

# Install desktop file and icon
install -d %{buildroot}/usr/share/applications
install -m 644 "/home/rono/Analyse historiques/Analyses-historiques-1/rpm/analyses-historiques.desktop" %{buildroot}/usr/share/applications/

install -d %{buildroot}/usr/share/icons/hicolor/1024x1024/apps
install -m 644 "/home/rono/Analyse historiques/Analyses-historiques-1/rpm/analyses-historiques.png" %{buildroot}/usr/share/icons/hicolor/1024x1024/apps/%{name}.png

# Install to pixmaps as fallback (often required for immediate visibility)
install -d %{buildroot}/usr/share/pixmaps
install -m 644 "/home/rono/Analyse historiques/Analyses-historiques-1/rpm/analyses-historiques.png" %{buildroot}/usr/share/pixmaps/%{name}.png

%post
mkdir -p /var/lib/%{name}/{data,db,imports}
chmod 755 /var/lib/%{name}
chmod 755 /var/lib/%{name}/data
chmod 755 /var/lib/%{name}/db
chmod 755 /var/lib/%{name}/imports

%files
%defattr(-,root,root,-)
/usr/bin/%{name}
/usr/bin/%{name}-gui
/usr/share/%{name}/
/usr/share/applications/analyses-historiques.desktop
/usr/share/icons/hicolor/1024x1024/apps/analyses-historiques.png
/usr/share/pixmaps/analyses-historiques.png
%dir /var/lib/%{name}
%dir /var/lib/%{name}/data
%dir /var/lib/%{name}/db
%dir /var/lib/%{name}/imports

%changelog
* Sun Feb 12 2026 Rono <rono@example.com> - 0.1.0-1.fedora43
- Pre-built binary RPM for Fedora 43
