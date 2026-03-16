#!/bin/bash

# Полное удаление ESConnect

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info() { echo -e "${BLUE}  $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_warn() { echo -e "${YELLOW}! $1${NC}"; }

echo "========================="
echo "  ESConnect Uninstaller  "
echo "========================="
echo ""

# Остановить демон
if [ -f /tmp/esconnect.pid ]; then
    print_info "Остановка демона..."
    kill "$(cat /tmp/esconnect.pid)" 2>/dev/null && print_success "Демон остановлен" || true
    rm -f /tmp/esconnect.pid
else
    print_warn "Демон не запущен"
fi

# Удалить бинарник
if [ -f /usr/local/bin/esconnect ]; then
    print_info "Удаление бинарника..."
    sudo rm /usr/local/bin/esconnect
    print_success "Удалён /usr/local/bin/esconnect"
else
    print_warn "Бинарник не найден"
fi

# Удалить конфиг
CONFIG_DIR="$HOME/Library/Application Support/com.esconnect.esconnect"
if [ -d "$CONFIG_DIR" ]; then
    print_info "Удаление конфига..."
    rm -rf "$CONFIG_DIR"
    print_success "Удалён конфиг"
else
    print_warn "Конфиг не найден"
fi

# Удалить секреты из Keychain
print_info "Удаление секретов из Keychain..."
security delete-generic-password -s esconnect -a auth_token 2>/dev/null \
    && print_success "auth_token удалён" \
    || print_warn "auth_token не найден в Keychain"
security delete-generic-password -s esconnect -a vpn_password 2>/dev/null \
    && print_success "vpn_password удалён" \
    || print_warn "vpn_password не найден в Keychain"

# Удалить логи
rm -f /tmp/esconnect.log
print_success "Временные файлы очищены"

echo ""
print_success "ESConnect полностью удалён"
