// (C) 2025 - Enzo Lombardi

//! QCell-based safe palette chain for Borland-compatible owner traversal.
//!
//! Replaces raw `*const dyn View` owner pointers with safe `QCell`-wrapped
//! palette chain nodes. Each node stores a view's palette and a link to its
//! parent's node, faithfully reproducing Borland's `TView::mapColor()` owner
//! chain walk without any `unsafe` code.
//!
//! A single `QCellOwner` lives in a `static OnceLock`, so no token parameter
//! needs to be threaded through `draw()` or `map_color()`.

use qcell::{QCell, QCellOwner};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::core::palette::Palette;

/// Global `QCellOwner` governing all palette chain access.
///
/// Initialized once on first use. Because `QCellOwner` is `Sync`, the static
/// is safe. Only `&QCellOwner` is ever handed out, limiting access to
/// `cell.ro()` (shared read). Mutable access via `cell.rw()` is impossible
/// without `&mut QCellOwner`, which cannot be obtained from a static.
static PALETTE_TOKEN: OnceLock<QCellOwner> = OnceLock::new();

/// Get the global palette token.
///
/// First call initializes the `QCellOwner`; subsequent calls return instantly
/// (single atomic load).
pub fn palette_token() -> &'static QCellOwner {
    PALETTE_TOKEN.get_or_init(QCellOwner::new)
}

/// A node in the palette owner chain.
///
/// Each view that participates in palette resolution holds one of these.
/// Children hold an `Rc` clone pointing to their parent's node, forming a
/// chain that mirrors Borland's `TView::owner` pointer chain.
///
/// The chain is rebuilt every frame (just like Borland refreshes owner pointers)
/// and accessed through the global `palette_token()` for safe borrowing.
#[derive(Clone)]
pub struct PaletteChainNode {
    inner: Rc<QCell<PaletteChainData>>,
}

impl std::fmt::Debug for PaletteChainNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaletteChainNode").finish_non_exhaustive()
    }
}

struct PaletteChainData {
    palette: Option<Palette>,
    parent: Option<PaletteChainNode>,
}

impl PaletteChainNode {
    /// Create a new palette chain node.
    ///
    /// # Arguments
    /// * `palette` - This view's palette (from `get_palette()`), or `None` if transparent
    /// * `parent` - Link to the parent view's chain node, or `None` if top-level
    pub fn new(
        palette: Option<Palette>,
        parent: Option<PaletteChainNode>,
    ) -> Self {
        Self {
            inner: Rc::new(QCell::new(palette_token(), PaletteChainData { palette, parent })),
        }
    }

    /// Walk up the owner chain, remapping a color index through each ancestor's palette.
    ///
    /// Faithful reproduction of the Borland `TView::mapColor()` owner-chain walk:
    /// ```text
    /// p = owner;
    /// while (p != 0) {
    ///     if ((curPalette = p->getPalette()) != 0)
    ///         if (color <= curPalette[0])
    ///             color = curPalette[color];
    ///     p = p->owner;
    /// }
    /// ```
    ///
    /// Returns the remapped color index, or 0 on error (caller maps to `ERROR_ATTR`).
    pub fn remap_color(&self, mut color: u8) -> u8 {
        let token = palette_token();
        let data = self.inner.ro(token);

        // Remap through this node's palette (if present and non-empty)
        if let Some(ref palette) = data.palette {
            if !palette.is_empty() && (color as usize) <= palette.len() {
                let remapped = palette.get(color as usize);
                if remapped == 0 {
                    return 0; // Error: caller handles ERROR_ATTR
                }
                color = remapped;
            }
        }

        // Continue up the chain
        if let Some(ref parent) = data.parent {
            parent.remap_color(color)
        } else {
            color
        }
    }
}
