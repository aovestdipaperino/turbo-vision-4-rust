# Reference

Documents kept in the repository and published here unchanged. They are written for people working
on the crate as much as with it.

## Release history

- **[Upgrading to 3.0.0](upgrading.md)** &mdash; the ordered list of what to change in a crate that implements views of its own, and what reaches ordinary application code.
- **[Changelog](changelog.md)** &mdash; every release, in Keep a Changelog form. The 3.0.0 entry doubles as the migration guide.
- **[What's new](../whats-new.md)** &mdash; the same recent work, in prose.

## Architecture

- **[Design document](design.md)** &mdash; the long one. How the Borland class tree became layered traits, and why each decision went the way it did.
- **[Owner-relative coordinates](owner-coordinates.md)** &mdash; the coordinate model: the origin stack, `extent`, and how a view draws and receives events in its own space.
- **[Missing inheritance](inheritance.md)** &mdash; the analysis that produced the 3.0.0 shape, listing what C++ inheritance provided and what replaced each piece.
- **[Implementation reference](implementation.md)** &mdash; events, commands, menus, status lines, message boxes and the enable/disable system, with code for each.

## API

- **[API index](api-index.md)** &mdash; the map. Start here to find a type.
- **[API catalogue](api-catalog.md)** &mdash; every public type and method.
- **[More controls](more-controls.md)** &mdash; the widgets added beyond the Borland set.
- **[Coding guidelines](coding-guidelines.md)** &mdash; the conventions the crate holds itself to. Worth reading before sending a patch.

## Palettes

- **[Palette system](palette-system.md)** &mdash; how a colour index is resolved through the chain of owners.
- **[Palette design](palette-design.md)** &mdash; why the chain is pushed down at draw time rather than pulled up through a parent pointer.
- **[Borland palette chart](palette-chart.md)** &mdash; the original colour tables, entry by entry.

## Persistence

- **[Serialization and persistence](serialization.md)** &mdash; saving a view tree and reading it back.
- **[Quick reference](serialization-quick.md)** &mdash; the short version.

## Elsewhere

- [The crate on docs.rs](https://docs.rs/turbo-vision) for generated API documentation.
- [The repository](https://github.com/aovestdipaperino/turbo-vision-4-rust) for source, issues and the examples.
