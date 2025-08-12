#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing ibtop...${NC}"

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        BINARY="ibtop-linux-amd64"
        ;;
    aarch64|arm64)
        BINARY="ibtop-linux-arm64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        echo "Supported: x86_64, aarch64/arm64"
        exit 1
        ;;
esac

# Get latest release version
echo "Fetching latest release..."
LATEST_VERSION=$(curl -s https://api.github.com/repos/JannikSt/ibtop/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_VERSION" ]; then
    echo -e "${RED}Error: Could not fetch latest release version${NC}"
    exit 1
fi

# Download URL
URL="https://github.com/JannikSt/ibtop/releases/download/$LATEST_VERSION/$BINARY"

echo "Downloading ibtop $LATEST_VERSION for $ARCH..."
if ! curl -L "$URL" -o ibtop 2>/dev/null; then
    echo -e "${RED}Error: Failed to download $URL${NC}"
    exit 1
fi

# Make executable
chmod +x ibtop

# Install to appropriate location
if [[ $EUID -eq 0 ]]; then
    # Running as root
    mv ibtop /usr/local/bin/
    echo -e "${GREEN}✅ Installed ibtop to /usr/local/bin/${NC}"
else
    # Running as regular user
    mkdir -p ~/.local/bin
    mv ibtop ~/.local/bin/
    echo -e "${GREEN}✅ Installed ibtop to ~/.local/bin/${NC}"
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo -e "${BLUE}💡 Add ~/.local/bin to your PATH:${NC}"
        echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
        echo "   source ~/.bashrc"
    fi
fi

echo -e "${GREEN}🚀 Installation complete! Run 'ibtop' to get started.${NC}"