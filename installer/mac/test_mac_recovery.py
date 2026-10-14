#!/usr/bin/env python3
"""Execute the kit's recovery heredoc with isolated command-function mocks.

No SSH connection or real systemctl/pgrep operation is performed. This verifies
shell control flow, not tablet behavior or Bash 3.2/macOS compatibility.
"""
import os
from pathlib import Path
import subprocess
import sys

DEFAULT_HELPER = Path(__file__).resolve().parent / "mac.sh"
helper = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_HELPER
source = helper.read_text()
body = source.split("<<'RECOVER'\n", 1)[1].split("\nRECOVER\n", 1)[0]
prefix = r'''
systemctl() {
    case "$1" in
        stop) printf 'MOCK stop\n' >&2; return "$MOCK_STOP" ;;
        show) printf '%s\n' "$MOCK_STATE"; return "$MOCK_SHOW" ;;
        start) printf 'MOCK start\n' >&2; return 0 ;;
        is-active) printf 'active\n'; return 0 ;;
        *) return 99 ;;
    esac
}
pgrep() { return "$MOCK_PGREP"; }
'''
cases = []
for state in ("active", "activating", "deactivating", "reloading", "unknown", ""):
    cases.append((state or "empty", "LoadState=loaded\nActiveState=" + state, 0, 0, 1, False))
for state in ("inactive", "failed"):
    for stop in (0, 1):
        cases.append((state + " stop=" + str(stop), "LoadState=loaded\nActiveState=" + state, stop, 0, 1, True))
cases.extend([
    ("not-found after failed stop", "LoadState=not-found\nActiveState=inactive", 1, 0, 1, True),
    ("failed state query", "LoadState=loaded\nActiveState=inactive", 1, 1, 1, False),
    ("active after failed stop", "LoadState=loaded\nActiveState=active", 1, 0, 1, False),
    ("competing display server", "LoadState=loaded\nActiveState=inactive", 0, 0, 0, False),
    ("pgrep error 2", "LoadState=loaded\nActiveState=inactive", 0, 0, 2, False),
    ("pgrep unavailable 127", "LoadState=loaded\nActiveState=inactive", 0, 0, 127, False),
])
for name, state, stop, show, pgrep, allowed in cases:
    env = dict(os.environ, MOCK_STATE=state, MOCK_STOP=str(stop), MOCK_SHOW=str(show), MOCK_PGREP=str(pgrep))
    result = subprocess.run(["/bin/sh", "-c", prefix + body], env=env, capture_output=True, text=True)
    started = "MOCK start" in result.stderr
    passed = started == allowed and (result.returncode == 0) == allowed
    print(f"{'PASS' if passed else 'FAIL'}: {name}; exit={result.returncode}; starts_xochitl={started}")
    if not passed:
        print(result.stdout, result.stderr)
        raise SystemExit(1)
print(f"{len(cases)}/{len(cases)} recovery mock cases passed. No device or real service operations performed.")
