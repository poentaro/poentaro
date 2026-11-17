use crate::ast::grouped::Metadata;
use crate::ast::parsed::{ParsedMetadata, TextStyle, TextStyleKind};

use super::resolve_metadata;

/// What each field of a metadata section resolves to when left unset there,
/// given `inherited` (the song's `metadata` when the section is a preset,
/// else nothing) and the rest of the section, `own`, as written: e.g.
/// `notes`' default `font_size` follows an explicit `lyrics` font size, and
/// the `row_height`-scaled font sizes follow an explicit `row_height`. Every
/// style default is found by clearing that kind's style and resolving, so it
/// can't drift from `resolve_metadata`. The text fields
/// (`title`/`subtitle`/`author`) are `inherited`'s.
pub fn metadata_defaults(inherited: &ParsedMetadata, own: &ParsedMetadata) -> Metadata {
    let mut defaults = resolve_metadata(inherited.clone());
    for &kind in TextStyleKind::ALL {
        let mut unset = own.clone();
        *unset.style_mut(kind) = TextStyle::default();
        *defaults.style_mut(kind) = *resolve_metadata(inherited.overlaid_with(&unset)).style(kind);
    }
    defaults
}
