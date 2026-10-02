#!/bin/bash

# Define colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}==============================${NC}"
echo -e "${BLUE}     AcerControl Setup${NC}"
echo -e "${BLUE}==============================${NC}"
echo "1 -> Install"
echo "2 -> Uninstall"
echo "3 -> Exit"
echo "4 -> Reinstall/Update"
echo -n "Choose an option: "
read choice

case $choice in
  1)
    echo -e "\n${GREEN}Starting Installation...${NC}"
    if [ ! -f "build/gui/acercontrol-gui" ] || [ ! -f "target/release/acercontrol-daemon" ]; then
        echo "Binaries not found. Building first..."
        ./build.sh
    fi
    sudo ./install.sh
    ;;
  2)
    echo -e "\n${GREEN}Starting Uninstallation...${NC}"
    sudo ./uninstall.sh
    ;;
  3)
    echo "Exiting..."
    exit 0
    ;;
  4)
    echo -e "\n${GREEN}Starting Reinstall/Update...${NC}"
    # Optionally git pull here if needed, but since we are local, just build and install
    echo "Rebuilding project..."
    ./build.sh
    sudo ./install.sh
    ;;
  *)
    echo -e "${RED}Invalid option. Exiting.${NC}"
    exit 1
    ;;
esac
