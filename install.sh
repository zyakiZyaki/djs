#!/bin/bash
# DJS Installer — like bun: curl -fsSL https://djs.dev/install.sh | sh

set -e

DJS_VERSION="0.1.0"
INSTALL_DIR="${DJS_INSTALL_DIR:-$HOME/.djs}"
BIN_URL="https://github.com/zyakiZyaki/djs/releases/download/v${DJS_VERSION}/djs"

echo "🚀 DJS Installer v${DJS_VERSION}"
echo ""

# Check if already installed
if [ -f "$INSTALL_DIR/bin/djs" ]; then
    echo "✓ DJS already installed at $INSTALL_DIR/bin/djs"
    echo " To reinstall: rm -rf $INSTALL_DIR and run this script again"
    exit 0
fi

# Create directories
mkdir -p "$INSTALL_DIR/bin"

# Download binary
echo "📥 Downloading DJS v${DJS_VERSION}..."
if command -v curl &> /dev/null; then
    curl -fsSL -o "$INSTALL_DIR/bin/djs" "$BIN_URL" || {
        echo "❌ Failed to download DJS binary"
        echo " Make sure you have curl installed and internet access"
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget -q -O "$INSTALL_DIR/bin/djs" "$BIN_URL" || {
        echo "❌ Failed to download DJS binary"
        exit 1
    }
else
    echo "❌ Neither curl nor wget found"
    exit 1
fi

# Make executable
chmod +x "$INSTALL_DIR/bin/djs"

# Add to PATH if needed
SHELL_CONFIG=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_CONFIG="$HOME/.bash_profile"
fi

PATH_EXPORT="export PATH=\"\$INSTALL_DIR/bin:\$PATH\""
if [ -n "$SHELL_CONFIG" ] && ! grep -q "djs" "$SHELL_CONFIG" 2>/dev/null; then
    echo "" >> "$SHELL_CONFIG"
    echo "# DJS - Declarative JavaScript VM" >> "$SHELL_CONFIG"
    echo "export DJS_INSTALL_DIR=\"$INSTALL_DIR\"" >> "$SHELL_CONFIG"
    echo "export PATH=\"\$DJS_INSTALL_DIR/bin:\$PATH\"" >> "$SHELL_CONFIG"
    echo "✓ Added DJS to PATH in $SHELL_CONFIG"
fi

echo ""
echo "✅ DJS installed successfully!"
echo ""
echo "To get started:"
echo " export PATH=\"$INSTALL_DIR/bin:\$PATH\""
echo " djs help"
echo " djs run src/main.js"
echo ""
