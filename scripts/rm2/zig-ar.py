#!/usr/bin/env python3
import os
import sys
from pathlib import Path
import ziglang
zig = str(Path(ziglang.__file__).parent / 'zig')
os.execv(zig, [zig, 'ar', *sys.argv[1:]])
