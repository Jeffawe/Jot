.PHONY: all install hooks setup start stop status clean rebuild help

# Default target
all: help

# Detect OS
UNAME := $(shell uname -s)
ifeq ($(UNAME),Darwin)
    OS := macos
else ifeq ($(UNAME),Linux)
    OS := linux
else
    OS := windows
endif

help:
	@echo "Jotx - Digital Memory CLI"
	@echo ""
	@echo "Available commands:"
	@echo "  make setup      - Full installation (install + hooks + start)"
	@echo "  make install    - Build and install jotx binary"
	@echo "  make hooks      - Setup shell hooks"
	@echo "  make start      - Start jotx daemon"
	@echo "  make stop       - Stop jotx daemon"
	@echo "  make status     - Check jotx status"
	@echo "  make clean      - Remove build artifacts and data"
	@echo "  make rebuild    - Clean and reinstall"
	@echo "  make uninstall  - Remove jotx completely"
	@echo ""
	@echo "Detected OS: $(OS)"

install:
	@echo "📦 Installing jotx..."
	@chmod +x ./bash_commands/install_jot.sh
	@./bash_commands/install_jot.sh

hooks:
	@echo "🔗 Setting up shell hooks..."
	@chmod +x ./bash_commands/setup_hook.sh
	@./bash_commands/setup_hook.sh
	@echo "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"

setup: install hooks
	@echo ""
	@echo "✅ Setup complete!"
	@echo ""
	@echo "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"
	@echo "Then start jotx with: jotx start"

start:
	@echo "▶️  Starting jotx daemon..."
	@jotx run
	@echo "✅ Jotx started! Use 'make status' to check."

stop:
	@echo "⏹️  Stopping jotx daemon..."
	@jotx exit || echo "Jotx was not running"

status:
	@jotx status

clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -f /tmp/jotx.pid
	@rm -f /tmp/jotx.log
	@rm -f /tmp/jotx.err
	@echo "✅ Clean complete"

clean-data:
	@echo "⚠️  This will delete all stored data!"
	@read -p "Are you sure? (y/N) " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		rm -rf ~/.jotx; \
		echo "✅ Data deleted"; \
	else \
		echo "❌ Cancelled"; \
	fi

rebuild: clean install
	@echo "✅ Rebuild complete"
	@echo "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"

uninstall: stop clean clean-data
	@echo "🗑️  Uninstalling jotx..."
	@cargo uninstall jotx || echo "Binary already removed"
	@echo ""

	@if [ -f ~/.zshrc ]; then \
		if grep -q "# JOTX_START" ~/.zshrc; then \
			cp ~/.zshrc ~/.zshrc.backup.$$(date +%Y%m%d_%H%M%S); \
			sed -i.tmp '/# JOTX_START/,/# JOTX_END/d' ~/.zshrc 2>/dev/null || \
			sed -i '' '/# JOTX_START/,/# JOTX_END/d' ~/.zshrc 2>/dev/null; \
			rm -f ~/.zshrc.tmp; \
			echo "✅ Removed hooks from ~/.zshrc"; \
		fi \
	fi
	@if [ -f ~/.bashrc ]; then \
		if grep -q "# JOTX_START" ~/.bashrc; then \
			cp ~/.bashrc ~/.bashrc.backup.$$(date +%Y%m%d_%H%M%S); \
			sed -i.tmp '/# JOTX_START/,/# JOTX_END/d' ~/.bashrc 2>/dev/null || \
			sed -i '' '/# JOTX_START/,/# JOTX_END/d' ~/.bashrc 2>/dev/null; \
			rm -f ~/.bashrc.tmp; \
			echo "✅ Removed hooks from ~/.bashrc"; \
		fi \
	fi

	@echo ""
	@echo "✅ Uninstall complete"
	@echo "   Run 'source ~/.zshrc' (or ~/.bashrc) to reload your shell"

# Development targets
dev-build:
	@echo "🔨 Building in debug mode..."
	@cargo build

dev-run: dev-build
	@echo "▶️  Running in debug mode..."
	@./target/debug/jotx run

dev-test:
	@echo "🧪 Running tests..."
	@cargo test

dev-check:
	@echo "🔍 Running cargo check..."
	@cargo check

# Logs
logs:
	@echo "📋 Recent logs:"
	@tail -n 50 /tmp/jotx.log 2>/dev/null || echo "No logs found"

errors:
	@echo "❌ Recent errors:"
	@tail -n 50 /tmp/jotx.err 2>/dev/null || echo "No errors found"

# Database
db-info:
	@echo "📊 Database info:"
	@ls -lh ~/.jotx/jotx.db 2>/dev/null || echo "Database not found"
	@echo ""
	@echo "Entry counts:"
	@jotx ask "count entries" 2>/dev/null || echo "Run 'jotx run' first"

# Quick restart
restart: stop start
	@echo "🔄 Restarted jotx"