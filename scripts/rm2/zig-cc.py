#!/usr/bin/env python3
"""Cross-compiler adapter for Rust's cc crate (Zig 0.13.0)."""
import os
import sys
from pathlib import Path
import ziglang
zig = str(Path(ziglang.__file__).parent / 'zig')
args = [a for a in sys.argv[1:] if not a.startswith('--target=')]
os.execv(zig, [zig, 'cc', '-target', 'arm-linux-musleabihf', '-mcpu=cortex_a9', *args])
