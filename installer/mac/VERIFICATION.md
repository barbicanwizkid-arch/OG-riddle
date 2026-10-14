# Preparation verification — October 14, 2026

## Provenance

- Repository: barbicanwizkid-arch/OG-riddle.
- Original draft PR: #1, open and unmerged at inspection.
- Source/base for this kit: `097ccb96eedde986f19ece1ebb2b3b0b8d881565`.
- Original PR base: `0d1a9feea75027543e2ed49c721656cd261f03a1`.
- Successful Actions run: `38033938534`; conclusion `success`.
- Artifact: `11663168560`, `riddle-rm2-3.0.4.1305-experimental`.
- Artifact size: 1,280,104 bytes. Reported expiry: January 8, 2027, 07:17:49 UTC.
- Artifact and run head both match the source commit above.
- Original payload is unchanged. The Mac helper is preparation code outside that
  build; its review and hardware execution are separate from original CI evidence.

## SHA-256 values independently calculated from downloaded bytes

| File | SHA-256 |
|---|---|
| Original artifact ZIP | `8e7a4d729bd1a3922b5150e988a3f0eec08abf51ea4b2e6ccf446457f93114c9` |
| riddle-rm2-3.0.4.1305.tar.gz | `d02d11e16c25c834436193c39e7e26575ee54acd24c62246bfceed4e303d2370` |
| Bundled rm2fb server | `3ec07729a6bb36c57648492e6d9b5c7abd4eb6c01c4c6079f70fd28c7eab4473` |

ZIP hash matches GitHub's artifact digest. TAR hash matches the artifact checksum
file. Server hash matches the source packaging pin. These verify byte integrity
and provenance linkage; they are not hardware certification or a code signature.

## Checks executed during preparation

- ZIP CRC and exact two-file inventory: PASS.
- TAR inventory: exactly one directory and ten regular files; no links,
  traversal names, or unexpected payloads: PASS.
- All five entries in bundled SHA256SUMS: PASS.
- Launcher scripts, example configuration, original guide, and license files
  byte-equal to the pinned source: PASS.
- Binary inspection: ELF32 little-endian ARM EABI5 hard-float, ARMv7,
  statically linked application. Separate display server is ARM shared ELF.
- Original launcher/recovery test suite rerun: **12/12 PASS** with simulated
  services; no live service operations.
- Mac helper shell syntax and offline pinned-hash verification: PASS on Linux
  Bash. Code uses Bash 3.2-compatible constructs; native macOS execution pending.
- Independent safety review required a guard against starting xochitl after an
  unsuccessful stop while Riddle remained active. Helper corrected; original
  payload remains unchanged. Guide supersedes the original short manual recovery.
- Recovery guard mock test details are recorded in the accompanying review file.

Original reported Rust test evidence (29 default host, 31 rm2 host, 31 ARM-emulated)
is retained in docs/RM2-VALIDATION.md and linked successful CI; those Rust suites
were not repeated during this packaging task. No QEMU execution was performed here.

## Remaining acceptance boundary

No Mac/tablet connection, installation, API request, key handling, or physical
hardware test was performed during preparation. All device rows remain pending
in ACCEPTANCE.md. Backup consistency/restore, stock xochitl provenance, pen
alignment, refresh quality, power behavior, service recovery, and a real provider
response require actual-device observations. No firmware modification, automatic
startup, PR merge, or credentials are included.

The current page is temporarily written to /tmp/riddle-page.png; normal flow
removes it after passing it to the provider worker unless RIDDLE_KEEP_PAGE is set.
Crash interruption may leave it behind. Provider-error bodies can contain private
data or credential fragments. Do not share raw logs or screenshots of errors.

The added font OFL license accompanies the font embedded in the existing binary.
