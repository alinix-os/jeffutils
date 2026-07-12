#!/usr/bin/env bash
# Post-install hook for the jeffutils .deb/.rpm package: refreshes the
# fontconfig cache so the bundled JSH Mono font (installed to
# /usr/share/fonts/jsh-mono) is picked up immediately, without requiring
# the user to log out or run fc-cache manually.
set -e

if command -v fc-cache >/dev/null 2>&1; then
  fc-cache -f /usr/share/fonts/jsh-mono >/dev/null 2>&1 || true
fi

echo "jeffutils installed. Set your terminal font to 'JSH Mono' to see the penguin/apple/window prompt icons."
