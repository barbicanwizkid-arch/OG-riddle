#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
"$HERE/run.sh" --check
# An existing service or a competing launch must not replace the active session.
if systemctl is-active --quiet riddle-rm2; then
    echo 'The diary is already running.'
    exit 0
fi
systemctl reset-failed riddle-rm2 2>/dev/null || true
# No fallback without ExecStopPost: recovery is mandatory. No boot-time changes.
# --no-block avoids waiting for xochitl's restart from inside this unit's cleanup.
exec systemd-run --unit=riddle-rm2 --collect \
    --property=Type=exec \
    --property=KillMode=control-group \
    --property=TimeoutStopSec=10 \
    --property="ExecStopPost=/bin/sh \"$HERE/recover.sh\"" \
    /bin/sh "$HERE/run.sh"
