#!/bin/sh
# Prerequisites: Rust 1.90.0, armv7 musl target, Python 3 + ziglang==0.13.0.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
export CC_armv7_unknown_linux_musleabihf="$ROOT/scripts/rm2/zig-cc.py"
export AR_armv7_unknown_linux_musleabihf="$ROOT/scripts/rm2/zig-ar.py"
HOST=$(rustc -vV | sed -n 's/^host: //p')
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER="$(rustc --print sysroot)/lib/rustlib/$HOST/bin/rust-lld"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUSTFLAGS="-C linker-flavor=ld.lld"
case "${1:-build}" in
    build) cargo build --locked --release --features rm2 --target armv7-unknown-linux-musleabihf ;;
    test)
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUNNER="${RIDDLE_ARM_RUNNER:-qemu-arm}"
        cargo test --locked --features rm2 --target armv7-unknown-linux-musleabihf
        ;;
    *) echo "usage: $0 [build|test]" >&2; exit 2 ;;
esac
