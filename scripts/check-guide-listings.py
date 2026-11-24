#!/usr/bin/env python3
"""Compile the complete programs in the user guide against this checkout.

A listing counts as a complete program when its ```rust block has a `fn main(`
and at least one `use turbo_vision` line. Fragments (no `fn main`, or no
imports) are left alone: they lean on code shown elsewhere in their chapter.

Each program becomes a binary of a throwaway crate that depends on this
repository by path, and `cargo check` builds them all, so a listing that stops
compiling after an API change fails here instead of in a reader's terminal.

To exempt a program that cannot compile on its own, put
<!-- guide-check: skip --> on the line directly above its opening fence.

Usage:  python3 scripts/check-guide-listings.py
Set CARGO_TARGET_DIR to reuse a build directory.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
GUIDE = REPO / "docs" / "user-guide"
FENCE = re.compile(r"^(?P<marker>[^\n]*)\n```rust\n(?P<code>.*?)```", re.S | re.M)
SKIP = "<!-- guide-check: skip -->"


def listings() -> list[tuple[str, int, str]]:
    found = []
    for page in sorted(GUIDE.glob("Chapter-*.md")):
        text = page.read_text(encoding="utf-8").replace("\r\n", "\n")
        for m in FENCE.finditer(text):
            code = m.group("code")
            if m.group("marker").strip() == SKIP:
                continue
            if not re.search(r"^fn main\(", code, re.M):
                continue
            if "use turbo_vision" not in code:
                continue
            line = text[: m.start("code")].count("\n") + 1
            found.append((page.name, line, code))
    return found


def main() -> int:
    programs = listings()
    if not programs:
        print("no complete listings found", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="guide-listings-") as tmp:
        crate = Path(tmp)
        (crate / "src" / "bin").mkdir(parents=True)
        (crate / "src" / "lib.rs").write_text("")
        names: dict[str, str] = {}
        for page, line, code in programs:
            number = re.search(r"Chapter-(\d+)", page).group(1)
            name = f"ch{number}_line{line}"
            names[name] = f"{page}:{line}"
            (crate / "src" / "bin" / f"{name}.rs").write_text(code)
        (crate / "Cargo.toml").write_text(
            "[package]\n"
            'name = "guide-listings"\n'
            'version = "0.0.0"\n'
            'edition = "2024"\n\n'
            "[dependencies]\n"
            f'turbo-vision = {{ path = "{REPO.as_posix()}" }}\n'
        )
        env = dict(os.environ)
        env.setdefault("CARGO_TARGET_DIR", str(REPO / "target" / "guide-listings"))
        result = subprocess.run(
            ["cargo", "check", "--bins", "--message-format=short", "--color=never"],
            cwd=crate,
            env=env,
            capture_output=True,
            text=True,
        )

    errors = [l for l in result.stderr.splitlines() if re.search(r"(^|[\s:])error(\[E\d+\])?:", l)]
    if result.returncode == 0:
        print(f"{len(programs)} guide listings compile")
        return 0
    print(f"guide listings failed to compile ({len(programs)} checked):", file=sys.stderr)
    for line in errors:
        for name, where in names.items():
            line = line.replace(f"src/bin/{name}.rs", where)
        print("  " + line, file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
