#!/usr/bin/env python3
"""Reject a built wasm binary that exceeds MAX_BYTES.

Guards against accidentally re-embedding a large asset (e.g. a CJK font) into
`poentaro_wasm.core.wasm` instead of loading it at runtime (see `src/font_metrics.rs`).
"""

from __future__ import annotations

import sys
from pathlib import Path

MAX_BYTES = 9 * 1024 * 1024
WASM_PATH = (
    Path(__file__).resolve().parent.parent
    / "crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm"
)


def main() -> int:
    size = WASM_PATH.stat().st_size
    if size <= MAX_BYTES:
        return 0

    print(
        f"{WASM_PATH.as_posix()} is {size / 1024 / 1024:.1f} MiB, "
        f"over the {MAX_BYTES / 1024 / 1024:.0f} MiB limit."
    )
    print(
        "If this is from a new embedded asset, load it at runtime instead "
        "(see set_layout_fonts in "
        "crates/poentaro-wasm/src/component/guest_metadata_and_misc.rs for the "
        "pattern)."
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
