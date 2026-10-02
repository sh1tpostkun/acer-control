#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo ./install.sh)"
  exit 1
fi

echo "Installing AcerControl..."

# Ensure group acercontrol exists
if ! getent group acercontrol >/dev/null; then
    echo "Creating 'acercontrol' system group..."
    groupadd -r acercontrol || true
fi

if [ -n "$SUDO_USER" ]; then
    usermod -aG acercontrol "$SUDO_USER"
    echo "Added user $SUDO_USER to acercontrol group."
fi

# Install binaries
install -m 755 target/release/acercontrol-daemon /usr/local/bin/acercontrol-daemon
install -m 755 target/release/acercontrol-cli /usr/local/bin/acercontrol
install -m 755 build/gui/acercontrol-gui /usr/local/bin/acercontrol-gui
install -m 755 acercontrol-toggle.sh /usr/local/bin/acercontrol-toggle

# Install systemd service
cp systemd/acercontrol.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable acercontrol.service
systemctl restart acercontrol.service

# Install desktop entry
mkdir -p /usr/share/pixmaps
cp gui/res/app_icon.png /usr/share/pixmaps/acercontrol.png
mkdir -p /usr/share/applications
cp acercontrol.desktop /usr/share/applications/

echo "Installation complete."
echo "You can check daemon status with: systemctl status acercontrol.service"
