#!/usr/bin/env bash
set -euo pipefail

# Clean uninstaller for rec-cli

PREFIX="${PREFIX:-$HOME/.local}"
[[ "${1:-}" == "--system" ]] && PREFIX="/usr/local"

rm -f "$PREFIX/bin/rec-cli" "$PREFIX/bin/rec-pty" "$PREFIX/bin/rec-gui" "$PREFIX/bin/rec-studio-engine" "$PREFIX/bin/rec-studio-editor" "$PREFIX/bin/rec-capture" "$PREFIX/bin/cast2mp4" "$PREFIX/bin/casts2mp4"
rm -f "$PREFIX/share/applications/rec-cli.desktop" "$PREFIX/share/icons/hicolor/scalable/apps/rec-cli.svg"
rm -rf "$PREFIX/share/rec-cli"

remove_hook() {
  local file="$1" pattern="$2"
  [[ -f "$file" ]] || return 0
  if grep -Fq "$pattern" "$file"; then
    local tmp
    tmp="$(mktemp)"
    grep -Fv "$pattern" "$file" | grep -Fv "# rec-cli terminal clip recorder" > "$tmp" || true
    mv -f "$tmp" "$file"
    echo "Removed hook from $file"
  fi
}

remove_hook "$HOME/.zshrc" 'eval "$(rec-cli init zsh)"'
remove_hook "$HOME/.bashrc" 'eval "$(rec-cli init bash)"'
remove_hook "$HOME/.config/fish/config.fish" 'rec-cli init fish | source'

echo "rec-cli uninstalled from $PREFIX. Your saved videos in ~/Videos/content were NOT touched."
