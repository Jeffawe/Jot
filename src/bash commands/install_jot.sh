#!/usr/bin/env bash
set -e  # Exit on error

echo "🔧 Installing Jot CLI globally..."
echo ""

# Step 1: Install the binary
echo "📦 Building and installing binary..."
cargo install --path .

# Step 2: Determine shell config file
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
echo "📝 Config file: $SHELL_RC"

# Step 3: Add to PATH if missing
if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo '# Added by jotx installer' >> "$SHELL_RC"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$SHELL_RC"
    echo "✅ Added ~/.cargo/bin to PATH in $SHELL_RC"
else
    echo "ℹ️  ~/.cargo/bin already in PATH"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "To use jotx in your current terminal, run:"
echo "  source $SHELL_RC"
echo ""
echo "Then you can run jotx from anywhere:"
echo "  jotx run      # Start the service"
echo "  jotx status   # Check if running"
echo "  jotx exit     # Stop the service"
echo ""