#!/bin/bash
# This script creates all the debian packaging files
# Run this once before building the package

set -e

echo "Creating debian/ directory structure..."

# Create debian directory
mkdir -p debian

# Create debian/control
cat > debian/control << 'EOF'
Source: tuxedo-control-center
Section: utils
Priority: optional
Maintainer: TUXEDO Control Center Team <dev@example.com>
Build-Depends: debhelper-compat (= 13),
               cargo,
               rustc,
               libgtk-4-dev,
               libadwaita-1-dev,
               pkg-config
Standards-Version: 4.6.2
Homepage: https://github.com/tuxedo/control-center

Package: tuxedo-control-center
Architecture: amd64
Depends: ${shlibs:Depends}, ${misc:Depends},
         libgtk-4-1,
         libadwaita-1-0,
         dbus,
         systemd,
         policykit-1
Recommends: tuxedo-drivers
Description: Hardware control application for TUXEDO/Clevo laptops
 TUXEDO Control Center provides a modern interface for controlling
 hardware features on TUXEDO and Clevo laptops including:
  - CPU frequency and power management
  - GPU control
  - Fan curves
  - Keyboard backlight
  - Profile management
 .
 This package includes both the GUI application and the system daemon.
EOF

# Create debian/changelog
cat > debian/changelog << 'EOF'
tuxedo-control-center (0.1.0-1) noble; urgency=medium

  * Initial release
  * Direct sysfs hardware access
  * GTK 4.14 + Libadwaita 1.5 interface
  * DBus daemon architecture
  * CPU frequency and governor control
  * AMD pstate status management
  * Fan curve configuration
  * Profile management system
  * Auto-switching profiles based on applications

 -- TUXEDO Control Center Team <dev@example.com>  Sat, 14 Dec 2024 00:00:00 +0000
EOF

# Create debian/compat
echo "13" > debian/compat

# Create debian/rules
cat > debian/rules << 'EOF'
#!/usr/bin/make -f

%:
	dh $@

override_dh_auto_build:
	cargo build --release --all

override_dh_auto_install:
	# Install daemon
	install -D -m 755 target/release/tuxedo-daemon debian/tuxedo-control-center/usr/bin/tuxedo-daemon
	
	# Install GUI
	install -D -m 755 target/release/tuxedo-control-center debian/tuxedo-control-center/usr/bin/tuxedo-control-center
	
	# Install systemd service
	install -D -m 644 debian/tuxedo-daemon.service debian/tuxedo-control-center/lib/systemd/system/tuxedo-daemon.service
	
	# Install DBus service file
	install -D -m 644 debian/com.tuxedo.Control.service debian/tuxedo-control-center/usr/share/dbus-1/system-services/com.tuxedo.Control.service
	
	# Install DBus policy
	install -D -m 644 debian/com.tuxedo.Control.conf debian/tuxedo-control-center/usr/share/dbus-1/system.d/com.tuxedo.Control.conf
	
	# Install desktop file
	install -D -m 644 debian/tuxedo-control-center.desktop debian/tuxedo-control-center/usr/share/applications/com.tuxedo.ControlCenter.desktop
	
	# Install icon
	install -D -m 644 debian/icon.svg debian/tuxedo-control-center/usr/share/icons/hicolor/scalable/apps/tuxedo-control-center.svg

override_dh_auto_clean:
	cargo clean || true

override_dh_auto_test:
	# Skip tests for now
EOF
chmod +x debian/rules

# Create debian/tuxedo-daemon.service
cat > debian/tuxedo-daemon.service << 'EOF'
[Unit]
Description=TUXEDO Hardware Control Daemon
After=multi-user.target
Documentation=man:tuxedo-daemon(8)

[Service]
Type=dbus
BusName=com.tuxedo.Control
ExecStart=/usr/bin/tuxedo-daemon
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/sys/devices/platform/tuxedo_io /sys/devices/system/cpu /sys/class/backlight /sys/class/leds
ProtectKernelTunables=false
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
LockPersonality=true
MemoryDenyWriteExecute=true
RestrictAddressFamilies=AF_UNIX AF_NETLINK

[Install]
WantedBy=multi-user.target
EOF

# Create debian/com.tuxedo.Control.service
cat > debian/com.tuxedo.Control.service << 'EOF'
[D-BUS Service]
Name=com.tuxedo.Control
Exec=/usr/bin/tuxedo-daemon
User=root
SystemdService=tuxedo-daemon.service
EOF

