#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
# A fresh staging directory prevents bundling a user's oracle.env by accident.
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT INT TERM
BUNDLE="$STAGE/riddle-rm2"
mkdir "$BUNDLE"
cp target/armv7-unknown-linux-musleabihf/release/riddle "$BUNDLE/riddle"
cp scripts/rm2/run.sh scripts/rm2/launch.sh scripts/rm2/recover.sh scripts/rm2/oracle.env.example "$BUNDLE/"
cp LICENSE "$BUNDLE/LICENSE-riddle"
cp docs/RM2.md "$BUNDLE/README.md"
curl --fail --location --retry 2 --output "$BUNDLE/librm2fb_server.so.1.0.1" \
    https://github.com/ddvk/remarkable2-framebuffer/releases/download/v0.0.33/librm2fb_server.so.1.0.1
printf '%s  %s\n' 3ec07729a6bb36c57648492e6d9b5c7abd4eb6c01c4c6079f70fd28c7eab4473 "$BUNDLE/librm2fb_server.so.1.0.1" | sha256sum -c -
cp docs/LICENSE-rm2fb "$BUNDLE/LICENSE-rm2fb"
chmod 755 "$BUNDLE/riddle" "$BUNDLE/run.sh" "$BUNDLE/launch.sh" "$BUNDLE/recover.sh"
(cd "$BUNDLE" && sha256sum riddle librm2fb_server.so.1.0.1 run.sh launch.sh recover.sh > SHA256SUMS)
mkdir -p dist
tar -czf dist/riddle-rm2-3.0.4.1305.tar.gz -C "$STAGE" riddle-rm2
(cd dist && sha256sum riddle-rm2-3.0.4.1305.tar.gz > riddle-rm2-3.0.4.1305.tar.gz.sha256)
