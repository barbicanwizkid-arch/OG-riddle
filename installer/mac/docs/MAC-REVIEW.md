# Independent Mac-helper review

Reviewed October 14, 2026. Scope: the Mac helper and installation/acceptance guide,
with the original tablet launchers and documentation as context. The original
build payload was not modified. No tablet was contacted.

## Result

No remaining material blockers were found within this source-review and mocked
recovery scope. This is not Mac compatibility certification or physical-device
acceptance. All tablet acceptance results remain pending.

Reviewed `mac.sh` SHA-256:

`bf7adf3bd200aff0378b65feb0810a775408e5707c8786ab2d61dc70c3f649f9`

Two recovery findings were corrected before this final review:

1. A failed stop could previously be followed by starting xochitl over an active
   Riddle session. Recovery now checks the unit state before restarting xochitl.
2. A failed process probe could previously count as no competing display server.
   Recovery and preflight now distinguish absence (exit 1) from errors (exit >1).

## Observed verification

- **16/16 recovery cases passed**, executing the actual `RECOVER` heredoc under
  `/bin/sh` with shell-function mocks for every systemctl and pgrep call.
- Active, activating, deactivating, reloading, unknown, and empty states blocked
  xochitl startup: 6 cases.
- Inactive and failed states permitted startup, each with successful and failed
  stop results: 4 cases.
- Absent unit after failed stop permitted startup: 1 case.
- Failed state query, active unit after failed stop, competing display server,
  pgrep error 2, and simulated command-unavailable status 127 blocked startup:
  5 cases.
- `bash -n mac.sh` passed under Bash 5.2.21. The source uses Bash 3.2-compatible
  constructs, but neither Bash 3.2 nor macOS was executed in this review.
- The helper's `verify` operation passed for the pinned original ZIP, TAR archive,
  and artifact-provided TAR checksum.

The reusable test is `review_tests/test_mac_recovery.py` in the preparation
workspace. It accepts an optional path to `mac.sh`, extracts the current recovery
heredoc, and never runs SSH or real service/process-management commands. Status
127 is simulated; this does not test an actual missing binary on a tablet.

## Safety and guide review

Installation creates an exclusive versioned directory, refuses overwrites,
checks transferred payload hashes, and does not stop the notebook interface.
The guide explains that interrupted staging is retained and must not be launched.
It distinguishes a live notebook copy from an atomic or proven-restorable backup
and requires usable exports of important notebooks before takeover.

The helper enters credentials in a tablet editor rather than shell arguments or
captured output. The guide warns that original provider errors may contain key
fragments and prohibits raw journals, config files, or page images in reports.
It explains memory-off temporary-image behavior and supersedes the original
unguarded short recovery instructions.

The acceptance sequence tests power exit before handwriting, then pen alignment,
display behavior, local help, a harmless AI reply, ordinary stop, crash recovery,
network failure, and notebook access. It does not enable startup, modify firmware,
or authorize merging the draft pull request.

## Limits

These checks do not establish USB connectivity, tablet SSH/tool compatibility,
actual systemd state transitions, rm2fb compatibility, backup restorability,
notebook integrity, pen alignment, display refresh, power-button behavior,
automatic recovery, or provider access. Mock results must not be entered as
hardware acceptance passes.
