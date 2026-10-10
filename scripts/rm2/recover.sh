#!/bin/sh
# systemd runs this after killing the session's remaining processes.
# A failed preflight must never start xochitl over another launcher's session.
set -eu
MARKER=/run/riddle-rm2.restore
[ -n "${INVOCATION_ID:-}" ] || exit 0
[ -f "$MARKER" ] || exit 0
[ "$(cat "$MARKER")" = "$INVOCATION_ID" ] || exit 0
systemctl --no-block start xochitl
rm -f "$MARKER"
