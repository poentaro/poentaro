#!/usr/bin/env python3
"""Extract ```poentaro fenced examples from syntax.md and verify each compiles.

Fenced blocks tagged ```poentaro are treated as complete, valid `.poentaro`
programs; every other fence in syntax.md is left alone (grammar fragments,
placeholders, etc. are not expected to be full programs). Add the `poentaro`
language tag to a fence when its content is a full compilable example.
"""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from pathlib import Path

SYNTAX_MD = Path(__file__).resolve().parent.parent / "syntax.md"
FENCE_RE = re.compile(r"```poentaro\n(.*?)```", re.DOTALL)


class Example:
    def __init__(self, line_number: int, source: str):
        self.line_number = line_number
        self.source = source


def find_examples(text: str) -> list[Example]:
    examples = []
    for match in FENCE_RE.finditer(text):
        line_number = text.count("\n", 0, match.start()) + 1
        examples.append(Example(line_number, match.group(1)))
    return examples


def main() -> int:
    text = SYNTAX_MD.read_text(encoding="utf-8")
    examples = find_examples(text)

    if not examples:
        print(f"no ```poentaro examples found in {SYNTAX_MD}")
        return 0

    failures: list[Example] = []
    with tempfile.TemporaryDirectory() as tmp_dir:
        for i, example in enumerate(examples):
            tmp_path = Path(tmp_dir) / f"example_{i}.poentaro"
            tmp_path.write_text(example.source, encoding="utf-8")

            result = subprocess.run(
                ["cargo", "run", "--quiet", "--", "check", str(tmp_path)],
                capture_output=True,
                text=True,
            )
            if result.returncode != 0:
                failures.append(example)
                print(f"--- syntax.md:{example.line_number} failed ---")
                print(result.stdout, end="")
                print(result.stderr, end="")

    if failures:
        print(
            f"{len(failures)}/{len(examples)} ```poentaro examples in syntax.md "
            "failed to compile"
        )
        return 1

    print(f"all {len(examples)} ```poentaro examples in syntax.md compiled cleanly")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
