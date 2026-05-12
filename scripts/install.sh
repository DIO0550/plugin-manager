#!/usr/bin/env sh
# Install plm (Plugin Manager CLI) from GitHub Releases.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/DIO0550/plugin-manager/main/scripts/install.sh | sh
#
# Environment variables:
#   PLM_VERSION      Release tag to install (e.g. "v0.5.0"). Defaults to the latest release.
#   PLM_INSTALL_DIR  Directory to place the `plm` binary. Defaults to "$HOME/.local/bin".
#   PLM_REPO         GitHub "owner/repo". Defaults to "DIO0550/plugin-manager".
#   PLM_NO_MODIFY_PATH  If set to "1", skip appending PATH to your shell profile.

set -eu

REPO="${PLM_REPO:-DIO0550/plugin-manager}"
INSTALL_DIR="${PLM_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${PLM_VERSION:-}"

BINARY_NAME="plm"

info()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m!!\033[0m  %s\n' "$*" >&2; }
error() { printf '\033[1;31mxx\033[0m  %s\n' "$*" >&2; exit 1; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || error "required command not found: $1"
}

detect_target() {
  uname_s=$(uname -s)
  uname_m=$(uname -m)

  case "$uname_s" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*)
      error "Windows shell detected. Download the .zip from https://github.com/${REPO}/releases and extract plm.exe manually." ;;
    *) error "unsupported OS: $uname_s" ;;
  esac

  case "$uname_m" in
    x86_64|amd64)  arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) error "unsupported architecture: $uname_m" ;;
  esac

  printf '%s-%s' "$arch" "$os"
}

resolve_version() {
  if [ -n "$VERSION" ]; then
    printf '%s' "$VERSION"
    return
  fi
  require_cmd curl
  api_url="https://api.github.com/repos/${REPO}/releases/latest"
  tag=$(curl -fsSL "$api_url" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
  [ -n "$tag" ] || error "failed to resolve latest release tag from $api_url"
  printf '%s' "$tag"
}

main() {
  require_cmd curl
  require_cmd tar
  require_cmd mkdir
  require_cmd mktemp

  target=$(detect_target)
  version=$(resolve_version)
  asset="${BINARY_NAME}-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${asset}"

  info "Installing ${BINARY_NAME} ${version} (${target})"
  info "Download: ${url}"

  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  if ! curl -fsSL "$url" -o "${tmpdir}/${asset}"; then
    error "failed to download ${url}"
  fi

  tar -xzf "${tmpdir}/${asset}" -C "$tmpdir"
  [ -f "${tmpdir}/${BINARY_NAME}" ] || error "archive did not contain ${BINARY_NAME}"

  mkdir -p "$INSTALL_DIR"
  install_path="${INSTALL_DIR}/${BINARY_NAME}"
  mv "${tmpdir}/${BINARY_NAME}" "$install_path"
  chmod +x "$install_path"

  info "Installed: $install_path"

  case ":${PATH:-}:" in
    *":${INSTALL_DIR}:"*)
      info "PATH already contains ${INSTALL_DIR}. Run \`${BINARY_NAME} --version\` to verify."
      return 0
      ;;
  esac

  if [ "${PLM_NO_MODIFY_PATH:-0}" = "1" ]; then
    warn "${INSTALL_DIR} is not in your PATH (PLM_NO_MODIFY_PATH=1, skipping profile update)."
    return 0
  fi

  add_to_path "$INSTALL_DIR"
}

# Append an "export PATH" / "fish_add_path" line to the user's shell profile(s).
# Idempotent: skips files that already reference the directory.
add_to_path() {
  dir=$1
  shell_name=$(basename "${SHELL:-}")
  appended=0
  already=0

  case "$shell_name" in
    fish)
      line="fish_add_path $dir"
      try_append "$HOME/.config/fish/config.fish" "$line" "$dir"
      ;;
    zsh)
      line="export PATH=\"$dir:\$PATH\""
      try_append "$HOME/.zshrc" "$line" "$dir"
      ;;
    bash)
      line="export PATH=\"$dir:\$PATH\""
      try_append "$HOME/.bashrc" "$line" "$dir"
      ;;
    *)
      line="export PATH=\"$dir:\$PATH\""
      try_append "$HOME/.profile" "$line" "$dir"
      ;;
  esac

  if [ "$appended" -eq 1 ]; then
    info "PATH entry written. Open a new shell or run \`exec \$SHELL\` to pick it up."
  elif [ "$already" -eq 1 ]; then
    info "PATH entry already present in your shell profile."
  else
    warn "${dir} not added to PATH (no writable profile found)."
    printf '    Add this manually:\n        %s\n' "$line"
  fi
}

# Append $2 to file $1 unless the file already contains $3.
# Updates the outer `appended` / `already` counters instead of relying on return codes.
try_append() {
  rc=$1
  line=$2
  marker=$3

  rc_dir=$(dirname "$rc")
  [ -d "$rc_dir" ] || mkdir -p "$rc_dir" 2>/dev/null || return

  if [ -f "$rc" ] && grep -Fq "$marker" "$rc" 2>/dev/null; then
    info "Already referenced in $rc — leaving it untouched."
    already=1
    return
  fi

  if {
    printf '\n# Added by plm install.sh\n'
    printf '%s\n' "$line"
  } >> "$rc" 2>/dev/null; then
    info "Appended PATH entry to $rc"
    appended=1
  fi
}

main "$@"
