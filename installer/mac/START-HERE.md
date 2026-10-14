# OG-riddle: Mac → reMarkable 2 acceptance kit

Prepared October 14, 2026. Experimental; physical-device acceptance is still pending.
Use only a stock reMarkable 2 on **3.0.4.1305**. Do not downgrade, update firmware,
enable developer mode, install an automatic launcher, or merge PR #1 for this test.

## 1. First step: verify and inspect without installing

Unzip this kit in Downloads on your Mac. Open Terminal and run:

```bash
cd ~/Downloads/OG-riddle-Mac-RM2
bash mac.sh verify
bash mac.sh preflight
```

If Finder placed the folder elsewhere, type `cd `, drag that folder into Terminal,
and press Return. No Homebrew, Rust, Python, or GitHub CLI is required.
Do not run with sudo or shell tracing (`bash -x`).

Connect the tablet using a USB data cable, wake/unlock it, and leave the normal
notebook interface open. Find its SSH password in Settings → Help → Copyrights
and licenses. Enter it only when SSH asks on your Mac; it will not echo.
The USB target is `root@10.11.99.1`. An initial SSH host-key prompt may appear;
proceed only when this is your connected tablet. If SSH reports a changed host
key, stop and investigate; do not disable host-key checking or delete known keys.

Preflight checks model, exact firmware, ARMv7, required tools, normal-interface
state, competing display-server state, and at least 50 MiB free on the tablet.
It prints notebook size and device UTC clock, but no notebook names or keys.
It does not write tablet files, stop services, or launch Riddle. Firmware text
cannot establish that xochitl is unpatched: confirm that separately. If uncertain,
stop before installation. Keep the tablet charged and USB attached throughout.

If preflight fails, stop and report its concise error (never passwords, keys, or
notebook contents). Do not bypass it. Check the clock against current UTC;
incorrect time can break HTTPS. This kit does not change the tablet clock.

## 2. Preserve notebooks before installation

Finish syncing. In the official reMarkable desktop app, confirm that important
notebooks are present and current. Export important notebooks as PDFs and open
the exports on your Mac. PDFs preserve readable content, not a full editable
notebook restore. Do not proceed on sync status alone if your only copy matters.

For a supplementary copy of the raw notebook directory:

```bash
bash mac.sh backup
```

Leave the tablet idle with notebooks closed and sync finished. Allow more free
Mac disk space than the notebook size printed by preflight. The copy goes to a
private timestamped directory under `~/Documents/OG-riddle-private-backups/`.
The helper validates the archive and records its checksum. It leaves xochitl
running, so this is a **live copy, not an atomic snapshot or a proven restorable
backup**. Changed-file errors or transfer failures stop the step; a `.partial`
file is not a completed backup. Nothing automatically restores or deletes files.
Keep exports and raw backups private; do not upload them here or to GitHub.

Before takeover, privately choose two existing notebooks to reopen afterward.
Record only “notebooks A/B” in the acceptance form, never client or matter names.

## 3. Stage the unchanged build

```bash
bash mac.sh install
```

The helper verifies the pinned payload, reruns preflight, and asks you to confirm
stock firmware and usable backups. It extracts locally, checks all five bundled
executable/launcher hashes, and copies to a new tablet directory:

`/home/root/riddle-rm2-097ccb96/riddle-rm2`

It refuses to overwrite an existing parent directory, including an interrupted
installation. If transfer fails, leave that directory in place and report the
error; do not launch a partial install. It checks hashes again on the tablet.
It does not stop the notebook interface or change system files.

## 4. Enter the provider key privately

```bash
bash mac.sh configure
```

This opens `vi` on the tablet. Press `i`, replace only
`REPLACE_WITH_YOUR_API_KEY` in the quoted value, press Escape, type `:wq`, and
press Return. To cancel without saving: Escape, `:q!`, Return. Keep
`RIDDLE_MEMORY=off`. The unchanged example selects the OpenAI-compatible endpoint
`https://api.openai.com/v1` and `gpt-4o-mini`; live account/model access is untested.
Use a provider API key authorized for this test. No key is bundled. Never paste
the key into chat, a shell command, an issue, or a screenshot. The configuration
is trusted shell syntax and must not contain commands or untrusted text.

```bash
bash mac.sh check
```

This invokes the original no-takeover preflight. It checks key presence, not
validity, billing, provider access, or TLS connectivity. It restricts config
permissions to 600. The provider receives an image of the current handwritten
page. Memory-off disables saved history/context; it does **not** prevent the
working image `/tmp/riddle-page.png` from being written. Normal memory-off flow
removes it after passing the image to the provider worker; a crash can interrupt
that cleanup. Treat any remaining image and provider error/log text as private.
Provider errors can include credential fragments. Do not upload raw journals,
configuration, or page images.

