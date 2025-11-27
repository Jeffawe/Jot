#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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
    echo "Jotx needs to be in your PATH to work."
    echo ""
    
    # Use read with timeout (10 seconds)
    echo "Add to PATH automatically? (Y/n) [Auto-yes in 10s]"
    if read -t 10 -n 1 -r; then
        echo  # Move to new line after input
    else
        # Timeout occurred, default to Yes
        REPLY="y"
        echo  # Move to new line
        echo -e "${BLUE}ℹ️  No input received, defaulting to Yes${NC}"
    fi
    
    # Default to Yes if empty or Y/y, No only if explicitly n/N
    if [[ -z "$REPLY" ]] || [[ ! $REPLY =~ ^[Nn]$ ]]; then
        CONFIGS_UPDATED=0
        
        # Add to .zshrc if exists
        if [ -f "$HOME/.zshrc" ]; then
            if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zshrc" 2>/dev/null; then
                echo "" >> "$HOME/.zshrc"
                echo '# Added by Jotx installer' >> "$HOME/.zshrc"
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
                echo -e "${GREEN}✅ Added to ~/.zshrc${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.zshrc${NC}"
            fi
        fi
        
        # Add to .bashrc if exists
        if [ -f "$HOME/.bashrc" ]; then
            if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc" 2>/dev/null; then
                echo "" >> "$HOME/.bashrc"
                echo '# Added by Jotx installer' >> "$HOME/.bashrc"
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
                echo -e "${GREEN}✅ Added to ~/.bashrc${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.bashrc${NC}"
            fi
        fi
        
        # Add to .bash_profile if exists (macOS)
        if [ -f "$HOME/.bash_profile" ]; then
            if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bash_profile" 2>/dev/null; then
                echo "" >> "$HOME/.bash_profile"
                echo '# Added by Jotx installer' >> "$HOME/.bash_profile"
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bash_profile"
                echo -e "${GREEN}✅ Added to ~/.bash_profile${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.bash_profile${NC}"
            fi
        fi
        
        if [ $CONFIGS_UPDATED -eq 0 ]; then
            echo -e "${YELLOW}⚠️  No shell config files found${NC}"
            echo "Manually add: export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
        
        # Update PATH for current session
        export PATH="$HOME/.local/bin:$PATH"
        echo -e "${GREEN}✅ PATH updated for current session${NC}"
    else
        echo -e "${YELLOW}⚠️  Skipped. Add manually later with:${NC}"
        echo -e "${BLUE}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
    echo ""
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
    echo -e "  1. Restart your terminal (or run: ${CYAN}source ~/.bashrc${NC})"
    echo -e "  2. Check status: ${GREEN}jotx status${NC}"
    echo -e "  3. Try searching: ${GREEN}jotx search 'test'${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}❌ Setup failed${NC}"
    echo "You can try running setup manually:"
    echo "  jotx setup"
    exit 1
fi