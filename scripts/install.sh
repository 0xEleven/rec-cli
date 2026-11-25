#!/usr/bin/env bash
set -euo pipefail

# Universal Linux installer for rec-cli
# Supports: Debian, Ubuntu, Kali, Mint, Pop!_OS, Arch, Manjaro, EndeavourOS,
#           Fedora, RHEL, Rocky, Alma, CentOS, openSUSE, Alpine, Void, Gentoo
# Shells:   Zsh, Bash, Fish

VERSION="1.4.0"
PREFIX="${PREFIX:-$HOME/.local}"
INSTALL_DEPS=1
CONFIGURE_SHELLS=1
TARGET_SHELL="auto"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix) PREFIX="$2"; shift 2 ;;
    --system) PREFIX="/usr/local"; shift ;;
    --no-deps) INSTALL_DEPS=0; shift ;;
    --no-shell) CONFIGURE_SHELLS=0; shift ;;
    --shell) TARGET_SHELL="$2"; shift 2 ;;
    -h|--help)
      cat <<EOF
Usage: install.sh [options]
  --prefix DIR     Install prefix (default: ~/.local)
  --system         Install system-wide to /usr/local
  --shell SHELL    Shell to configure: auto|zsh|bash|fish|all (default: auto)
  --no-deps        Skip installing system dependencies
  --no-shell       Skip adding init line to shell rc files
EOF
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

C_RESET=$'\033[0m'
C_BOLD=$'\033[1m'
C_GREEN=$'\033[38;2;158;206;106m'
C_PURPLE=$'\033[38;2;187;154;247m'
C_CYAN=$'\033[38;2;125;207;255m'
C_YELLOW=$'\033[38;2;224;175;104m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${C_PURPLE}${C_BOLD}Installing rec-cli v${VERSION}${C_RESET} -> ${C_CYAN}${PREFIX}${C_RESET}"

