#!/usr/bin/env bash
set -euo pipefail
umask 022

# Build a Debian/Ubuntu/Kali/Mint/Pop!_OS .deb package for rec-cli

VERSION="${VERSION:-2.1.0}"
ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

GUI_BIN="$REPO_ROOT/gui/target/release/rec-gui"
if [[ ! -x "$GUI_BIN" ]]; then
  CARGO_BIN="$(command -v cargo 2>/dev/null || echo "$HOME/.cargo/bin/cargo")"
  "$CARGO_BIN" build --release --manifest-path "$REPO_ROOT/gui/Cargo.toml"
fi

PKG_ROOT="$STAGE/rec-cli_${VERSION}_${ARCH}"
mkdir -p "$PKG_ROOT/DEBIAN" \
         "$PKG_ROOT/usr/bin" \
         "$PKG_ROOT/usr/share/rec-cli/shell" \
         "$PKG_ROOT/usr/share/rec-cli/starship" \
         "$PKG_ROOT/usr/share/applications" \
         "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps" \
         "$PKG_ROOT/usr/share/doc/rec-cli"

install -m 755 "$REPO_ROOT/bin/rec-cli"   "$PKG_ROOT/usr/bin/rec-cli"
install -m 755 "$REPO_ROOT/bin/rec-pty"   "$PKG_ROOT/usr/bin/rec-pty"
install -m 755 "$GUI_BIN"                 "$PKG_ROOT/usr/bin/rec-gui"
install -m 755 "$REPO_ROOT/bin/cast2mp4"  "$PKG_ROOT/usr/bin/cast2mp4"
install -m 755 "$REPO_ROOT/bin/casts2mp4" "$PKG_ROOT/usr/bin/casts2mp4"

install -m 644 "$REPO_ROOT/shell/rec.zsh"  "$PKG_ROOT/usr/share/rec-cli/shell/rec.zsh"
install -m 644 "$REPO_ROOT/shell/rec.bash" "$PKG_ROOT/usr/share/rec-cli/shell/rec.bash"
install -m 644 "$REPO_ROOT/shell/rec.fish" "$PKG_ROOT/usr/share/rec-cli/shell/rec.fish"
install -m 644 "$REPO_ROOT/starship/rec-module.toml" "$PKG_ROOT/usr/share/rec-cli/starship/rec-module.toml"
install -m 644 "$REPO_ROOT/packaging/desktop/rec-cli.desktop" "$PKG_ROOT/usr/share/applications/rec-cli.desktop"
install -m 644 "$REPO_ROOT/assets/logo.svg" "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps/rec-cli.svg"
install -m 644 "$REPO_ROOT/README.md" "$PKG_ROOT/usr/share/doc/rec-cli/README.md"
install -m 644 "$REPO_ROOT/LICENSE"   "$PKG_ROOT/usr/share/doc/rec-cli/copyright"

cat > "$PKG_ROOT/DEBIAN/control" <<EOF
Package: rec-cli
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: bash, python3, asciinema, ffmpeg, util-linux
Recommends: zsh | fish, python3-pil
Maintainer: 0xEleven <0xEleven@users.noreply.github.com>
Homepage: https://github.com/0xEleven/rec-cli
Description: Terminal clip recorder and native Rust desktop app for Linux
 Record crisp terminal demos, tutorials, and proof-of-concepts in Zsh, Bash,
 and Fish. Includes a fast native Rust GUI (rec-gui) to start, pause, snapshot,
 bookmark, stop, and organize your clips library with thumbnails, GIF export,
 and lossless trimming.
EOF

cat > "$PKG_ROOT/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
echo ""
echo "rec-cli installed! Enable it in your shell:"
echo '  Zsh  (~/.zshrc):                  eval "$(rec-cli init zsh)"'
echo '  Bash (~/.bashrc):                 eval "$(rec-cli init bash)"'
echo '  Fish (~/.config/fish/config.fish): rec-cli init fish | source'
echo ""
echo "Then launch the desktop app with: rec gui"
echo ""
EOF
chmod 755 "$PKG_ROOT/DEBIAN" "$PKG_ROOT/DEBIAN/postinst"
chmod 644 "$PKG_ROOT/DEBIAN/control"

mkdir -p "$DIST_DIR"
OUT_DEB="$DIST_DIR/rec-cli_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$PKG_ROOT" "$OUT_DEB"
echo "Built: $OUT_DEB"
