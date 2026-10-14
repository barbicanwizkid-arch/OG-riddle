#!/bin/bash
# Mac helper for the unchanged, pinned experimental build. Never run with sudo.
set -euo pipefail
umask 077
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TARGET=root@10.11.99.1
REMOTE=/home/root/riddle-rm2-097ccb96
ARCHIVE=riddle-rm2-3.0.4.1305.tar.gz
HASH=d02d11e16c25c834436193c39e7e26575ee54acd24c62246bfceed4e303d2370
SSH=(ssh -o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3)
fail() { echo "STOP: $*" >&2; exit 1; }
confirm() {
    local answer
    printf '%s Type YES: ' "$1" >/dev/tty
    read -r answer </dev/tty
    [ "$answer" = YES ] || fail 'No change made by this step.'
}
verify() {
    cd "$HERE/payload"
    printf '%s  %s\n' "$HASH" "$ARCHIVE" | shasum -a 256 -c -
    printf '%s  %s\n' 8e7a4d729bd1a3922b5150e988a3f0eec08abf51ea4b2e6ccf446457f93114c9 riddle-rm2-original.zip | shasum -a 256 -c -
    shasum -a 256 -c "$ARCHIVE.sha256"
}
preflight() {
    "${SSH[@]}" "$TARGET" 'sh -s' <<'REMOTE_CHECK'
set -eu
fail() { echo "STOP: $*" >&2; exit 1; }
[ "$(cat /sys/devices/soc0/machine)" = 'reMarkable 2.0' ] || fail 'Wrong model.'
version=$(sed -n 's/^REMARKABLE_RELEASE_VERSION=//p' /usr/share/remarkable/update.conf | tr -d '"\r')
[ "$version" = 3.0.4.1305 ] || fail "Firmware is $version; do not downgrade or proceed."
[ "$(uname -m)" = armv7l ] || fail 'Unexpected processor architecture.'
for tool in systemctl systemd-run sha256sum flock pgrep tar df du awk; do
    command -v "$tool" >/dev/null || fail "Missing command: $tool"
done
systemctl is-active --quiet xochitl || fail 'Normal notebook interface is not active.'
if systemctl is-active --quiet riddle-rm2; then fail 'Riddle is already active.'; fi
server_status=0
pgrep -x rm2fb-server >/dev/null || server_status=$?
case "$server_status" in
    0) fail 'Another display server is running.' ;;
    1) ;;
    *) fail 'Cannot determine competing display-server state.' ;;
esac
systemd-run --help | grep -q -- '--collect' || fail 'systemd-run lacks --collect.'
[ -d /home/root/.local/share/remarkable/xochitl ] || fail 'Notebook directory not found.'
[ ! -L /home/root/.local/share/remarkable/xochitl ] || fail 'Notebook directory is a symlink; review before backup.'
free=$(df -Pk /home/root | awk 'END {print $4}')
[ "$free" -ge 51200 ] || fail 'Need at least 50 MiB free for staging and runtime headroom.'
echo 'PASS: model, exact firmware, ARMv7, commands, normal UI, and free space.'
echo 'Stock/unpatched xochitl still requires your confirmation; version text alone cannot prove it.'
printf 'Notebook directory size (KiB): '
du -sk /home/root/.local/share/remarkable/xochitl | awk '{print $1}'
printf 'Device UTC clock: '; date -u '+%Y-%m-%d %H:%M:%S UTC'
REMOTE_CHECK
}
case "${1:-help}" in
verify) verify ;;
preflight) verify; preflight ;;
backup)
    preflight
    confirm 'Close notebooks, let sync finish, and leave the tablet idle. This creates a supplementary live copy, not a guaranteed consistent snapshot.'
    dest="$HOME/Documents/OG-riddle-private-backups/$(date -u +%Y%m%dT%H%M%SZ)"
    mkdir -p "$dest"
    chmod 700 "$HOME/Documents/OG-riddle-private-backups" "$dest"
    echo "Copying notebooks privately to $dest. This can take several minutes."
    if ! "${SSH[@]}" "$TARGET" 'tar -czf - -C /home/root/.local/share/remarkable xochitl' > "$dest/notebooks.tar.gz.partial"; then
        fail 'Backup failed or files changed. Partial copy retained; do not treat it as a backup.'
    fi
    tar -tzf "$dest/notebooks.tar.gz.partial" >/dev/null
    mv "$dest/notebooks.tar.gz.partial" "$dest/notebooks.tar.gz"
    (cd "$dest" && shasum -a 256 notebooks.tar.gz > notebooks.tar.gz.sha256)
    echo 'Archive structure verified. Keep private; do not upload it. Export/open important notebooks separately before installation.'
    ;;
