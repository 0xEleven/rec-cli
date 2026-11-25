#!/usr/bin/env bash
set -euo pipefail
umask 022

# Build a Fedora/RHEL/Rocky/Alma/openSUSE .rpm package for rec-cli

VERSION="${VERSION:-1.3.0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
TOPDIR="$(mktemp -d)"
trap 'rm -rf "$TOPDIR"' EXIT

mkdir -p "$TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS,TMP,rpmdb}

SRC_STAGE="$TOPDIR/rec-cli-${VERSION}"
mkdir -p "$SRC_STAGE"
cp -r "$REPO_ROOT/bin" "$REPO_ROOT/shell" "$REPO_ROOT/starship" "$REPO_ROOT/README.md" "$REPO_ROOT/LICENSE" "$SRC_STAGE/"
tar -czf "$TOPDIR/SOURCES/rec-cli-${VERSION}.tar.gz" -C "$TOPDIR" "rec-cli-${VERSION}"

sed "s/^Version:.*/Version:        ${VERSION}/" "$SCRIPT_DIR/rec-cli.spec" > "$TOPDIR/SPECS/rec-cli.spec"

rpmbuild \
  --define "_topdir $TOPDIR" \
  --define "_tmppath $TOPDIR/TMP" \
  --define "_dbpath $TOPDIR/rpmdb" \
  --define "dist %{nil}" \
  --nodeps \
  -bb "$TOPDIR/SPECS/rec-cli.spec"

mkdir -p "$DIST_DIR"
cp "$TOPDIR"/RPMS/noarch/rec-cli-*.rpm "$DIST_DIR/"
echo "Built: $(ls "$DIST_DIR"/rec-cli-*.rpm)"
