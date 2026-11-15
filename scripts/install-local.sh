#!/bin/sh
# Builds a release binary into ~/.local/bin (replacing any older one), then
# restarts the core the planned way so running terminals pick up the new build.
set -eu
cd "$(dirname "$0")/.."
nice -n 10 cargo build --release
bin="${LBC_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$bin"
if [ "$(uname -s)" = Darwin ]; then
  app="target/release/Lumbridge Core.app"
  rm -rf "$app"
  sh scripts/mac-bundle target/release/lumbridge-core "$app"
  bash scripts/mac-sign "$app"
  "$app/Contents/MacOS/lumbridge-core" --lbc-install-bundle "$app/Contents/MacOS/lumbridge-core" "$bin/lumbridge-core"
else
  install -m 755 target/release/lumbridge-core "$bin/lumbridge-core.new"
  mv -f "$bin/lumbridge-core.new" "$bin/lumbridge-core"
fi
[ -e "$bin/lbc" ] || [ -L "$bin/lbc" ] || ln -s lumbridge-core "$bin/lbc"
"$bin/lumbridge-core" server restart