install_deps() {
  local missing=()
  for cmd in asciinema ffmpeg python3 flock setsid; do
    command -v "$cmd" >/dev/null 2>&1 || missing+=("$cmd")
  done

  if (( ${#missing[@]} > 0 )); then
    echo -e "${C_CYAN}Installing missing dependencies:${C_RESET} ${missing[*]}"
    local sudo_cmd=""
    [[ $EUID -ne 0 ]] && command -v sudo >/dev/null 2>&1 && sudo_cmd="sudo"

    if command -v apt-get >/dev/null 2>&1; then
      $sudo_cmd apt-get update -qq
      $sudo_cmd apt-get install -y asciinema ffmpeg python3 util-linux
    elif command -v pacman >/dev/null 2>&1; then
      $sudo_cmd pacman -Sy --noconfirm --needed asciinema ffmpeg python util-linux
    elif command -v dnf >/dev/null 2>&1; then
      $sudo_cmd dnf install -y asciinema ffmpeg python3 util-linux
    elif command -v yum >/dev/null 2>&1; then
      $sudo_cmd yum install -y asciinema ffmpeg python3 util-linux
    elif command -v zypper >/dev/null 2>&1; then
      $sudo_cmd zypper --non-interactive install asciinema ffmpeg python3 util-linux
    elif command -v apk >/dev/null 2>&1; then
      $sudo_cmd apk add --no-cache asciinema ffmpeg python3 util-linux bash
    elif command -v xbps-install >/dev/null 2>&1; then
      $sudo_cmd xbps-install -Sy asciinema ffmpeg python3 util-linux
    elif command -v emerge >/dev/null 2>&1; then
      $sudo_cmd emerge --noreplace app-misc/asciinema media-video/ffmpeg dev-lang/python sys-apps/util-linux
    else
      echo -e "${C_YELLOW}Warning: unknown package manager. Please install: asciinema ffmpeg python3 util-linux${C_RESET}"
    fi
  fi

  # Install agg (asciinema gif generator) if not already installed
  if ! command -v agg >/dev/null 2>&1; then
    local arch agg_asset
    arch="$(uname -m)"
    case "$arch" in
      x86_64|amd64)  agg_asset="agg-x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) agg_asset="agg-aarch64-unknown-linux-gnu" ;;
      *) agg_asset="" ;;
    esac
    if [[ -n "$agg_asset" ]] && command -v curl >/dev/null 2>&1; then
      echo -e "${C_CYAN}Installing agg (${agg_asset})...${C_RESET}"
      mkdir -p "$PREFIX/bin"
      if curl -fsSL "https://github.com/asciinema/agg/releases/latest/download/${agg_asset}" -o "$PREFIX/bin/agg"; then
        chmod +x "$PREFIX/bin/agg"
        echo -e "${C_GREEN}✓ Installed agg to ${PREFIX}/bin/agg${C_RESET}"
      else
        echo -e "${C_YELLOW}Warning: could not download agg automatically. Install from https://github.com/asciinema/agg/releases${C_RESET}"
      fi
    else
      echo -e "${C_YELLOW}Note: please install 'agg' (https://github.com/asciinema/agg/releases) for MP4 rendering.${C_RESET}"
    fi
  fi
}

install_files() {
  local bin_dir="$PREFIX/bin"
  local share_dir="$PREFIX/share/rec-cli"
  local apps_dir="$PREFIX/share/applications"
  local icons_dir="$PREFIX/share/icons/hicolor/scalable/apps"
  mkdir -p "$bin_dir" "$share_dir/shell" "$share_dir/starship" "$apps_dir" "$icons_dir"

  install -m 755 "$REPO_ROOT/bin/rec-cli"   "$bin_dir/rec-cli"
  install -m 755 "$REPO_ROOT/bin/rec-pty"   "$bin_dir/rec-pty"
  install -m 755 "$REPO_ROOT/bin/rec-gui"   "$bin_dir/rec-gui"
  install -m 755 "$REPO_ROOT/bin/cast2mp4"  "$bin_dir/cast2mp4"
  install -m 755 "$REPO_ROOT/bin/casts2mp4" "$bin_dir/casts2mp4"

  install -m 644 "$REPO_ROOT/shell/rec.zsh"  "$share_dir/shell/rec.zsh"
  install -m 644 "$REPO_ROOT/shell/rec.bash" "$share_dir/shell/rec.bash"
  install -m 644 "$REPO_ROOT/shell/rec.fish" "$share_dir/shell/rec.fish"
  install -m 644 "$REPO_ROOT/starship/rec-module.toml" "$share_dir/starship/rec-module.toml"
  install -m 644 "$REPO_ROOT/packaging/desktop/rec-cli.desktop" "$apps_dir/rec-cli.desktop"
  install -m 644 "$REPO_ROOT/assets/logo.svg" "$icons_dir/rec-cli.svg"

  echo -e "${C_GREEN}✓ Installed binaries to ${bin_dir}${C_RESET}"
  echo -e "${C_GREEN}✓ Installed shell integrations to ${share_dir}/shell${C_RESET}"
}

add_line_once() {
  local file="$1" line="$2"
  mkdir -p "$(dirname "$file")"
  touch "$file"
  if ! grep -Fq "$line" "$file"; then
    printf '\n# rec-cli terminal clip recorder\n%s\n' "$line" >> "$file"
    echo -e "${C_GREEN}✓ Added rec-cli hook to ${file}${C_RESET}"
  else
    echo -e "${C_CYAN}• Hook already present in ${file}${C_RESET}"
  fi
}

configure_shells() {
  local shells=()
  case "$TARGET_SHELL" in
    auto)
      [[ -f "$HOME/.zshrc" || "$(basename "${SHELL:-}")" == "zsh" ]] && shells+=(zsh)
      [[ -f "$HOME/.bashrc" || "$(basename "${SHELL:-}")" == "bash" ]] && shells+=(bash)
      [[ -d "$HOME/.config/fish" || "$(basename "${SHELL:-}")" == "fish" ]] && shells+=(fish)
      ;;
    all) shells=(zsh bash fish) ;;
    zsh|bash|fish) shells=("$TARGET_SHELL") ;;
    *) echo "Unknown shell: $TARGET_SHELL" >&2; exit 1 ;;
  esac

  for sh in "${shells[@]}"; do
    case "$sh" in
      zsh)  add_line_once "$HOME/.zshrc" 'eval "$(rec-cli init zsh)"' ;;
      bash) add_line_once "$HOME/.bashrc" 'eval "$(rec-cli init bash)"' ;;
      fish) add_line_once "$HOME/.config/fish/config.fish" 'rec-cli init fish | source' ;;
    esac
  done
}

[[ "$INSTALL_DEPS" -eq 1 ]] && install_deps
install_files
[[ "$CONFIGURE_SHELLS" -eq 1 ]] && configure_shells

echo ""
echo -e "${C_GREEN}${C_BOLD}rec-cli installed successfully!${C_RESET}"
echo -e "Make sure ${C_CYAN}${PREFIX}/bin${C_RESET} is in your PATH, then open a new shell and run: ${C_BOLD}rec doctor${C_RESET}"
