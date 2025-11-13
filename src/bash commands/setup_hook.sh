#!/bin/bash

set -e  # Exit on error

# Detect shell
SHELL_NAME=$(basename "$SHELL")

# Hook configuration
BASH_HOOK='PROMPT_COMMAND='\''history -a; cmd=$(history 1 | sed "s/^[ ]*[0-9]*[ ]*//"); $DIGITAL_MEMORY_BIN capture "$cmd"'\'''
ZSH_HOOK='precmd() { cmd=$(fc -ln -1 | sed "s/^[ ]*//"); $DIGITAL_MEMORY_BIN capture "$cmd"; }'

add_hook() {
    local rc_file=$1
    local hook_line=$2
    
    # Check if rc file exists, create if it doesn't
    if [ ! -f "$rc_file" ]; then
        echo "Creating $rc_file..."
        touch "$rc_file"
    fi
    
    echo "Setting up hook in $rc_file..."
    
    # Create backup
    cp "$rc_file" "${rc_file}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "📦 Backup created: ${rc_file}.backup.$(date +%Y%m%d_%H%M%S)"
    
    # Add digital memory marker comments for easy identification
    if ! grep -q "# DIGITAL_MEMORY_START" "$rc_file"; then
        echo "" >> "$rc_file"
        echo "# DIGITAL_MEMORY_START - Do not edit this section manually" >> "$rc_file"
        echo 'export DIGITAL_MEMORY_BIN="$HOME/.digital_memory/memory"' >> "$rc_file"
        echo "$hook_line" >> "$rc_file"
        echo "# DIGITAL_MEMORY_END" >> "$rc_file"
        echo "✅ Hook added to $rc_file"
    else
        echo "⚠️  Digital Memory hook already exists in $rc_file"
        echo "   Remove the section between DIGITAL_MEMORY_START and DIGITAL_MEMORY_END to reinstall"
    fi
}

case "$SHELL_NAME" in
    bash)
        add_hook "$HOME/.bashrc" "$BASH_HOOK"
        ;;
    zsh)
        add_hook "$HOME/.zshrc" "$ZSH_HOOK"
        ;;
    *)
        echo "❌ Unsupported shell: $SHELL_NAME"
        echo "   Supported shells: bash, zsh"
        exit 1
        ;;
esac

echo ""
echo "✅ Hook setup complete!"
echo "   Please restart your terminal or run: source $HOME/.${SHELL_NAME}rc"