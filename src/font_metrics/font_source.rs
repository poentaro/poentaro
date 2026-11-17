//! Loads and caches the parsed `ttf_parser::Face`s that back
//! `font_metrics`'s glyph-advance measurements, one per `FontFamily` role —
//! one implementation for non-wasm builds (fonts embedded at compile time)
//! and one for wasm (fonts fetched and pushed in at runtime). Faces are
//! addressed only by role; which role measures which text (directive line,
//! lyrics, noteheads...) is decided by `super`'s measuring functions.

// Non-wasm builds (the CLI, `cargo test`) embed the fonts at compile time.
#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use crate::compositor::types::FontFamily;
    use crate::fonts::FontBytesByFamily;
    use std::sync::LazyLock;

    type LazyFace = LazyLock<Option<ttf_parser::Face<'static>>>;

    // Each role's embedded font, parsed once so its real glyph advance
    // widths can be used instead of a character-bucket heuristic. `None`
    // only if the embedded font fails to parse, which shouldn't happen for
    // a file fixed at compile time.
    static SERIF_FACE: LazyFace =
        LazyLock::new(|| ttf_parser::Face::parse(crate::fonts::SERIF_FONT_BYTES, 0).ok());
    static SANS_SERIF_FACE: LazyFace =
        LazyLock::new(|| ttf_parser::Face::parse(crate::fonts::SANS_SERIF_FONT_BYTES, 0).ok());
    static MONOSPACE_FACE: LazyFace =
        LazyLock::new(|| ttf_parser::Face::parse(crate::fonts::MONOSPACE_FONT_BYTES, 0).ok());

    pub(crate) fn face_for_family(
        family: FontFamily,
    ) -> Option<&'static ttf_parser::Face<'static>> {
        match family {
            FontFamily::Serif => SERIF_FACE.as_ref(),
            FontFamily::SansSerif => SANS_SERIF_FACE.as_ref(),
            FontFamily::Monospace => MONOSPACE_FACE.as_ref(),
        }
    }

    /// No-op on non-wasm builds: the fonts are already embedded at compile
    /// time, so there's nothing to receive at runtime. Exists so callers
    /// (e.g. `crates/poentaro-wasm`, which is also built for the host arch as
    /// a workspace member) don't need their own `cfg` gate.
    pub(crate) fn set_layout_font_bytes(_fonts: FontBytesByFamily) {}
}

// The wasm build has no compile-time font bytes: `set_layout_font_bytes` is
// called at runtime (from `crates/poentaro-wasm`) once the app has fetched the
// same font bytes it already needs for PDF export. Two `OnceLock`s per role
// (rather than a `LazyLock`) so a render that races ahead of the fetch just
// falls back to `FALLBACK_ADVANCE_WIDTH_RATIO` for that one call, instead of
// a `LazyLock` permanently caching `None` if it happened to be evaluated
// before the bytes arrived.
#[cfg(target_arch = "wasm32")]
mod imp {
    use crate::compositor::types::FontFamily;
    use crate::fonts::FontBytesByFamily;
    use std::sync::OnceLock;

    struct FontSlot {
        bytes: OnceLock<Vec<u8>>,
        face: OnceLock<ttf_parser::Face<'static>>,
    }

    impl FontSlot {
        const fn new() -> Self {
            Self {
                bytes: OnceLock::new(),
                face: OnceLock::new(),
            }
        }

        fn face(&'static self) -> Option<&'static ttf_parser::Face<'static>> {
            if let Some(face) = self.face.get() {
                return Some(face);
            }
            let bytes = self.bytes.get()?;
            let face = ttf_parser::Face::parse(bytes, 0).ok()?;
            Some(self.face.get_or_init(|| face))
        }
    }

    static SERIF_SLOT: FontSlot = FontSlot::new();
    static SANS_SERIF_SLOT: FontSlot = FontSlot::new();
    static MONOSPACE_SLOT: FontSlot = FontSlot::new();

    fn slot_for_family(family: FontFamily) -> &'static FontSlot {
        match family {
            FontFamily::Serif => &SERIF_SLOT,
            FontFamily::SansSerif => &SANS_SERIF_SLOT,
            FontFamily::Monospace => &MONOSPACE_SLOT,
        }
    }

    pub(crate) fn set_layout_font_bytes(fonts: FontBytesByFamily) {
        let FontBytesByFamily {
            serif,
            sans_serif,
            monospace,
        } = fonts;
        slot_for_family(FontFamily::Serif).bytes.set(serif).ok();
        slot_for_family(FontFamily::SansSerif)
            .bytes
            .set(sans_serif)
            .ok();
        slot_for_family(FontFamily::Monospace)
            .bytes
            .set(monospace)
            .ok();
    }

    pub(crate) fn face_for_family(
        family: FontFamily,
    ) -> Option<&'static ttf_parser::Face<'static>> {
        slot_for_family(family).face()
    }
}

pub(crate) use imp::{face_for_family, set_layout_font_bytes};
