#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║          Installing Jotx               ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        OS_TYPE="linux"
        ;;
    Darwin*)
        OS_TYPE="macos"
        ;;
    *)
        echo -e "${RED}❌ Unsupported operating system: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_TYPE="x86_64"
        ;;
    arm64|aarch64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${BLUE}Detected: ${OS_TYPE}-${ARCH_TYPE}${NC}"
echo ""

# Set installation directory
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="jotx"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"

# GitHub repository
REPO="Jeffawe/Jot"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/jotx-${OS_TYPE}-${ARCH_TYPE}"

# Create installation directory
echo -e "${YELLOW}📁 Creating installation directory...${NC}"
mkdir -p "$INSTALL_DIR"

# Download binary
echo -e "${YELLOW}📦 Downloading jotx binary...${NC}"
if command -v curl &> /dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$BINARY_PATH"
elif command -v wget &> /dev/null; then
    wget -q "$DOWNLOAD_URL" -O "$BINARY_PATH"
else
    echo -e "${RED}❌ Neither curl nor wget found. Please install one of them.${NC}"
    exit 1
fi

# Make binary executable
chmod +x "$BINARY_PATH"
echo -e "${GREEN}✅ Binary installed to: $BINARY_PATH${NC}"
echo ""

# Check if directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  $INSTALL_DIR is not in your PATH${NC}"
    echo ""
    echo "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo -e "${BLUE}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    echo ""
    
    # Offer to add it automatically
    read -p "Would you like to add it automatically? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -f "$HOME/.zshrc" ]; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
            echo -e "${GREEN}✅ Added to ~/.zshrc${NC}"
        fi
        if [ -f "$HOME/.bashrc" ]; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            echo -e "${GREEN}✅ Added to ~/.bashrc${NC}"
        fi
        export PATH="$HOME/.local/bin:$PATH"
    fi
fi

echo ""
echo -e "${YELLOW}🚀 Running jotx setup...${NC}"
echo ""

if "$BINARY_PATH" setup; then
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     ✅ Installation Complete!          ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    
    # Try to start jotx in background
    echo -e "${YELLOW}▶️  Starting jotx daemon...${NC}"
    "$BINARY_PATH" run > /dev/null 2>&1 &
    sleep 2  # Give it time to start
    
    if "$BINARY_PATH" status > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Jotx is running in the background!${NC}"
    else
        echo -e "${YELLOW}⚠️  Jotx may need manual start${NC}"
        echo -e "Run ${GREEN}jotx run${NC} to start it"
    fi
    
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Restart your terminal (or run: source ~/.bashrc)"
    echo "  2. Check status: ${GREEN}jotx status${NC}"
    echo "  3. Try searching: ${GREEN}jotx search 'test'${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}❌ Setup failed${NC}"
    echo "You can try running setup manually:"
    echo "  jotx setup"
    exit 1
fi