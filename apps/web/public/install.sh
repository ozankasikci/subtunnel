#!/bin/sh
set -e

REPO="winterwindgames/subtunnel"
INSTALL_DIR="/usr/local/bin"

main() {
    echo "Installing SubTunnel..."
    echo ""

    # Detect OS
    OS="$(uname -s)"
    case "$OS" in
        Linux)  OS="linux" ;;
        Darwin) OS="macos" ;;
        *)      echo "Error: Unsupported OS: $OS"; exit 1 ;;
    esac

    # Detect arch
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    # Map to Rust target
    case "${OS}-${ARCH}" in
        linux-x86_64)   TARGET="x86_64-unknown-linux-gnu" ;;
        linux-aarch64)   TARGET="aarch64-unknown-linux-gnu" ;;
        macos-x86_64)    TARGET="x86_64-apple-darwin" ;;
        macos-aarch64)   TARGET="aarch64-apple-darwin" ;;
        *)               echo "Error: Unsupported platform: ${OS}-${ARCH}"; exit 1 ;;
    esac

    # Get latest release tag
    echo "Fetching latest release..."
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command -v curl >/dev/null 2>&1; then
        RELEASE_JSON=$(curl -sL "$RELEASE_URL")
    elif command -v wget >/dev/null 2>&1; then
        RELEASE_JSON=$(wget -qO- "$RELEASE_URL")
    else
        echo "Error: curl or wget required"; exit 1
    fi

    TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')
    if [ -z "$TAG" ]; then
        echo "Error: Could not determine latest release. Is the repo public or is there a release?"
        exit 1
    fi

    ARCHIVE="subtunnel-${TAG}-${TARGET}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE}"

    echo "Downloading SubTunnel ${TAG} for ${OS}/${ARCH}..."
    
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -sL "$DOWNLOAD_URL" -o "${TMPDIR}/${ARCHIVE}"
    else
        wget -q "$DOWNLOAD_URL" -O "${TMPDIR}/${ARCHIVE}"
    fi

    # Extract
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    # Install
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMPDIR}/subtunnel" "${INSTALL_DIR}/subtunnel"
    else
        echo "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "${TMPDIR}/subtunnel" "${INSTALL_DIR}/subtunnel"
    fi
    chmod +x "${INSTALL_DIR}/subtunnel"

    echo ""
    echo "✓ SubTunnel ${TAG} installed to ${INSTALL_DIR}/subtunnel"
    echo ""
    "${INSTALL_DIR}/subtunnel" --version 2>/dev/null || true
    echo ""
    echo "Get started:"
    echo "  subtunnel local http 3000"
    echo ""
}

main
