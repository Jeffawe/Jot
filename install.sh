#!/usr/bin/env bash
set -e

echo "🔧 Installing Jotx..."

# Install Rust if missing
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Clone if not exists
if [ ! -d "jot-cli" ]; then
    git clone https://github.com/Jeffawe/Jot.git jot-cli
    cd jot-cli
else
    cd jot-cli
    git pull
fi

# Run your setup
make setup

jotx run

echo "🎉 Installed and Running!"