## 5. First launch: short, staged checks

Open a second Terminal in this kit folder and keep it ready with:

```bash
bash mac.sh recover
```

Do not execute recovery yet. In the first Terminal:

```bash
bash mac.sh launch
bash mac.sh status
```

Wait for the diary. If it is blank, frozen, or misaligned, **stop**; press power
once or use recovery. A service becoming active is not proof of working hardware.

1. **Power exit first.** Without writing, briefly press power once. Wait up to
   20 seconds. Confirm the normal notebook interface returns, and run `status`.
   Expect xochitl active/running and Riddle inactive or unloaded. Reopen notebooks
   A/B and confirm their existing pages. Do not continue if this fails.
2. **Pen/display.** Launch again. Use only dots/lines at the four corners (about
   1 cm inside the edges) and center. Check no mirroring, swapping, or material
   offset; note approximate offset in millimeters. Compare light/heavy pressure.
   Test the eraser only if your pen has one; otherwise mark it not applicable.
   Pause may submit these strokes after about 3 seconds, so keep everything
   non-sensitive. Check fade/refresh and legible rendering without persistent
   ghosting or tearing. Power-exit again.
3. **Local help.** Launch, draw a large `?`, and test touch dismissal. Record
   behavior; failure is an acceptance issue, not a reason to change firmware.
4. **One AI reply.** With Wi-Fi connected and the clock correct, write:
   “What is two plus two? Answer briefly.” Rest the pen for about 3 seconds.
   Record whether a relevant, readable reply appears and approximate time to
   first visible reply. Allow up to 120 seconds; do not repeatedly resubmit.
   Power must still exit. A valid reply proves this test only, not general quality.
5. **Normal service stop.** Launch again; run `bash mac.sh stop`. Verify automatic
   notebook-interface restoration and inspect `status` before using `recover`.
6. **Crash recovery.** Only after steps 1 and 5 pass, launch again and run
   `bash mac.sh crash-test`. Verify automatic restoration within 20 seconds.
   This deliberately kills only the Riddle service. If manual recovery is needed,
   mark automatic recovery **FAIL**, even if manual recovery succeeds.
7. **Network failure.** From the normal interface, turn Wi-Fi off, launch, and
   write a harmless question. Observe the error/timeout (up to 120 seconds) and
   test power exit. Re-enable Wi-Fi from the normal interface afterward.
8. **Final notebook check.** Confirm xochitl active, reopen A/B, and check pages
   still present. Leave Riddle stopped. Optional normal reboot can check the
   stock interface returns; never force power off during file transfer/backup.

## Recovery

If power does not restore the interface:

```bash
bash mac.sh recover
bash mac.sh status
```

If you need a fresh Mac Terminal, change to this kit folder and run recovery
there. The helper checks that Riddle is inactive/absent and that no competing
display server remains before starting xochitl. If it cannot establish this, it
stops for investigation. Do not bypass that check with the shorter recovery
commands in the original `docs/RM2.md`.

```bash
cd ~/Downloads/OG-riddle-Mac-RM2
bash mac.sh recover
```

Only use this while recovering this Riddle test, with no other launcher running.
If USB SSH disconnects, reconnect USB and retry. Do not repeatedly launch or
delete notebook files to clear a screen fault. If normal recovery is unavailable,
use the tablet's normal restart procedure; no boot service was installed. Stop
testing and report the failure if the normal interface does not recover.

## Record and return results

Fill `ACCEPTANCE.md` after each step. “Pending,” “not tested,” and “not applicable”
are distinct from pass. Return only the status table and sanitized short errors.
Do not include keys, passwords, notebook names, full journals, or handwriting.
We can then record actual observations in the repository and make any required
correction through a reviewed PR. PR #1 must remain draft/unmerged until all
necessary hardware checks pass. No hardware acceptance is claimed by this kit.

## References and provenance

- [Draft PR #1](https://github.com/barbicanwizkid-arch/OG-riddle/pull/1)
- [Successful build run](https://github.com/barbicanwizkid-arch/OG-riddle/actions/runs/38033938534)
- Source commit: `097ccb96eedde986f19ece1ebb2b3b0b8d881565`
- Artifact ID: `11663168560`; name: `riddle-rm2-3.0.4.1305-experimental`
- `VERIFICATION.md` records checksum and host checks performed for this kit.
- `docs/RM2.md` and `docs/RM2-VALIDATION.md` are unchanged original documents.
  Use this guide for the versioned installation path and temporary-image caveat.
- [Official import/export guidance](https://support.remarkable.com/articles/Knowledge/importing-and-exporting-files)
- [OpenSSH scp manual](https://man.openbsd.org/scp.1): `-O` selects the legacy
  SCP protocol used here for compatibility with the tablet SSH environment.
