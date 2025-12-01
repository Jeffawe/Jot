.PHONY: all install hooks setup start stop status clean rebuild help uninstall install-llm clean-data clean-llm dev-build dev-run dev-test dev-check logs errors db-info restart

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

JOTX_DIR := $(HOME)/.jotx

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
	@chmod +x ./src/scripts/install_jot.sh
	@./src/scripts/install_jot.sh

install-llm:
	@echo "📦 Jotx requires Ollama to work. We will install it for you!"
	@echo ""
	@read -p "Continue with installation? (y/N) " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		chmod +x ./src/scripts/install_llm.sh; \
		if ./src/scripts/install_llm.sh; then \
			echo ""; \
			echo "✅ LLM setup complete! You can now use: jotx ask <query>"; \
		else \
			echo ""; \
			echo "❌ Installation failed. Try running:"; \
			echo "   make install-llm or use jotx handle-llm"; \
			exit 1; \
		fi \
	else \
		echo "❌ Cancelled"; \
		echo "   You can install later with: make install-llm or use jotx handle-llm"; \
	fi

hooks:
	@echo "🔗 Setting up shell hooks..."
	@chmod +x ./src/scripts/setup_hook.sh
	@./src/scripts/setup_hook.sh
	@echo "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"

setup: install hooks install-llm
	@mkdir -p $(JOTX_DIR)
	@echo "$(PWD)" > $(JOTX_DIR)/path
	@echo ""
	@echo "📁 Saved repo path to $(JOTX_DIR)/path"
	@echo "   -> $(PWD)"
	@echo ""
	@echo "✅ Setup complete!"
	@echo ""
	@echo "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"
	@echo "Then start jotx with: jotx run"

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

uninstall: stop clean clean-data clean-llm
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

clean-llm:
	@echo "🤖 LLM Cleanup Options"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "This will let you:"
	@echo "  • Remove downloaded Ollama models"
	@echo "  • Optionally uninstall Ollama itself"
	@echo ""
	@read -p "Continue? (y/N) " -n 1 -r; \
	echo; \
	if [[ ! $$REPLY =~ ^[Yy]$$ ]]; then \
		echo "❌ Cancelled"; \
		exit 0; \
	fi; \
	echo ""; \
	if command -v ollama &> /dev/null; then \
		echo "📦 Installed Ollama models:"; \
		ollama list || echo "   (none)"; \
		echo ""; \
		read -p "Remove Ollama models? (y/N) " -n 1 -r; \
		echo; \
		if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
			for model in $$(ollama list | tail -n +2 | awk '{print $$1}'); do \
				if [ -n "$$model" ]; then \
					echo "  🗑️  Removing $$model..."; \
					ollama rm $$model 2>/dev/null || true; \
				fi \
			done; \
			echo "✅ Models removed"; \
		else \
			echo "ℹ️  Keeping models"; \
		fi; \
		echo ""; \
		read -p "Uninstall Ollama completely? (y/N) " -n 1 -r; \
		echo; \
		if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
			echo "🗑️  Uninstalling Ollama..."; \
			if [[ "$$OSTYPE" == "darwin"* ]]; then \
				if command -v brew &> /dev/null && brew list ollama &> /dev/null; then \
					brew uninstall ollama; \
				else \
					sudo rm -f /usr/local/bin/ollama; \
					rm -rf ~/.ollama; \
				fi; \
			else \
				sudo systemctl stop ollama 2>/dev/null || true; \
				sudo systemctl disable ollama 2>/dev/null || true; \
				sudo rm -f /usr/local/bin/ollama; \
				sudo rm -f /etc/systemd/system/ollama.service; \
				sudo rm -rf /usr/share/ollama; \
				rm -rf ~/.ollama; \
			fi; \
			echo "✅ Ollama uninstalled"; \
		else \
			echo "ℹ️  Keeping Ollama (you can use it for other projects)"; \
		fi; \
	else \
		echo "ℹ️  Ollama not installed"; \
	fi; \
	echo ""; \
	echo "✅ LLM cleanup complete"

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