# Create debian/com.tuxedo.Control.conf
cat > debian/com.tuxedo.Control.conf << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="com.tuxedo.Control"/>
    <allow send_destination="com.tuxedo.Control"/>
    <allow send_interface="com.tuxedo.Control"/>
  </policy>
  
  <policy context="default">
    <allow send_destination="com.tuxedo.Control"/>
    <allow send_interface="com.tuxedo.Control"/>
    <allow receive_sender="com.tuxedo.Control"/>
  </policy>
</busconfig>
EOF

# Create debian/tuxedo-control-center.desktop
cat > debian/tuxedo-control-center.desktop << 'EOF'
[Desktop Entry]
Name=TUXEDO Control Center
GenericName=Hardware Control
Comment=Control hardware features on TUXEDO/Clevo laptops
Exec=tuxedo-control-center
Icon=tuxedo-control-center
Terminal=false
Type=Application
Categories=System;Settings;HardwareSettings;
Keywords=tuxedo;clevo;hardware;cpu;gpu;fan;power;performance;
StartupNotify=true
EOF

# Create debian/postinst
cat > debian/postinst << 'EOF'
#!/bin/bash
set -e

case "$1" in
    configure)
        # Reload DBus configuration
        if [ -d /run/systemd/system ]; then
            systemctl daemon-reload || true
            dbus-send --system --type=method_call --dest=org.freedesktop.DBus / org.freedesktop.DBus.ReloadConfig || true
        fi
        
        # Enable and start the daemon
        if [ -d /run/systemd/system ]; then
            systemctl enable tuxedo-daemon.service || true
            systemctl start tuxedo-daemon.service || true
        fi
        
        # Update icon cache
        if [ -x /usr/bin/gtk-update-icon-cache ]; then
            gtk-update-icon-cache -q -f /usr/share/icons/hicolor || true
        fi
        
        # Update desktop database
        if [ -x /usr/bin/update-desktop-database ]; then
            update-desktop-database -q /usr/share/applications || true
        fi
        ;;
esac

#DEBHELPER#

exit 0
EOF
chmod +x debian/postinst

# Create debian/prerm
cat > debian/prerm << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove|deconfigure)
        # Stop the daemon
        if [ -d /run/systemd/system ]; then
            systemctl stop tuxedo-daemon.service || true
            systemctl disable tuxedo-daemon.service || true
        fi
        ;;
esac

#DEBHELPER#

exit 0
EOF
chmod +x debian/prerm

# Create debian/postrm
cat > debian/postrm << 'EOF'
#!/bin/bash
set -e

case "$1" in
    purge)
        # Remove configuration files
        rm -rf /etc/tuxedo-control-center || true
        
        # Reload DBus
        if [ -d /run/systemd/system ]; then
            systemctl daemon-reload || true
            dbus-send --system --type=method_call --dest=org.freedesktop.DBus / org.freedesktop.DBus.ReloadConfig || true
        fi
        ;;
    
    remove)
        # Update icon cache
        if [ -x /usr/bin/gtk-update-icon-cache ]; then
            gtk-update-icon-cache -q -f /usr/share/icons/hicolor || true
        fi
        
        # Update desktop database
        if [ -x /usr/bin/update-desktop-database ]; then
            update-desktop-database -q /usr/share/applications || true
        fi
        ;;
esac

#DEBHELPER#

exit 0
EOF
chmod +x debian/postrm

# Create debian/copyright
cat > debian/copyright << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: tuxedo-control-center
Upstream-Contact: TUXEDO Control Center Team <dev@example.com>
Source: https://github.com/tuxedo/control-center

Files: *
Copyright: 2024 TUXEDO Control Center Team
License: GPL-2.0

License: GPL-2.0
 This package is free software; you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation; version 2 of the License.
 .
 This package is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.
 .
 You should have received a copy of the GNU General Public License
 along with this program. If not, see <https://www.gnu.org/licenses/>
 .
 On Debian systems, the complete text of the GNU General
 Public License version 2 can be found in "/usr/share/common-licenses/GPL-2".
EOF

# Create debian/icon.svg
cat > debian/icon.svg << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg width="64" height="64" version="1.1" xmlns="http://www.w3.org/2000/svg">
  <rect width="64" height="64" rx="8" fill="#0066cc"/>
  <path d="m16 20h32v4h-32zm0 10h32v4h-32zm0 10h32v4h-32z" fill="#ffffff"/>
  <circle cx="48" cy="48" r="8" fill="#00cc66"/>
</svg>
EOF

echo "✅ Debian packaging files created successfully!"
echo ""
echo "Files created in debian/:"
ls -la debian/
echo ""
echo "Next step: Run ./build-deb.sh to build the package"