#!/usr/bin/env bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     JotX CLI Global Installation       ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
echo ""

# Parse arguments
AUTO_YES=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -y|--yes)
            AUTO_YES=true
            shift
            ;;
        -h|--help)
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -y, --yes   Automatically answer yes to all prompts"
            echo "  -h, --help  Show this help message"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

echo "🔧 Installing Jotx CLI globally..."
echo ""

# Install Rust if missing
if ! command -v cargo &> /dev/null; then
    echo "🦀 Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✅ Rust installed${NC}"
    echo ""
fi

# Check if we're in the repo or doing remote install
if [ -f "Cargo.toml" ] && grep -q 'name = "jotx"' Cargo.toml 2>/dev/null; then
    # Local install (for developers)
    echo "🔨 Building from local source..."
    cargo install --path . --force
else
    # Remote install (for end users)
    echo "📥 Installing from GitHub..."
    cargo install --git https://github.com/Jeffawe/Jot.git jotx --force
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to install jotx${NC}"
    exit 1
fi

echo -e "${GREEN}✅ jotx binary installed${NC}"
echo ""

# Cargo bin directory
CARGO_BIN="$HOME/.cargo/bin"

# Check if cargo bin is already in PATH
if [[ ":$PATH:" == *":$CARGO_BIN:"* ]]; then
    echo -e "${GREEN}✅ Cargo bin directory is already in PATH${NC}"
else
    echo -e "${YELLOW}⚠️  $CARGO_BIN is not in your PATH${NC}"
    echo ""
    echo "Jotx needs to be in your PATH to work."
    echo ""
    
    # Determine if we're in interactive mode
    SHOULD_ADD_PATH=false
    
    if [ "$AUTO_YES" = true ]; then
        # Auto-yes flag provided
        echo "Auto-yes mode: Adding to PATH automatically..."
        SHOULD_ADD_PATH=true
    elif [ -t 0 ]; then
        # Interactive mode (stdin is a terminal)
        echo "Add to PATH automatically? (Y/n) [Auto-yes in 10s]"
        if read -t 10 -n 1 -r < /dev/tty 2>/dev/null; then
            echo
            # Default to Yes unless explicitly N/n
            if [[ -z "$REPLY" ]] || [[ ! $REPLY =~ ^[Nn]$ ]]; then
                SHOULD_ADD_PATH=true
            fi
        else
            # Timeout - default to Yes
            echo
            echo -e "${BLUE}ℹ️  No input received, defaulting to Yes${NC}"
            SHOULD_ADD_PATH=true
        fi
    else
        # Non-interactive mode (piped from curl)
        echo -e "${BLUE}ℹ️  Non-interactive mode detected, adding to PATH automatically${NC}"
        SHOULD_ADD_PATH=true
    fi
    
    if [ "$SHOULD_ADD_PATH" = true ]; then
        CONFIGS_UPDATED=0
        
        # Add to .zshrc if exists
        if [ -f "$HOME/.zshrc" ]; then
            if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$HOME/.zshrc" 2>/dev/null; then
                echo "" >> "$HOME/.zshrc"
                echo '# Added by Jotx installer' >> "$HOME/.zshrc"
                echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.zshrc"
                echo -e "${GREEN}✅ Added to ~/.zshrc${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.zshrc${NC}"
            fi
        fi
        
        # Add to .bashrc if exists
        if [ -f "$HOME/.bashrc" ]; then
            if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$HOME/.bashrc" 2>/dev/null; then
                echo "" >> "$HOME/.bashrc"
                echo '# Added by Jotx installer' >> "$HOME/.bashrc"
                echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bashrc"
                echo -e "${GREEN}✅ Added to ~/.bashrc${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.bashrc${NC}"
            fi
        fi
        
        # Add to .bash_profile if exists (macOS)
        if [ -f "$HOME/.bash_profile" ]; then
            if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$HOME/.bash_profile" 2>/dev/null; then
                echo "" >> "$HOME/.bash_profile"
                echo '# Added by Jotx installer' >> "$HOME/.bash_profile"
                echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bash_profile"
                echo -e "${GREEN}✅ Added to ~/.bash_profile${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.bash_profile${NC}"
            fi
        fi
        
        # Add to .profile if exists (fallback)
        if [ -f "$HOME/.profile" ]; then
            if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$HOME/.profile" 2>/dev/null; then
                echo "" >> "$HOME/.profile"
                echo '# Added by Jotx installer' >> "$HOME/.profile"
                echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.profile"
                echo -e "${GREEN}✅ Added to ~/.profile${NC}"
                CONFIGS_UPDATED=1
            else
                echo -e "${BLUE}ℹ️  Already in ~/.profile${NC}"
            fi
        fi
        
        if [ $CONFIGS_UPDATED -eq 0 ]; then
            echo -e "${YELLOW}⚠️  No shell config files found or already configured${NC}"
        fi
        
        # Update PATH for current session
        export PATH="$HOME/.cargo/bin:$PATH"
        echo -e "${GREEN}✅ PATH updated for current session${NC}"
    else
        echo -e "${YELLOW}⚠️  Skipped. You'll need to add it manually:${NC}"
        echo -e "${BLUE}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
    fi
fi

# Detect current shell for instructions
CURRENT_SHELL_CONFIG="~/.zshrc or ~/.bashrc"
if [ -n "$ZSH_VERSION" ]; then
    CURRENT_SHELL_CONFIG="~/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    CURRENT_SHELL_CONFIG="~/.bashrc"
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          🎉 Installation Complete!            ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════╝${NC}"
echo ""
echo "📝 Notes:"
echo "  • Embedding models will download automatically on first use (~50MB)"
echo "  • Shell hooks will be set up when you run: jotx setup"
echo ""
echo "🚀 To get started:"
echo -e "  1. Reload your shell: ${CYAN}source $CURRENT_SHELL_CONFIG${NC}"
echo -e "  2. Run setup: ${BLUE}jotx setup${NC}"
echo -e "  3. Check status: ${BLUE}jotx status${NC}"
echo ""
echo "Need help? Run: ${BLUE}jotx --help${NC}"