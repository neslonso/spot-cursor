#!/bin/bash
set -e

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== FindMyCursor Build ===${NC}"

# Verificar requisitos
check_requirements() {
    echo -e "\n${YELLOW}Verificando requisitos...${NC}"
    
    if ! command -v rustup &> /dev/null; then
        echo -e "${RED}Error: rustup no encontrado${NC}"
        exit 1
    fi
    
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        echo -e "${YELLOW}Instalando target Windows...${NC}"
        rustup target add x86_64-pc-windows-gnu
    fi
    
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo -e "${RED}Error: MinGW no encontrado${NC}"
        echo "Instalar con: sudo apt-get install gcc-mingw-w64-x86-64"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Requisitos OK${NC}"
}

# Build
build() {
    local mode=${1:-release}
    
    echo -e "\n${YELLOW}Compilando ($mode)...${NC}"
    
    if [ "$mode" == "release" ]; then
        cargo build --release --target x86_64-pc-windows-gnu
        local exe="target/x86_64-pc-windows-gnu/release/find-my-cursor.exe"
    else
        cargo build --target x86_64-pc-windows-gnu
        local exe="target/x86_64-pc-windows-gnu/debug/find-my-cursor.exe"
    fi
    
    if [ -f "$exe" ]; then
        local size=$(du -h "$exe" | cut -f1)
        echo -e "${GREEN}✓ Build exitoso${NC}"
        echo -e "  Ejecutable: ${YELLOW}$exe${NC}"
        echo -e "  Tamaño: ${YELLOW}$size${NC}"
    else
        echo -e "${RED}Error: No se generó el ejecutable${NC}"
        exit 1
    fi
}

# Main
check_requirements

case "${1:-release}" in
    debug)
        build debug
        ;;
    release)
        build release
        ;;
    *)
        echo "Uso: $0 [debug|release]"
        exit 1
        ;;
esac
