#!/usr/bin/env python3
"""Copy documentation out of the repository into website/docs.

Run from anywhere:  python3 website/sync_docs.py
Hand-written pages (index, whats-new, comparison, examples, tutorials index)
live in website/docs and are never touched by this script.
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SITE_DOCS = REPO / "website" / "docs"

# Repository file -> page under website/docs
COPIES: dict[str, str] = {
    "CHANGELOG.md": "reference/changelog.md",
    "UPGRADING-TO-3.0.md": "reference/upgrading.md",
    "docs/TURBO-VISION-DESIGN.md": "reference/design.md",
    "docs/OWNER-COORDINATES.md": "reference/owner-coordinates.md",
    "docs/MISSING-INHERITANCE.md": "reference/inheritance.md",
    "docs/RUST-API-CATALOG.md": "reference/api-catalog.md",
    "docs/RUST-API-CATALOGUE-INDEX.md": "reference/api-index.md",
    "docs/RUST-IMPLEMENTATION-REFERENCE.md": "reference/implementation.md",
    "docs/RUST-CODING-GUIDELINES.md": "reference/coding-guidelines.md",
    "docs/PALETTE-SYSTEM.md": "reference/palette-system.md",
    "docs/PALETTE-SYSTEM-DESIGN.md": "reference/palette-design.md",
    "docs/BORLAND-PALETTE-CHART.md": "reference/palette-chart.md",
    "docs/SERIALIZATION-PERSISTENCE.md": "reference/serialization.md",
    "docs/SERIALIZATION-QUICK-REFERENCE.md": "reference/serialization-quick.md",
    "docs/MORE-CONTROLS.md": "reference/more-controls.md",
    "docs/BIORHYTHM-CALCULATOR-TUTORIAL.md": "tutorials/biorhythm.md",
    "docs/CUSTOM-APPLICATION-RUST-EXAMPLE.md": "compare/custom-application-rust.md",
    "docs/CUSTOM-PROGRAM-CPP-EXAMPLE.md": "compare/custom-program-cpp.md",
}

IMAGES = ["logo.png"]
IMAGE_DIRS: list[str] = []  # captures live in docs/assets/shots

# Links that point at repository paths and must be rewritten for the site.
LINK_REWRITES = [
    (r"\]\(\.\./CHANGELOG\.md", "](../reference/changelog.md"),
    (r"\]\(docs/user-guide/Chapter-(\d\d)[^)]*\.md\)", r"](guide/chapter-\1.md)"),
    (r"\]\(screenshots/", "](assets/screenshots/"),
    # Images referenced by the coding guidelines were never in the repository.
    (r"!\[[^\]]*\]\(M-[A-Z0-9_-]+\.png\)\n?", ""),
]

# A bare filename link between two repository documents becomes the site slug of
# the page that document was copied to.
def basename_rewrites() -> list[tuple[str, str]]:
    rules = []
    for rel_src, rel_dst in {**COPIES, **chapter_pages()}.items():
        name = Path(rel_src).name
        slug = Path(rel_dst).name
        rules.append((rf"\]\((?:\./)?(?:docs/)?(?:user-guide/)?{re.escape(name)}", f"]({slug}"))
    return rules


def chapter_pages() -> dict[str, str]:
    pages = {}
    for src in sorted((REPO / "docs" / "user-guide").glob("Chapter-*.md")):
        number = src.name.split("-", 2)[1]
        pages[f"docs/user-guide/{src.name}"] = f"guide/chapter-{number}.md"
    return pages


def rewrite(text: str) -> str:
    for pattern, replacement in LINK_REWRITES + basename_rewrites():
        text = re.sub(pattern, replacement, text)
    return text


def copy_page(rel_src: str, rel_dst: str) -> None:
    src = REPO / rel_src
    dst = SITE_DOCS / rel_dst
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(rewrite(src.read_text(encoding="utf-8")), encoding="utf-8")


def main() -> None:
    pages = {**COPIES, **chapter_pages()}
    for rel_src, rel_dst in pages.items():
        copy_page(rel_src, rel_dst)

    assets = SITE_DOCS / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    for name in IMAGES:
        shutil.copy2(REPO / name, assets / name)
    for name in IMAGE_DIRS:
        target = assets / name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(REPO / name, target)

    print(f"synced {len(pages)} pages and {len(IMAGES) + len(IMAGE_DIRS)} asset entries")


if __name__ == "__main__":
    main()
