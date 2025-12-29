#!/bin/sh
# PLM installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/DIO0550/plugin-manager/main/install.sh | sh

set -e

REPO="DIO0550/plugin-manager"
BINARY_NAME="plm"
INSTALL_DIR="${HOME}/.local/bin"

# 色付き出力
info() { printf "\033[0;34m[INFO]\033[0m %s\n" "$1"; }
error() { printf "\033[0;31m[ERROR]\033[0m %s\n" "$1" >&2; exit 1; }
success() { printf "\033[0;32m[OK]\033[0m %s\n" "$1"; }

# OS検出
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported OS: $(uname -s)" ;;
    esac
}

# アーキテクチャ検出
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# ターゲット名を生成
get_target() {
    local os="$1"
    local arch="$2"

    case "$os" in
        linux)   echo "${arch}-unknown-linux-gnu" ;;
        darwin)  echo "${arch}-apple-darwin" ;;
        windows) echo "${arch}-pc-windows-gnu" ;;
    esac
}

# 最新バージョンを取得
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# メイン処理
main() {
    info "Detecting platform..."
    OS=$(detect_os)
    ARCH=$(detect_arch)
    TARGET=$(get_target "$OS" "$ARCH")

    info "Platform: ${OS}/${ARCH} (${TARGET})"

    info "Fetching latest version..."
    VERSION=$(get_latest_version)
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi
    info "Latest version: ${VERSION}"

    # ダウンロードURL
    if [ "$OS" = "windows" ]; then
        ARCHIVE="${BINARY_NAME}-${VERSION}-${TARGET}.zip"
    else
        ARCHIVE="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
    fi
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

    # 一時ディレクトリ
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    info "Downloading ${ARCHIVE}..."
    curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE}" || \
        error "Failed to download from ${DOWNLOAD_URL}"

    info "Extracting..."
    cd "$TMP_DIR"
    if [ "$OS" = "windows" ]; then
        unzip -q "$ARCHIVE"
    else
        tar xzf "$ARCHIVE"
    fi

    # インストールディレクトリ作成
    mkdir -p "$INSTALL_DIR"

    info "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
    if [ "$OS" = "windows" ]; then
        mv "${BINARY_NAME}.exe" "${INSTALL_DIR}/"
    else
        mv "$BINARY_NAME" "${INSTALL_DIR}/"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    success "Successfully installed ${BINARY_NAME} ${VERSION}"

    # PATHチェック
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            info "Add the following to your shell config (.bashrc, .zshrc, etc.):"
            echo ""
            echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
            echo ""
            ;;
    esac
}

main "$@"