install)
    verify; preflight
    confirm 'Confirm stock/unpatched firmware AND completed sync plus usable exports/backups of important notebooks. Installation only; no launch.'
    stage=$(mktemp -d "${TMPDIR:-/tmp}/riddle-stage.XXXXXX")
    trap 'rm -rf "$stage"' EXIT
    tar -xzf "$HERE/payload/$ARCHIVE" -C "$stage"
    (cd "$stage/riddle-rm2" && shasum -a 256 -c SHA256SUMS)
    # mkdir fails if this build is already staged. Never overwrite an install.
    "${SSH[@]}" "$TARGET" "umask 077; mkdir '$REMOTE'"
    scp -O -o ConnectTimeout=10 -r "$stage/riddle-rm2" "$TARGET:$REMOTE/"
    "${SSH[@]}" "$TARGET" "set -eu; cd '$REMOTE/riddle-rm2'; sha256sum -c SHA256SUMS; chmod 755 riddle run.sh launch.sh recover.sh"
    echo 'Installed and checked; normal interface has not been stopped. Next: bash mac.sh configure'
    ;;
configure)
    # Credentials are entered only in the tablet editor, never in arguments or logs.
    "${SSH[@]}" -t "$TARGET" "set -eu; umask 077; cd '$REMOTE/riddle-rm2'; if [ ! -e oracle.env ]; then cp oracle.env.example oracle.env; fi; chmod 600 oracle.env; vi oracle.env"
    ;;
check)
    preflight
    "${SSH[@]}" "$TARGET" "cd '$REMOTE/riddle-rm2' && ./run.sh --check"
    ;;
launch)
    preflight
    confirm 'Keep USB attached and a second Terminal ready for recovery. Only non-sensitive test handwriting; this starts screen takeover.'
    "${SSH[@]}" "$TARGET" "cd '$REMOTE/riddle-rm2' && ./launch.sh"
    echo 'Launch requested, not yet hardware-verified. Press power to exit; follow START-HERE.md.'
    ;;
status)
    "${SSH[@]}" "$TARGET" 'systemctl show riddle-rm2 xochitl -p Id -p LoadState -p ActiveState -p SubState -p Result'
    ;;
stop)
    "${SSH[@]}" "$TARGET" 'systemctl stop riddle-rm2'
    echo 'Stop requested. Check tablet visually and run: bash mac.sh status'
    ;;
recover)
    confirm 'Use only for this Riddle test session, with no other launcher running. Restore the notebook interface?'
    "${SSH[@]}" "$TARGET" 'sh -s' <<'RECOVER'
set -eu
systemctl stop riddle-rm2 || true
state=$(systemctl show riddle-rm2 -p LoadState -p ActiveState) || {
    echo 'STOP: Cannot determine Riddle state; refusing overlapping display ownership.' >&2
    exit 1
}
case "$state" in
    *LoadState=not-found*) ;;
    *ActiveState=inactive*|*ActiveState=failed*) ;;
    *) echo 'STOP: Riddle is still active/transitional or unknown; do not start another display owner.' >&2; exit 1 ;;
esac
server_status=0
pgrep -x rm2fb-server >/dev/null || server_status=$?
case "$server_status" in
    0) echo 'STOP: A display server remains; investigate before restarting xochitl.' >&2; exit 1 ;;
    1) ;;
    *) echo 'STOP: Cannot determine competing display-server state.' >&2; exit 1 ;;
esac
systemctl start xochitl
systemctl is-active xochitl
RECOVER
    ;;
crash-test)
    confirm 'Only after ordinary power/stop recovery passes: deliberately kill the Riddle test service and check automatic restoration?'
    "${SSH[@]}" "$TARGET" 'systemctl is-active --quiet riddle-rm2 && systemctl kill --kill-whom=all --signal=KILL riddle-rm2'
    echo 'Kill sent. Wait up to 20 seconds; inspect tablet and run: bash mac.sh status. Use recover if needed.'
    ;;
*) echo 'Usage: bash mac.sh {verify|preflight|backup|install|configure|check|launch|status|stop|recover|crash-test}' ;;
esac
