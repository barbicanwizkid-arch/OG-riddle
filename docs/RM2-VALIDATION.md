# Experimental reMarkable 2 validation

Target: stock reMarkable 2 firmware 3.0.4.1305. Baseline: `0d1a9feea75027543e2ed49c721656cd261f03a1`. Branch: `codex/remarkable-2-port`.

## Executed locally

- Rust 1.90.0, locked dependencies: default build tests, **29 passed**.
- Host `rm2` feature tests: **31 passed**.
- ARMv7 musl release cross-build: **passed**; ELF is 32-bit ARM EABI5, statically linked.
- Release executable `--version` under QEMU ARM 7.2.0: **passed**.
- ARMv7 musl tests under QEMU: **31 passed** (including native message queue layout, bounded queue-full timeout, coordinate transforms, help rendering, and upstream parser/memory checks).
- Launcher tests with simulated hardware/services: **12 passed**, including wrong firmware/model, missing key, checksum failure, competing server, inactive interface, normal exit, abnormal app exit, idle server crash, termination, and recovery-marker ownership.
- Shell syntax and `git diff --check`: **passed**.

These tests use fake hardware/services or emulation. They do not test the physical e-ink panel, digitizer, tablet's Qt/display stack, or live provider credentials. Unused-code warnings remain in upstream/shared modules.

## Independent review

A separate reviewer inspected the changed Rust and launcher code against the upstream rm2fb/libremarkable interfaces. Two findings were fixed:

| Finding | Change | Verification |
| --- | --- | --- |
| Server death while the diary is idle leaves normal UI stopped | Watchdog terminates diary when its display-server child dies | Idle-crash recovery test; reviewer reran it |
| Unconditional stop hook could restart the UI over another launcher's session after failed preflight | Invocation-bound restoration marker; restore only if this service owns takeover | Missing/stale/matching marker test; source re-review |

Reviewer found no remaining material source-review blockers. Hardware acceptance remains open; do not equate this with production/device certification.

## Remaining gate

Follow the tablet acceptance checks in [RM2.md](RM2.md), starting with non-sensitive handwriting. GitHub Actions separately builds and packages the pushed commit; inspect its result before downloading its artifact. No code has been installed on the user's tablet by this work.
