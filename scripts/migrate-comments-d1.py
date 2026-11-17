#!/usr/bin/env python3
"""Turn the `#` comments in stored scores into `##` comments.

    python3 scripts/migrate-comments-d1.py [--sqlite <path>] [--apply]

`#` now starts an anyx line string, so a file saved when it started a comment
reads differently. `tools/migrate-comments` rewrites each such file, and holds
back, by name, any that still isn't a document with its `#` lines as comments
(such as text in the removed section syntax), for a person to fix. Without
`--apply` this is a dry run: it writes the SQL and changes nothing.
`--sqlite <path>` works on a local copy instead of production.

Each update also writes a `file_versions` row, because a new revision with no
row leaves a gap that hides the file's older versions. It only applies if the
revision is unchanged, so a save made during the migration wins and that file
can be picked up by running this again.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKER_DIR = ROOT / "crates" / "live-share-worker"
SELECT_FILES = "SELECT id, name, content, revision FROM files"
# D1 rejects a longer statement.
MAX_STATEMENT_BYTES = 100_000


def run(command: list[str], stdin: str | None = None, cwd: Path = ROOT) -> str:
    result = subprocess.run(command, input=stdin, capture_output=True, text=True, cwd=cwd)
    if result.returncode != 0:
        sys.exit(f"{' '.join(command)} failed:\n{result.stderr}")
    return result.stdout


def wrangler_execute(*arguments: str) -> str:
    return run(
        ["pnpm", "exec", "wrangler", "d1", "execute", "jianpu-live-share", "--remote", *arguments],
        cwd=WORKER_DIR,
    )


def fetch_rows(sqlite_path: str | None) -> list[dict]:
    if sqlite_path is None:
        return json.loads(wrangler_execute("--json", f"--command={SELECT_FILES}"))[0]["results"]
    with sqlite3.connect(sqlite_path) as database:
        database.row_factory = sqlite3.Row
        return [dict(row) for row in database.execute(SELECT_FILES)]


def sql_string(text: str) -> str:
    return "'" + text.replace("'", "''") + "'"


def statements_for(migration: dict) -> list[str]:
    file_id = sql_string(migration["id"])
    revision = migration["revision"]
    unchanged = f"id = {file_id} AND revision = {revision}"
    return [
        "INSERT INTO file_versions (file_id, revision, newer_revision, reverse_patch, saved_at) "
        f"SELECT id, revision, revision + 1, {sql_string(migration['reverse_patch'])}, updated_at "
        f"FROM files WHERE {unchanged};",
        f"UPDATE files SET content = {sql_string(migration['converted'])}, revision = revision + 1 "
        f"WHERE {unchanged};",
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--apply", action="store_true", help="execute the generated SQL")
    parser.add_argument("--sqlite", help="use this local copy instead of production")
    parser.add_argument("--output", default="target/migrate-comments.sql")
    args = parser.parse_args()

    rows = fetch_rows(args.sqlite)
    output = run(["cargo", "run", "--quiet", "-p", "migrate-comments"], stdin=json.dumps(rows))
    result = json.loads(output)
    migrations = result["converted"]
    for name in result["held"]:
        print(f"HELD {name}")
    for migration in migrations:
        print(f"{migration['name']} [{migration['id']}]")

    statements = [statement for migration in migrations for statement in statements_for(migration)]
    too_long = [s for s in statements if len(s.encode()) > MAX_STATEMENT_BYTES]
    if too_long:
        sys.exit(f"{len(too_long)} statements are over D1's {MAX_STATEMENT_BYTES} byte limit")

    sql_path = ROOT / args.output
    sql_path.write_text("\n".join(statements) + ("\n" if statements else ""))
    print(f"wrote {len(migrations)} of {len(rows)} files to {sql_path}")

    if not args.apply or not statements:
        return
    if args.sqlite is None:
        wrangler_execute("--yes", f"--file={sql_path}")
    else:
        with sqlite3.connect(args.sqlite) as database:
            database.executescript(sql_path.read_text())
    print("applied")


if __name__ == "__main__":
    main()
