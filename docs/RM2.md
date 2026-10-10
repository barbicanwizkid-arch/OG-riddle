# Riddle for reMarkable 2 — experimental

Target: **reMarkable 2, software 3.0.4.1305**. This is a port of Maxime Rivest's Riddle, not an official reMarkable app. Host tests and cross-compilation do not certify hardware compatibility. The first tablet run is an acceptance test.

## What works in the port

The diary retains the upstream handwritten prompt → vision model → animated handwritten reply experience. The `rm2` build selects a 1404 × 1872 RGB565 display, direct rm2fb IPC, ARMv7 input records, and the rotated Wacom digitizer. Pen ranges and pressure are read from the device. Pressing **power exits** the diary and restores xochitl (the normal notebook interface). Five-finger exit and in-diary sleep are not enabled on this target.

The bundled rm2fb server is upstream v0.0.33, checksum-verified during packaging. Its embedded configuration explicitly includes 3.0.4.1305 and the matching display-function addresses. This does not establish that a modified xochitl binary works. Use stock firmware; do not install this on a tablet with patched xochitl.

No developer-mode reset, firmware downgrade, AppLoad, remagic, or Toltec installation is part of this procedure. No boot service is enabled and no system files are replaced. The launcher stops the notebook interface for the session, starts a separate rm2fb server, and restores the notebook interface on exit. Back up/sync notebooks before the first hardware test.

## Get the build

Open this repository's **Actions → reMarkable 2 experimental build**. Download the artifact from a successful run on `codex/remarkable-2-port`. Unzip the artifact; it contains `riddle-rm2-3.0.4.1305.tar.gz` and its checksum file. An artifact from a failed run is not an approved test bundle.

## Copy to the tablet (Mac, Linux, or Windows PowerShell with OpenSSH)

Connect the tablet by USB. In the tablet's Settings, open **Help → Copyrights and licenses** to find the SSH password. Keep that password on your computer; do not paste it into GitHub or this chat.

From the folder containing the downloaded archive:

```sh
scp -O riddle-rm2-3.0.4.1305.tar.gz root@10.11.99.1:/home/root/
ssh root@10.11.99.1
```

All following commands run **inside the tablet SSH session**. For the first installation, use a new directory; do not unpack over an existing configured install:

```sh
test ! -e /home/root/riddle-rm2 && tar -xzf /home/root/riddle-rm2-3.0.4.1305.tar.gz -C /home/root
cd /home/root/riddle-rm2
sha256sum -c SHA256SUMS
cp -n oracle.env.example oracle.env
chmod 600 oracle.env
vi oracle.env
```

Replace `REPLACE_WITH_YOUR_API_KEY` with your API key. An API provider key is needed; no account/subscription integration is bundled. Keep the tablet on Wi-Fi with a correct clock for HTTPS. The example uses a vision-capable OpenAI-compatible endpoint; adjust the endpoint/model if you use another provider.

The configuration is shell syntax: values containing special characters must be quoted. Do not source configuration from an untrusted party.

## First launch

```sh
./run.sh --check
./launch.sh
journalctl -u riddle-rm2 -n 80 --no-pager
```

The preflight checks the device model, exact firmware, files/checksums, nonempty API-key setting, and competing rm2fb sessions. It checks key presence, not key validity. No screen takeover occurs during `--check`. The launch then requires a live display-server acknowledgement before opening the diary. The binary also requires exclusive access to the pen and power button.

Write a short non-sensitive question. Rest the pen for about 3 seconds. The page should fade and the reply should appear. Press power to leave.

Recovery, from the same SSH connection:

```sh
systemctl stop riddle-rm2
systemctl start xochitl
```

If the SSH connection drops, the transient service continues. Press power to exit; if needed, reconnect over USB and run the recovery commands. A normal reboot starts the stock interface because no autostart was installed.

## Acceptance checks on the actual tablet

1. Pen strokes align at all four corners, with no mirrored axes; pressure and the eraser work.
2. A short question reaches the configured provider and its reply is legible; no persistent ghosting or display tearing.
3. A large `?` opens the local help page; touching it closes it.
4. Power returns to the notebook interface. Existing notebooks remain accessible.
5. Launch again; `systemctl stop riddle-rm2` restores the notebook interface.
6. In a test session, `systemctl kill --kill-whom=all --signal=KILL riddle-rm2` also restores the interface through systemd's stop hook.
7. With Wi-Fi off, an error appears and power still exits. Never post an API key or handwritten page in a bug report.

## Data handling

The current page image goes to the configured AI provider. `RIDDLE_MEMORY=off` is the bundle default: no stored diary pages or prior-page context. Enabling upstream memory stores handwriting/transcripts/replies locally and supplies context on later requests. Do not use confidential work material without an approved provider/setup. The application's HTTPS client validates certificates; this port does not disable TLS checks.

## Build from source

On Linux (the GitHub workflow does this automatically):

```sh
rustup toolchain install 1.90.0 --profile minimal
rustup override set 1.90.0
rustup target add armv7-unknown-linux-musleabihf
python3 -m pip install ziglang==0.13.0
cargo test --locked
cargo test --locked --features rm2
python3 -m unittest discover -s tests -v
./scripts/build-rm2.sh
./scripts/package-rm2.sh
```

The Rust binary is statically linked using musl to avoid depending on a newer tablet glibc. The separate upstream rm2fb server uses the tablet's own Qt/display libraries. Build flags `rm2` and `takeover` are mutually exclusive. The original Paper Pro build stays available without `rm2`.

## Inspected sources

- Riddle baseline: `0d1a9feea75027543e2ed49c721656cd261f03a1` (MIT; attribution retained).
- [rm2fb IPC and firmware configuration](https://github.com/ddvk/remarkable2-framebuffer/tree/3ce4f8109a146edf5602ac1e434f052659756441/src): queue key, message type/payload layout, RGB565 buffer dimensions, firmware function addresses.
- [rm2fb v0.0.33](https://github.com/ddvk/remarkable2-framebuffer/releases/tag/v0.0.33): server SHA-256 `3ec07729a6bb36c57648492e6d9b5c7abd4eb6c01c4c6079f70fd28c7eab4473`. The actual downloaded ELF includes 3.0.4.1305. MIT license included in the bundle.
- [libremarkable device and rotation implementation](https://github.com/canselcik/libremarkable/tree/d9125f136ed34926c2528c34722aa3494611d644/src/device): rm2 model name, framebuffer path, Wacom rotation. Used as hardware-interface reference; no dependency or code copied from it.

Unverified until hardware acceptance: screen refresh quality/latency, exact power-button/input behavior, server compatibility with the installed binary, and an end-to-end AI reply from the tablet.
