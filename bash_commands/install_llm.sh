#!/bin/bash

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  JotX LLM Setup (Ollama Installation) ║${NC}"
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo ""

# Check if Ollama is already installed
if command -v ollama &> /dev/null; then
    echo -e "${GREEN}✓ Ollama is already installed${NC}"
    OLLAMA_VERSION=$(ollama --version 2>/dev/null || echo "unknown")
    echo -e "  Version: ${OLLAMA_VERSION}"
else
    echo -e "${YELLOW}→ Installing Ollama...${NC}"
    
    # Detect OS and install accordingly
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "  Detected: Linux"
        curl -fsSL https://ollama.com/install.sh | sh
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  Detected: macOS"
        # Check if Homebrew is available
        if command -v brew &> /dev/null; then
            brew install ollama
        else
            curl -fsSL https://ollama.com/install.sh | sh
        fi
    else
        echo -e "${RED}✗ Unsupported OS: $OSTYPE${NC}"
        echo "  Please install Ollama manually from: https://ollama.com"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Ollama installed successfully${NC}"
fi

echo ""

# Check if Ollama service is running
echo -e "${YELLOW}→ Checking Ollama service...${NC}"
if curl -s http://localhost:11434 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Ollama service is running${NC}"
else
    echo -e "${YELLOW}→ Starting Ollama service...${NC}"
    
    # Try to start Ollama in background
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # On Linux, start as systemd service if available
        if systemctl is-active --quiet ollama 2>/dev/null; then
            sudo systemctl start ollama
        else
            nohup ollama serve > /dev/null 2>&1 &
            sleep 3
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # On macOS, start as background process
        nohup ollama serve > /dev/null 2>&1 &
        sleep 3
    fi
    
    # Verify it started
    if curl -s http://localhost:11434 > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Ollama service started${NC}"
    else
        echo -e "${RED}✗ Failed to start Ollama service${NC}"
        echo "  Try running manually: ollama serve"
        exit 1
    fi
fi

echo ""

DEFAULT_MODEL="qwen2.5:1.5b"

echo -e "${YELLOW}→ Checking for model: ${DEFAULT_MODEL}${NC}"

if ollama list | grep -q "$DEFAULT_MODEL"; then
    echo -e "${GREEN}✓ Model ${DEFAULT_MODEL} already installed${NC}"
else
    echo -e "${YELLOW}→ Downloading model: ${DEFAULT_MODEL} (~300MB)${NC}"
    echo "  This may take a few minutes..."
    
    ollama pull "$DEFAULT_MODEL"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Model ${DEFAULT_MODEL} downloaded successfully${NC}"
    else
        echo -e "${RED}✗ Failed to download model${NC}"
        exit 1
    fi
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✓ Setup Complete!                  ║${NC}"
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo ""
echo -e "You can now use natural language search:"
echo -e "  ${CYAN}jotx ask \"ssh I used yesterday\"${NC}"
echo ""
echo -e "To manage models:"
echo -e "  ${CYAN}jotx handle-llm${NC}"
echo ""