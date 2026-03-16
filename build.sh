#!/bin/bash

# Собирает universal binary (arm64 + x86_64) и кладёт в bin/

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_info() { echo -e "${BLUE}  $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

print_info "Добавление targets..."
rustup target add x86_64-apple-darwin aarch64-apple-darwin 2>/dev/null

print_info "Сборка x86_64..."
cargo build --release --target x86_64-apple-darwin

print_info "Сборка aarch64..."
cargo build --release --target aarch64-apple-darwin

mkdir -p bin
print_info "Склейка universal binary..."
lipo -create -output bin/esconnect \
    target/x86_64-apple-darwin/release/esconnect \
    target/aarch64-apple-darwin/release/esconnect

print_success "Готово: bin/esconnect ($(lipo -info bin/esconnect | grep -o 'x86_64 arm64'))"
ls -lh bin/esconnect
