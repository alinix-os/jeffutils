#!/usr/bin/env bash
# Builds a .deb or .rpm package for JeffUtils: every utility binary in the
# monorepo (built via `make build`) plus the jsh custom font (assets/font/).
#
# Usage: packaging/build-pkg.sh <deb|rpm> [version]
#
# Requires: fpm (https://fpm.readthedocs.io), and either dpkg-deb (for deb)
# or rpmbuild (for rpm) on PATH.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REL_DIR="${REL_DIR:-target/release}"
FORMAT="${1:?usage: build-pkg.sh <deb|rpm> [version]}"
VERSION="${2:-$(grep -m1 '^version' "$ROOT_DIR/jsh/Cargo.toml" | sed -E 's/version = "(.*)"/\1/')}"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) FPM_ARCH="amd64" ;;
  aarch64|arm64) FPM_ARCH="arm64" ;;
  *) FPM_ARCH="$ARCH" ;;
esac

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

echo ">> Staging package contents in $STAGE"

# --- binaries: every utility already built by `make build` ---
mkdir -p "$STAGE/opt/jutils"
found_any=0
for proj_dir in "$ROOT_DIR"/*/; do
  proj="$(basename "$proj_dir")"
  [ -f "$proj_dir/Cargo.toml" ] || continue
  bin="$ROOT_DIR/$REL_DIR/$proj"
  if [ -f "$bin" ]; then
    cp "$bin" "$STAGE/opt/jutils/$proj"
    found_any=1
  fi
done
if [ "$found_any" -eq 0 ]; then
  echo "!! No built binaries found under */target/release/. Run 'make build' first." >&2
  exit 1
fi

# --- PATH configuration ---
mkdir -p "$STAGE/etc/profile.d"
cat << 'EOF' > "$STAGE/etc/profile.d/jeffutils.sh"
# Prepend /opt/jutils to PATH so it takes priority
if [ -d "/opt/jutils" ]; then
  export PATH="/opt/jutils:$PATH"
fi
EOF
chmod 644 "$STAGE/etc/profile.d/jeffutils.sh"

# --- jsh custom font ---
FONT_DIR="$ROOT_DIR/jsh/assets/font"
if [ -d "$FONT_DIR" ]; then
  mkdir -p "$STAGE/usr/share/fonts/jsh-mono"
  cp "$FONT_DIR"/*.ttf "$STAGE/usr/share/fonts/jsh-mono/"
fi

# --- directories to package ---
PACKAGED_DIRS=()
[ -d "$STAGE/opt" ] && PACKAGED_DIRS+=("opt")
[ -d "$STAGE/etc" ] && PACKAGED_DIRS+=("etc")
[ -d "$STAGE/usr" ] && PACKAGED_DIRS+=("usr")

echo ">> Building .$FORMAT package (version $VERSION, arch $FPM_ARCH)"

fpm -s dir -t "$FORMAT" -f \
  -n jeffutils \
  -v "$VERSION" \
  -a "$FPM_ARCH" \
  --license MIT \
  --maintainer "Jefferson" \
  --description "JeffUtils - suite of JeffNix command-line utilities, including the jsh shell and its bundled JSH Mono font." \
  --url "https://github.com/JeffNix/jeffutils" \
  --after-install "$ROOT_DIR/packaging/post-install.sh" \
  --package "$ROOT_DIR/dist/" \
  -C "$STAGE" \
  "${PACKAGED_DIRS[@]}"

echo ">> Done. Package written to $ROOT_DIR/dist/"
