#!/bin/bash
# Apicentric Management Wizard
# Provides an interactive menu for common project tasks

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Clear screen for a clean start
clear

show_logo() {
    echo -e "${CYAN}"
    echo "  ___  ______ _____ _____ _____ _   _ _____ ______ _____ _____ "
    echo "  / _ \ | ___ \_   _/  __ \  ___| \ | |_   _|| ___ \_   _/  __ \ "
    echo "  / /_\ \| |_/ / | | | /  \/ |__ |  \| | | |  | |_/ / | | | /  \/"
    echo "  |  _  ||  __/  | | | |   |  __|| . \` | | |  |    /  | | | |    "
    echo "  | | | || |    _| |_| \__/\ |___| |\  | | |  | |\ \ _| |_| \__/"
    echo "  \_| |_/\_|    \___/ \____\____/\_| \_/ \_/  \_| \_|\___/ \____/"
    echo -e "${NC}"
    echo -e "  ${YELLOW}🧙 Project Management Wizard${NC}"
    echo "  --------------------------------------------------"
}

show_menu() {
    echo -e "\n  ${YELLOW}Please select an option:${NC}"
    echo -e "  1.  🚀 ${GREEN}Build All${NC} (Backend + Frontend)"
    echo -e "  2.  🖥️  ${GREEN}Launch TUI${NC} (Terminal User Interface)"
    echo -e "  3.  🏃 ${GREEN}Start Simulator${NC} (Background)"
    echo -e "  4.  🩺 ${GREEN}Run Doctor${NC} (Diagnostics)"
    echo -e "  5.  🎬 ${GREEN}Run Demo Suite${NC}"
    echo -e "  6.  🧪 ${GREEN}Run Health Check${NC} (Tests + Lint)"
    echo -e "  7.  🧹 ${GREEN}Clean Workspace${NC}"
    echo -e "  8.  🚪 ${RED}Exit${NC}"
    echo ""
}

while true; do
    show_logo
    show_menu
    read -p "  Enter selection [1-8]: " choice

    case $choice in
        1)
            echo -e "\n🏗️  Building..."
            make build
            read -p "Press enter to continue..."
            ;;
        2)
            echo -e "\n🖥️  Launching TUI..."
            make tui
            ;;
        3)
            echo -e "\n🏃 Starting Simulator..."
            make run
            read -p "Press enter to continue..."
            ;;
        4)
            echo -e "\n🩺 Running Doctor..."
            make doctor
            read -p "Press enter to continue..."
            ;;
        5)
            echo -e "\n🎬 Running Demo..."
            make demo
            read -p "Press enter to continue..."
            ;;
        6)
            echo -e "\n🏥 Running Health Check..."
            ./scripts/health_check.sh
            read -p "Press enter to continue..."
            ;;
        7)
            echo -e "\n🧹 Cleaning..."
            make clean
            read -p "Press enter to continue..."
            ;;
        8)
            echo -e "\n👋 Goodbye!"
            exit 0
            ;;
        *)
            echo -e "\n❌ Invalid option"
            sleep 1
            ;;
    esac
    clear
done
