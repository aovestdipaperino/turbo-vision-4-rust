# How to Build and Deploy the Website

The documentation site at [tv.enzolombardi.net](https://tv.enzolombardi.net) is
an [MkDocs](https://www.mkdocs.org) site using the
[Material](https://squidfunk.github.io/mkdocs-material/) theme. Everything it
needs lives under `website/` in this repository, and Netlify builds and hosts
it from there. This page explains where the content comes from, how to build
the site locally, and what happens on a deploy.

## Layout

```
website/
├── mkdocs.yml          # site configuration and navigation
├── netlify.toml        # Netlify build command and HTTP headers
├── requirements.txt    # pinned mkdocs-material version
├── sync_docs.py        # copies repository docs into website/docs
└── docs/
    ├── index.md        # hand-written pages: home, what's new, tutorials index,
    ├── whats-new.md    #   examples, comparison overview, section indexes
    ├── assets/         # logo, stylesheet, screenshots
    ├── guide/          # generated from docs/user-guide/Chapter-*.md
    ├── reference/      # generated from docs/*.md and CHANGELOG.md
    ├── compare/        # partly generated, partly hand-written
    └── tutorials/      # partly generated, partly hand-written
```

The `site/` and `.cache/` directories that a build produces are ignored by git.

## Where the content comes from

Most pages are copies of documents kept elsewhere in the repository. The
mapping is the `COPIES` table in `website/sync_docs.py`, plus every
`docs/user-guide/Chapter-NN-*.md`, which becomes `guide/chapter-NN.md`. While
copying, the script rewrites links so that references between repository
documents still resolve on the site. A link to `docs/user-guide/Chapter-07-...`
becomes `guide/chapter-07.md`, a link to `../CHANGELOG.md` becomes
`../reference/changelog.md`, and a bare filename link between two copied
documents becomes the slug of the destination page.

The generated copies are committed. That keeps the Netlify build free of any
step other than `mkdocs build`, and it means a reviewer sees exactly what the
site will show. The cost is that the copies go stale when the source changes,
so running the sync script is part of every documentation change.

Hand-written pages (the home page, "What's new", the section index pages, the
tutorials index, the examples pages and the comparison overview) live only in
`website/docs` and are never touched by the script.

## Adding a page

To publish a repository document on the site:

1. Add a line to `COPIES` in `website/sync_docs.py` mapping the source path to
   its page under `website/docs`, for example
   `"docs/CLASS-DIAGRAM.md": "reference/class-diagram.md"`.
2. Add the page to the `nav` section of `website/mkdocs.yml` in the section it
   belongs to. Material shows the page's first heading as its title.
3. Add an entry to the relevant section index, such as
   `website/docs/reference/index.md`, so the page is reachable from the
   section landing page as well as the sidebar.
4. If the document links to other documents with paths the script does not
   already rewrite, add a rule to `LINK_REWRITES`. Links from `guide/` into
   `reference/` or the other way round need an explicit rule, because the
   bare-filename rule only works between pages in the same directory.
5. Run the sync script and build locally as described below.

Mermaid diagrams work out of the box. The `pymdownx.superfences` extension in
`mkdocs.yml` declares a `mermaid` fence, and Material loads the renderer on
pages that contain one.

## Building locally

The site needs Python 3.12 or later. Use a virtual environment so the pinned
`mkdocs-material` version does not fight with anything else on the machine.

```sh
cd website
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt

# Refresh the generated pages from the repository documents.
python3 sync_docs.py

# Serve with live reload at http://127.0.0.1:8000
.venv/bin/mkdocs serve

# Or produce the static site in website/site
.venv/bin/mkdocs build
```

`mkdocs build --strict` turns every warning into a failure. The site does not
currently pass strict mode, because a few copied documents carry anchors and
image paths that were never in the repository, so use the plain build and read
the warnings for the pages you changed. A warning naming a page you edited
usually means a link the sync script did not rewrite.

## Deploying

The site is the Netlify project `turbo-vision-rust`, and it is published from
this machine with the Netlify CLI rather than by a Git-triggered build. Pushing
to `main` does not deploy anything on its own. `website/netlify.toml` still
matters: it sets the publish directory and the HTTP headers that Netlify
applies, so keep it in place.

A deploy is a local build followed by an upload of the `site/` directory:

```sh
cd website
python3 sync_docs.py
.venv/bin/mkdocs build --site-dir site
npx -y netlify-cli deploy --prod --dir=site --no-build --message "what changed"
```

`--no-build` tells the CLI to upload what is already in `site/` instead of
running the build command from `netlify.toml`. The CLI prints the production
URL, a unique URL for that deploy, and a link to the deploy log when it
finishes.

The CLI needs two things. It must be logged in, which `npx netlify-cli login`
does once through the browser and stores under the user's Netlify preferences.
And the `website/` directory must be linked to the site, which is the ignored
file `website/.netlify/state.json` holding the site ID. If it is missing, run
`npx netlify-cli link` from `website/` and pick the `turbo-vision-rust` project,
or recreate the file by hand:

```json
{"siteId": "c56713c9-ce51-4b25-bafa-adca506e1725"}
```

Commit and push the source changes as well, so the repository matches what is
live. If a deploy goes wrong, the previous deploy stays live and any earlier
deploy can be republished from the Netlify dashboard.

`netlify.toml` also sets cache headers: screenshots and images are immutable,
while stylesheets, scripts and pages must be revalidated so a redeploy reaches
browsers that already hold a copy.

## Checklist for a documentation change

Before pushing a change that touches anything under `docs/` or `website/`:

- Run `python3 website/sync_docs.py` so the generated copies match the source.
- Run `mkdocs build` from `website/` and check the warnings for the pages you
  touched.
- Open the page in `mkdocs serve` if it contains a diagram, a table or an
  admonition, since those render differently from a plain Markdown preview.
- Commit the source document and the generated copy together.
- Deploy with the Netlify CLI as described above. A push alone does not update the site.
