#!/bin/bash
set -e  # Exit on error

echo "Installing sqlite-vec..."

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

# Determine platform and extension
case "$OS" in
    Darwin)
        PLATFORM="macos"
        EXT="dylib"
        ;;
    Linux)
        PLATFORM="linux"
        EXT="so"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows"
        EXT="dll"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64|AMD64)
        ARCH_SUFFIX="x86_64"
        ;;
    arm64|aarch64)
        ARCH_SUFFIX="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Fetch latest release version from GitHub API
echo "Fetching latest sqlite-vec release..."
if command -v curl &> /dev/null; then
    LATEST_RELEASE=$(curl -s https://api.github.com/repos/asg017/sqlite-vec/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
elif command -v wget &> /dev/null; then
    LATEST_RELEASE=$(wget -qO- https://api.github.com/repos/asg017/sqlite-vec/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Fallback to known version if API call fails
if [ -z "$LATEST_RELEASE" ]; then
    echo "⚠ Could not fetch latest version, using fallback v0.1.6"
    VERSION="0.1.6"
else
    # Remove 'v' prefix from version (v0.1.6 -> 0.1.6)
    VERSION="${LATEST_RELEASE#v}"
    echo "Latest version: $LATEST_RELEASE"
fi

# Build download URL
BASE_URL="https://github.com/asg017/sqlite-vec/releases/download"
FILENAME="sqlite-vec-${VERSION}-loadable-${PLATFORM}-${ARCH_SUFFIX}.tar.gz"
DOWNLOAD_URL="${BASE_URL}/v${VERSION}/${FILENAME}"

echo "Detected: $OS ($ARCH)"
echo "Downloading: $FILENAME"

# Determine installation directory
if [ "$OS" = "Darwin" ]; then
    # macOS: Try /usr/local/lib first, fallback to user directory
    if [ -w "/usr/local/lib" ]; then
        INSTALL_DIR="/usr/local/lib"
        NEED_SUDO=false
    else
        # Check if we can write with sudo
        if sudo -n true 2>/dev/null; then
            INSTALL_DIR="/usr/local/lib"
            NEED_SUDO=true
        else
            # Fallback to user directory
            INSTALL_DIR="$HOME/.local/lib"
            mkdir -p "$INSTALL_DIR"
            NEED_SUDO=false
            echo "Note: Installing to user directory (no sudo access): $INSTALL_DIR"
        fi
    fi
elif [[ "$OS" == MINGW* ]] || [[ "$OS" == MSYS* ]] || [[ "$OS" == CYGWIN* ]]; then
    # Windows: install to user's local AppData
    INSTALL_DIR="$LOCALAPPDATA/sqlite-vec"
    mkdir -p "$INSTALL_DIR"
    NEED_SUDO=false
else
    # Linux: Try user directory first, fallback to system
    if [ -w "$HOME/.local/lib" ] || mkdir -p "$HOME/.local/lib" 2>/dev/null; then
        INSTALL_DIR="$HOME/.local/lib"
        NEED_SUDO=false
    else
        INSTALL_DIR="/usr/local/lib"
        NEED_SUDO=true
    fi
fi

# Create directory if needed
if [ "$NEED_SUDO" = true ]; then
    echo "Creating directory with sudo: $INSTALL_DIR"
    sudo mkdir -p "$INSTALL_DIR" 2>/dev/null || true
else
    mkdir -p "$INSTALL_DIR" 2>/dev/null || true
fi

# Download
TMP_DIR="/tmp/sqlite-vec-install"
# Windows compatibility for temp dir
if [[ "$OS" == MINGW* ]] || [[ "$OS" == MSYS* ]] || [[ "$OS" == CYGWIN* ]]; then
    TMP_DIR="$TEMP/sqlite-vec-install"
fi

TMP_FILE="$TMP_DIR/${FILENAME}"
mkdir -p "$TMP_DIR"

echo "Downloading sqlite-vec..."
if command -v curl &> /dev/null; then
    curl -L -f -o "$TMP_FILE" "$DOWNLOAD_URL" || {
        echo "❌ Download failed from: $DOWNLOAD_URL"
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget -O "$TMP_FILE" "$DOWNLOAD_URL" || {
        echo "❌ Download failed from: $DOWNLOAD_URL"
        exit 1
    }
fi

# Extract tar.gz
echo "Extracting..."
tar -xzf "$TMP_FILE" -C "$TMP_DIR" || {
    echo "❌ Extraction failed"
    exit 1
}

# Find the library file in the extracted directory
EXTRACTED_FILE=$(find "$TMP_DIR" -name "vec0.${EXT}" | head -n 1)

if [ -z "$EXTRACTED_FILE" ]; then
    echo "❌ Could not find vec0.${EXT} in extracted files"
    echo "Contents of extracted directory:"
    ls -la "$TMP_DIR"
    exit 1
fi

echo "Found: $EXTRACTED_FILE"

# Install
TARGET_FILE="$INSTALL_DIR/vec0.${EXT}"

if [ "$NEED_SUDO" = true ]; then
    echo "Installing with sudo to: $TARGET_FILE"
    sudo mv "$EXTRACTED_FILE" "$TARGET_FILE" || {
        echo "❌ Failed to install. You may need to run this script with sudo."
        exit 1
    }
    sudo chmod 755 "$TARGET_FILE"
else
    mv "$EXTRACTED_FILE" "$TARGET_FILE" || {
        echo "❌ Failed to move file to: $TARGET_FILE"
        exit 1
    }
    chmod 755 "$TARGET_FILE" 2>/dev/null || true
fi

# Cleanup
rm -rf "$TMP_DIR"

echo "✅ sqlite-vec v${VERSION} installed to: $TARGET_FILE"

# Verify installation
if [ -f "$TARGET_FILE" ]; then
    echo "✅ Installation successful!"
    echo ""
    echo "sqlite-vec is now available at: $TARGET_FILE"
    
    # Platform-specific notes
    if [[ "$OS" == MINGW* ]] || [[ "$OS" == MSYS* ]] || [[ "$OS" == CYGWIN* ]]; then
        echo ""
        echo "Note: You may need to add this directory to your PATH:"
        echo "  $INSTALL_DIR"
    elif [ "$INSTALL_DIR" = "$HOME/.local/lib" ]; then
        echo ""
        echo "Note: Installed to user directory. Your app will automatically find it."
    fi
else
    echo "❌ Installation verification failed"
    exit 1
fi