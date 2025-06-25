#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/conanctl/tagit"
BINARY_NAME="tag"

show_banner() {
  echo "\033[1;32m                    _       \033[0m"
  echo "\033[1;32m   _              (_)  _    \033[0m"
  echo "\033[1;32m _| |_ _____  ____ _ _| |_  \033[0m"
  echo "\033[1;32m(_   _|____ |/ _  | (_   _) \033[0m"
  echo "\033[1;32m  | |_/ ___ ( (_| | | | |_  \033[0m"
  echo "\033[1;32m   \__)_____|\___ |_|  \__) \033[0m"
  echo "\033[1;32m            (_____|         \033[0m"
  echo
}

command_exists() { command -v "$1" >/dev/null 2>&1; }

show_banner

if ! command_exists cargo; then
  echo "[!] Rust toolchain (cargo) not found. Please install Rust from https://rustup.rs first." >&2
  exit 1
fi

echo "[+] Installing/Updating tagit via cargo …"
cargo install --git ${REPO} --force

cat <<'EOS'

✅ tagit installed!

Add the following to your shell config (e.g., ~/.zshrc) to enable directory jumping:

function tag() {
  if [[ "$1" == "jump" ]]; then
    local _out
    _out="$(TAGIT_SHELL_INTEGRATION=1 tag "$@")"
    [[ -n "${_out}" && "${_out}" != ":" ]] && eval "${_out}"
  else
    command tag "$@"
  fi
}

# Reload your shell or `source ~/.zshrc` and enjoy 🚀.
EOS 