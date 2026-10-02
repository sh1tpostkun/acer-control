#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo ./uninstall.sh)"
  exit 1
fi

echo "Uninstalling AcerControl..."

systemctl stop acercontrol.service || true
systemctl disable acercontrol.service || true

rm -f /etc/systemd/system/acercontrol.service
systemctl daemon-reload

rm -f /usr/local/bin/acercontrol-daemon
rm -f /usr/local/bin/acercontrol
rm -f /usr/local/bin/acercontrol-gui
rm -f /usr/local/bin/acercontrol-toggle

rm -f /usr/share/applications/acercontrol.desktop
rm -f /usr/share/pixmaps/acercontrol.png

echo "Uninstallation complete."
