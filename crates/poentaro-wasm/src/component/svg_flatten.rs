use super::*;

/// Pushes a non-`Group` element (`kind` already converted) onto the shared
/// arena, returning its own index into it. `Group` is handled directly in
/// [`flatten_svg_element`]'s own match arm instead, since only that arm
/// needs to recurse into children before this element's record is complete.
pub(super) fn push_leaf_element(
    elements: &mut Vec<SvgElement>,
    element: &crate::svg_types::SvgElementOut,
    kind: SvgKind,
) -> u32 {
    elements.push(SvgElement {
        x: element.x,
        y: element.y,
        variant_tag: element.variant.map(svg_variant_to_wit),
        kind,
    });
    elements.len() as u32 - 1
}

/// Reserves `element`'s own slot in `elements`, recurses into its `Group`
/// children, then patches the reserved slot's `child-indices` in with the
/// real, now-known indices. Split out of [`flatten_svg_element`] purely to
/// keep that function's line count under the crate's `too_many_lines` limit.
pub(super) fn flatten_svg_group(
    element: &crate::svg_types::SvgElementOut,
    children: &[crate::svg_types::SvgElementOut],
    tag: Option<&crate::svg_types::TagOut>,
    elements: &mut Vec<SvgElement>,
) -> u32 {
    // Reserve this element's slot before recursing into its children so its
    // own index stays lower than every descendant's (pre-order), then patch
    // in the real child indices once they're known.
    let self_index = elements.len() as u32;
    elements.push(SvgElement {
        x: element.x,
        y: element.y,
        variant_tag: element.variant.map(svg_variant_to_wit),
        kind: SvgKind::Group(SvgGroupKind {
            child_indices: Vec::new(),
            tag: tag.map(tag_to_wit),
        }),
    });
    let child_indices: Vec<u32> = children
        .iter()
        .map(|child| flatten_svg_element(child, elements))
        .collect();
    if let Some(SvgElement {
        kind: SvgKind::Group(group_kind),
        ..
    }) = elements.get_mut(self_index as usize)
    {
        group_kind.child_indices = child_indices;
    }
    self_index
}

/// Converts every non-`Group` `SvgKindOut` variant to its WIT `SvgKind`.
/// Split out of [`flatten_svg_element`] purely to keep that function's line
/// count under the crate's `too_many_lines` limit — `Group` stays behind in
/// `flatten_svg_element` itself since only that variant needs to recurse
/// into `elements` rather than just convert in place.
pub(super) fn svg_kind_out_to_wit(kind: &crate::svg_types::SvgKindOut) -> SvgKind {
    use crate::svg_types::SvgKindOut;
    match kind {
        SvgKindOut::Group { .. } => unreachable!("Group is handled by flatten_svg_element"),
        SvgKindOut::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
            underline,
        } => SvgKind::Text(svg_text_kind_to_wit(
            content, *font_size, anchor, baseline, font, weight, *italic, *underline,
        )),
        SvgKindOut::Line {
            x2,
            y2,
            stroke_width,
        } => SvgKind::Line(svg_line_kind_to_wit(*x2, *y2, *stroke_width)),
        SvgKindOut::Circle { r } => SvgKind::Circle(svg_circle_kind_to_wit(*r)),
        SvgKindOut::RingCircle { r, stroke_width } => {
            SvgKind::RingCircle(svg_ring_circle_kind_to_wit(*r, *stroke_width))
        }
        SvgKindOut::RoundedBar { width, height } => {
            SvgKind::RoundedBar(svg_rounded_bar_kind_to_wit(*width, *height))
        }
        SvgKindOut::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => SvgKind::Path(svg_path_kind_to_wit(
            *control_x,
            *control_y,
            *end_x,
            *end_y,
            *stroke_width,
        )),
        SvgKindOut::Rect { width, height } => SvgKind::Rect(svg_rect_kind_to_wit(*width, *height)),
        SvgKindOut::ErrorRect { width, height } => {
            SvgKind::ErrorRect(svg_error_rect_kind_to_wit(*width, *height))
        }
        SvgKindOut::PlaybackCursorRect { width, height } => {
            SvgKind::PlaybackCursorRect(svg_playback_cursor_rect_kind_to_wit(*width, *height))
        }
        SvgKindOut::TransparentRect {
            width,
            height,
            role,
        } => SvgKind::TransparentRect(svg_transparent_rect_kind_to_wit(*width, *height, role)),
        SvgKindOut::TextWithTspans {
            font_size,
            anchor,
            baseline,
            font,
            spans,
        } => SvgKind::TextWithTspans(svg_text_with_tspans_kind_to_wit(
            *font_size, anchor, baseline, font, spans,
        )),
    }
}

/// Pre-order-flattens `element` (and, recursively, every descendant it
/// contains via `Group`) into `elements`, returning `element`'s own index
/// into that arena. See `svg-document`'s doc comment in `wit/world.wit` for
/// the flattening contract this must produce: a `Group`'s children become
/// indices into this same arena (`svg-group-kind.child-indices`) instead of
/// nested elements, since the component model cannot express
/// `SvgElementOut`'s original directly-recursive shape. `Group` is the only
/// variant handled here directly (see [`svg_kind_out_to_wit`] for the rest)
/// since it's the only one that needs to recurse into `elements` before its
/// own record is complete.
pub(super) fn flatten_svg_element(
    element: &crate::svg_types::SvgElementOut,
    elements: &mut Vec<SvgElement>,
) -> u32 {
    use crate::svg_types::SvgKindOut;
    match &element.kind {
        SvgKindOut::Group { children, tag } => {
            flatten_svg_group(element, children, tag.as_ref(), elements)
        }
        kind => {
            let kind = svg_kind_out_to_wit(kind);
            push_leaf_element(elements, element, kind)
        }
    }
}

pub(super) fn svg_document_to_wit(document: &crate::svg_types::SvgDocumentOut) -> SvgDocument {
    let mut elements = Vec::new();
    let root_element_indices = document
        .elements
        .iter()
        .map(|element| flatten_svg_element(element, &mut elements))
        .collect();
    SvgDocument {
        width_pt: document.width_pt,
        height_pt: document.height_pt,
        elements,
        root_element_indices,
    }
}
