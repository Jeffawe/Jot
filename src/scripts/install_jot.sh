#!/usr/bin/env bash
set -e

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  JotX LLM Global Installation ║${NC}"
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo ""

echo "🔧 Installing Jot CLI globally..."
echo ""

# Install Rust if missing
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

if [ -f "Cargo.toml" ]; then
    # Local install (for developers)
    echo "🔨 Building from local source..."
    cargo install --path .
else
    # Remote install (for end users)
    echo "📥 Installing from GitHub..."
    cargo install --git https://github.com/Jeffawe/Jot.git jotx
fi

if [ $? -ne 0 ]; then
    echo "❌ Failed to build jotx"
    exit 1
fi



# Determine shell config file
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
    SHELL_NAME="zsh"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
    SHELL_NAME="bash"
else
    SHELL_RC="$HOME/.profile"
    SHELL_NAME="sh"
fi

echo ""
echo "🔍 Detected shell: $SHELL_NAME"

# Add to PATH if missing
if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo '# Added by jotx installer' >> "$SHELL_RC"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$SHELL_RC"
    echo "✅ Added ~/.cargo/bin to PATH"
else
    echo "ℹ️  ~/.cargo/bin already in PATH"
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     🎉 Installation Complete!        ║${NC}"
echo -e "${GREEN}╔═══════════════════════════════════════════════╗${NC}"
echo ""
echo "📝 Note: Embedding models will be downloaded automatically"
echo "   on first use (~50MB for the default model)"
echo ""
echo "To use jotx in your current terminal, run:"
echo "  source $SHELL_RC"
echo ""
echo "Run jotx help to get started and see available commands."