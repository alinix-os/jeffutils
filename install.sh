#!/usr/bin/env bash
set -euo pipefail
STAGE="$1"

# Try compiling from source first to ensure latest changes are included
if command -v cargo >/dev/null 2>&1; then
    echo "  Compiling JeffUtils from source..."
    cargo build --release --workspace
    
    # Install binaries to /opt/jeffutils
    mkdir -p "$STAGE/opt/jeffutils"
    for proj_dir in */; do
        [ -f "$proj_dir/Cargo.toml" ] || continue
        proj=$(basename "$proj_dir")
        if [ -f "target/release/$proj" ]; then
            cp "target/release/$proj" "$STAGE/opt/jeffutils/"
        fi
    done
    
    # Path configuration
    mkdir -p "$STAGE/etc/profile.d"
    cat << 'EOF' > "$STAGE/etc/profile.d/jeffutils.sh"
# Prepend /opt/jeffutils to PATH so it takes priority
if [ -d "/opt/jeffutils" ]; then
  export PATH="/opt/jeffutils:$PATH"
fi
EOF
    chmod 644 "$STAGE/etc/profile.d/jeffutils.sh"
    
    # Font installation
    if [ -d "jsh/assets/font" ]; then
        mkdir -p "$STAGE/usr/share/fonts/jsh-mono"
        cp jsh/assets/font/*.ttf "$STAGE/usr/share/fonts/jsh-mono/"
    fi
elif [ -f dist/jeffutils_1.0.0_amd64.deb ]; then
    echo "  Unpacking pre-built jeffutils_1.0.0_amd64.deb..."
    dpkg-deb -x dist/jeffutils_1.0.0_amd64.deb "$STAGE"
elif [ -f dist/jeffutils_0.1.0_amd64.deb ]; then
    echo "  Unpacking pre-built jeffutils_0.1.0_amd64.deb..."
    dpkg-deb -x dist/jeffutils_0.1.0_amd64.deb "$STAGE"
else
    echo "Error: cargo not found and no pre-built package available to extract." >&2
    exit 1
fi
