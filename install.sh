#!/bin/bash

# ESConnect Installer
# Version: 2.0.0

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info() { echo -e "${BLUE}  $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }
print_warn() { echo -e "${YELLOW}! $1${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/bin/esconnect"
INSTALL_DIR="/usr/local/bin"

check_binary() {
    if [ ! -f "$BINARY" ]; then
        print_error "Binary not found at $BINARY"
        exit 1
    fi
}

install_binary() {
    print_info "Installing binary to $INSTALL_DIR..."

    # Remove quarantine (set by macOS when downloaded from the internet)
    xattr -dr com.apple.quarantine "$BINARY" 2>/dev/null || true

    if [ ! -w "$INSTALL_DIR" ]; then
        print_info "Requires sudo for $INSTALL_DIR:"
        sudo cp "$BINARY" "$INSTALL_DIR/esconnect"
        sudo codesign --force --deep --sign - "$INSTALL_DIR/esconnect"
    else
        cp "$BINARY" "$INSTALL_DIR/esconnect"
        codesign --force --deep --sign - "$INSTALL_DIR/esconnect"
    fi

    print_success "Installed to $INSTALL_DIR/esconnect"
}

print_permissions_reminder() {
    echo ""
    echo -e "${YELLOW}Необходимо выдать разрешения в System Settings → Privacy & Security:${NC}"
    echo ""
    echo "  1. Accessibility         → добавить esconnect"
    echo "     (нужно для управления интерфейсом VPN через osascript)"
    echo ""
    echo "  2. Input Monitoring      → добавить esconnect"
    echo "     (нужно для ввода текста в поля пароля)"
    echo ""
    echo "  Без этих разрешений автоматизация работать не будет."
    echo ""
}

main() {
    echo "================================="
    echo "   ESConnect Installer v2.0.0    "
    echo "================================="
    echo ""

    check_binary
    install_binary

    echo ""
    print_success "Бинарник установлен!"
    print_info "Запуск начальной настройки..."
    echo ""

    esconnect setup

    echo ""
    print_info "Запуск демона..."
    esconnect start

    print_permissions_reminder

    echo ""
    print_success "Готово! Проверить статус: esconnect status"
    echo "Логи: tail -f /tmp/esconnect.log"
}

main
