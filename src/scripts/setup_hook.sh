#!/bin/bash
set -e  # Exit on error

# Detect shell
SHELL_NAME=$(basename "$SHELL")

# Hook configuration - SAFE versions that fail silently
# Bash hook with context - fails silently if jotx isn't available/running
BASH_HOOK='PROMPT_COMMAND='\''if command -v jotx >/dev/null 2>&1; then history -a; cmd=$(history 1 | sed "s/^[ ]*[0-9]*[ ]*//"); jotx capture --cmd "$cmd" --pwd "$PWD" --user "$USER" --host "$HOSTNAME" 2>/dev/null || true; fi'\'''

# Zsh hook with context - fails silently if jotx isn't available/running
ZSH_HOOK='precmd() { if command -v jotx >/dev/null 2>&1; then cmd=$(fc -ln -1 | sed "s/^[ ]*//"); jotx capture --cmd "$cmd" --pwd "$PWD" --user "$USER" --host "$HOSTNAME" 2>/dev/null || true; fi }'

BASH_SEARCH_WRAPPER='export GIN_MODE=release
export LLAMA_LOG_LEVEL=0

jotx() {
    if [ "$1" = "search" ]; then
        shift
        local result=$(command jotx search "$@" --print-only)
        if [ -n "$result" ]; then
            echo "Found: $result"
            history -s "$result"  # Add to history for ↑ arrow access
        fi
    elif [ "$1" = "ask" ]; then
        shift
        local result=$(command jotx ask "$@" --print-only)
        if [ -n "$result" ]; then
            echo "Found: $result"
            history -s "$result"  # Add to history for ↑ arrow access
        fi
    else
        command jotx "$@"
    fi
}
js() { jotx search "$@"; }
ja() { jotx ask "$@"; }'

# Zsh search wrapper - uses print -z (Zsh built-in)
ZSH_SEARCH_WRAPPER='export GIN_MODE=release
export LLAMA_LOG_LEVEL=0

jotx() {
    if [ "$1" = "search" ]; then
        shift
        local result=$(command jotx search "$@" --print-only)
        if [ -n "$result" ]; then
            print -z "$result"
        fi
    elif [ "$1" = "ask" ]; then
        shift
        local result=$(command jotx ask "$@" --print-only)
        if [ -n "$result" ]; then
            print -z "$result"
        fi
    else
        command jotx "$@"
    fi
}
js() { jotx search "$@"; }
ja() { jotx ask "$@"; }'

add_hook() {
    local rc_file=$1
    local hook_line=$2
    local search_wrapper=$3
    
    # Check if rc file exists, create if it doesn't
    if [ ! -f "$rc_file" ]; then
        echo "Creating $rc_file..."
        touch "$rc_file"
    fi
    
    echo "Setting up hook in $rc_file..."
    
    # Create backup
    cp "$rc_file" "${rc_file}.backup.$(date +%Y%m%d_%H%M%S)"
    
    # Remove old hook if exists
    if grep -q "# JOTX_START" "$rc_file"; then
        echo "🔄 Removing old jotx hook..."
        # macOS and Linux compatible sed
        sed -i.tmp '/# JOTX_START/,/# JOTX_END/d' "$rc_file" 2>/dev/null || \
        sed -i '' '/# JOTX_START/,/# JOTX_END/d' "$rc_file" 2>/dev/null
        rm -f "${rc_file}.tmp"
    fi
    
    # Add new safe hook
    echo "" >> "$rc_file"
    echo "# JOTX_START - Do not edit this section manually" >> "$rc_file"
    echo "# This hook captures command history for jotx" >> "$rc_file"
    echo "# It fails silently if jotx is not running to avoid interrupting your workflow" >> "$rc_file"
    echo "$hook_line" >> "$rc_file"
    echo "" >> "$rc_file"
    echo "# Jotx search wrapper - type 'js <query>' to search and insert command" >> "$rc_file"
    echo "$search_wrapper" >> "$rc_file"
    echo "# JOTX_END" >> "$rc_file"
    echo "✅ Hook added to $rc_file"
}

case "$SHELL_NAME" in
    bash)
        add_hook "$HOME/.bashrc" "$BASH_HOOK" "$BASH_SEARCH_WRAPPER"
        ;;
    zsh)
        add_hook "$HOME/.zshrc" "$ZSH_HOOK" "$ZSH_SEARCH_WRAPPER"
        ;;
    *)
        echo "❌ Unsupported shell: $SHELL_NAME"
        echo "   Supported shells: bash, zsh"
        exit 1
        ;;
esac

echo ""
echo ""