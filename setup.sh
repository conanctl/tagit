#!/usr/bin/env sh
set -e

REPO="conanctl/tagit"
BIN="tag"

OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
  Linux*)  OS="unknown-linux-musl" ;;
  Darwin*) OS="apple-darwin" ;;
  *) echo "❌ Unsupported OS: $OS" >&2; exit 1 ;;
esac
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "❌ Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac
TARGET="${ARCH}-${OS}"

VERSION="${TAGIT_VERSION:-latest}"
if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep -Po '"tag_name": *"\K[^"]+')"
fi

TARBALL="${BIN}-${TARGET}.tar.gz"
MIRROR="${TAGIT_MIRROR:-https://github.com/${REPO}/releases/download/${VERSION}}"
URL="${MIRROR}/${TARBALL}"

printf '👉 Downloading %s\n' "$URL"
TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

curl -#fL "$URL" -o "$TMP_DIR/$TARBALL"

tar -xzf "$TMP_DIR/$TARBALL" -C "$TMP_DIR"

DEST="${TAGIT_DEST:-/usr/local/bin}"
mkdir -p "$DEST"
install -m 755 "$TMP_DIR/$BIN" "$DEST/$BIN"

cat <<'EOF'
✅ tagit installed!

Add this to your shell config (e.g., ~/.zshrc) to enable the `tag jump` command:

function tag() {
  if [ "$1" = "jump" ]; then
    local o; o="$(TAGIT_SHELL_INTEGRATION=1 tag "$@")"
    [ -n "$o" ] && [ "$o" != ":" ] && eval "$o"
  else
    command tag "$@"
  fi
}

Reload your shell and enjoy 🚀
EOF 