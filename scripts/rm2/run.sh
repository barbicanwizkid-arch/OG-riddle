#!/bin/sh
# Runs only inside a transient systemd service; launch.sh creates that service.
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$HERE"
fail() { echo "riddle-rm2: $*" >&2; exit 1; }
[ "$(id -u)" = 0 ] || fail 'Run on the tablet as root.'
[ "$(cat /sys/devices/soc0/machine)" = 'reMarkable 2.0' ] || fail 'This bundle is for reMarkable 2 only.'
VERSION=$(sed -n 's/^REMARKABLE_RELEASE_VERSION=//p' /usr/share/remarkable/update.conf | tr -d '"\r')
[ "$VERSION" = 3.0.4.1305 ] || fail "Firmware $VERSION is outside this experimental build's target (3.0.4.1305)."
[ -x "$HERE/riddle" ] || fail 'Missing executable riddle.'
[ -f "$HERE/librm2fb_server.so.1.0.1" ] || fail 'Missing bundled rm2fb server.'
[ -f "$HERE/SHA256SUMS" ] || fail 'Missing bundle checksums.'
sha256sum -c SHA256SUMS >/dev/null || fail 'Bundle checksum verification failed.'
[ -f "$HERE/oracle.env" ] || fail 'Create oracle.env from oracle.env.example and enter your API key.'
# User-owned shell configuration. Never print its contents or enable shell tracing.
chmod 600 "$HERE/oracle.env"
set -a
. "$HERE/oracle.env"
set +a
[ -n "${RIDDLE_OPENAI_KEY:-}" ] && [ "$RIDDLE_OPENAI_KEY" != REPLACE_WITH_YOUR_API_KEY ] || fail 'Set RIDDLE_OPENAI_KEY in oracle.env.'
pgrep -x rm2fb-server >/dev/null 2>&1 && fail 'An rm2fb server is already running; stop the other launcher first.'
[ "${1:-}" = --check ] && { echo 'Device, firmware, bundle, and API-key presence checks passed.'; exit 0; }
[ "${INVOCATION_ID:-}" ] || fail 'Use launch.sh so systemd can restore the normal interface even after a crash.'
# Exclusive ownership prevents two diary processes from sharing the display.
exec 9>/run/riddle-rm2.lock
flock -n 9 || fail 'Another diary session owns the display.'
systemctl is-active --quiet xochitl || fail 'The normal interface is not active; refusing to take over an unknown session.'
server_pid=
app_pid=
watcher_pid=
restore_needed=0
cleanup() {
    trap - EXIT INT TERM HUP
    if [ -n "$watcher_pid" ]; then kill "$watcher_pid" 2>/dev/null || true; wait "$watcher_pid" 2>/dev/null || true; fi
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; wait "$app_pid" 2>/dev/null || true; fi
    if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
    if [ "$restore_needed" = 1 ]; then
        if systemctl start xochitl; then rm -f /run/riddle-rm2.restore; fi
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
printf '%s\n' "$INVOCATION_ID" > /run/riddle-rm2.restore
restore_needed=1
systemctl stop xochitl
LD_PRELOAD="$HERE/librm2fb_server.so.1.0.1" /usr/bin/xochitl &
server_pid=$!
ready=0
for attempt in 1 2 3 4 5; do
    kill -0 "$server_pid" 2>/dev/null || fail 'rm2fb server exited. See journalctl -u riddle-rm2.'
    if "$HERE/riddle" --rm2-check; then ready=1; break; fi
    sleep 1
done
[ "$ready" = 1 ] || fail 'rm2fb did not become ready.'
"$HERE/riddle" &
app_pid=$!
# A server crash must restore the interface even when the diary is idle.
(
    while kill -0 "$app_pid" 2>/dev/null; do
        if ! kill -0 "$server_pid" 2>/dev/null; then
            echo "riddle-rm2: display server died; stopping the diary" >&2
            kill "$app_pid" 2>/dev/null || true
            exit 1
        fi
        sleep 1
    done
) &
watcher_pid=$!
status=0
wait "$app_pid" || status=$?
app_pid=
exit "$status"
