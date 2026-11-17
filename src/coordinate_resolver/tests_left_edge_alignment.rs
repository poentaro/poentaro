//! A note head, a rest, a percussion hit, a chord symbol, a Latin lyric
//! syllable, and a CJK lyric syllable can all share the same column. Every
//! one of these six flush-left content types now draws `TextAnchor::Start`
//! at exactly the anchor `resolve()` computes (`x_start(column) +
//! padding - bearing(leading_char)` — see
//! `resolve::flush_left_padding`), so every ink-left-edge has the same
//! shape: `anchor + bearing(leading_char)`. This test computes each
//! element's predicted ink-left-edge that way and asserts they coincide for
//! elements sharing one column — the perfect left-edge alignment this
//! anchor-mode standardization exists to make achievable exactly, not just
//! approximately.

use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::compositor::types::{AbsoluteContent, FontFamily, GlyphFontFamilies};
use crate::coordinate_resolver::resolve::{
    resolve, ElementPaddings, LabelFontSizes, LyricFontSizes, ResolveFontSizes,
};
use crate::font_metrics::{cjk_glyph_left_bearing, glyph_left_bearing_for_family};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

const NOTE_NUMBER_WIDTH: f32 = 12.0;
const NOTES_FONT_SIZE: f32 = 12.0;
const CHORDS_FONT_SIZE: f32 = 12.0;
const LYRIC_FONT_SIZES: LyricFontSizes = LyricFontSizes {
    base: 14.4,
    cjk: 17.28,
};
const LABEL_FONT_SIZES: LabelFontSizes = LabelFontSizes {
    measure_number: 10.0,
    section_label: 12.0,
    section_label_vertical_padding_pt: 0.0,
    part_label: 12.0,
    measure_number_bold: false,
    measure_number_italic: false,
    measure_number_underline: false,
    measure_number_font_family: FontFamily::SansSerif,
    section_label_bold: false,
    section_label_italic: false,
    section_label_underline: false,
    section_label_font_family: FontFamily::SansSerif,
    part_label_bold: false,
    part_label_italic: false,
    part_label_underline: false,
    part_label_font_family: FontFamily::SansSerif,
    sequence_bold: false,
    sequence_italic: false,
    sequence_underline: false,
    sequence_font_family: FontFamily::SansSerif,
};
const ELEMENT_PADDINGS: ElementPaddings = ElementPaddings {
    notes: 4.0,
    chords: 4.0,
    lyrics: 4.0,
    note_dash: 4.0,
};
const GLYPH_FONT_FAMILIES: GlyphFontFamilies = GlyphFontFamilies {
    notes: FontFamily::Monospace,
    chords: FontFamily::Monospace,
    note_dash: FontFamily::Monospace,
};
const RESOLVE_FONT_SIZES: ResolveFontSizes = ResolveFontSizes {
    lyric: LYRIC_FONT_SIZES,
    notes: NOTES_FONT_SIZE,
    chords: CHORDS_FONT_SIZE,
    labels: LABEL_FONT_SIZES,
    paddings: ELEMENT_PADDINGS,
    page_number_vertical_padding_pt: 0.0,
    glyph_font_families: GLYPH_FONT_FAMILIES,
};

fn single_row_page(element: GridElement) -> GridPage {
    GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![element],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![],
        bar_line_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    }
}

/// Resolves a single element at column 0 and returns its `AbsoluteElement`'s
/// `x` (the anchor the renderer draws `TextAnchor::Start` at).
fn resolve_anchor_x(content: GridContent) -> f32 {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content,
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        NOTE_NUMBER_WIDTH,
        super::tests::TEST_PAGE_FRAME,
        RESOLVE_FONT_SIZES,
    )
    .unwrap();
    abs[0]
        .elements
        .iter()
        .find(|e| {
            !matches!(
                e.content,
                AbsoluteContent::Underline { .. }
                    | AbsoluteContent::TieOrSlur { .. }
                    | AbsoluteContent::TupletBracket { .. }
            )
        })
        .expect("should have exactly one non-span-marking element")
        .x
}

/// Ink-left-edge of a `TextAnchor::Start`-drawn glyph: ink starts `bearing`
/// to the right of the anchor. The same shape for every flush-left content
/// type now that they all share one anchor mode — only the font face
/// (monospace vs. CJK) and font size behind `bearing` differ per type.
fn ink_left_edge(anchor: f32, bearing: f32) -> f32 {
    anchor + bearing
}

#[test]
fn note_head_and_latin_lyric_syllable_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let lyric_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "la".to_string(),
        ruby: None,
        ruby_band_pt: 0.0,
        source_part_index: 0,
        note_id: 0,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        glyph_left_bearing_for_family(FontFamily::Monospace, '1', NOTES_FONT_SIZE),
    );
    let lyric_ink_left = ink_left_edge(
        lyric_anchor,
        cjk_glyph_left_bearing('l', LYRIC_FONT_SIZES.base),
    );

    assert!(
        (note_ink_left - lyric_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} lyric_ink_left={lyric_ink_left} should coincide: a note \
         and a Latin lyric syllable sharing a column should have perfectly aligned left ink \
         edges"
    );
}

#[test]
fn note_head_and_cjk_lyric_syllable_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let lyric_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "春".to_string(),
        ruby: None,
        ruby_band_pt: 0.0,
        source_part_index: 0,
        note_id: 0,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        glyph_left_bearing_for_family(FontFamily::Monospace, '1', NOTES_FONT_SIZE),
    );
    let lyric_ink_left = ink_left_edge(
        lyric_anchor,
        cjk_glyph_left_bearing('春', LYRIC_FONT_SIZES.cjk),
    );

    assert!(
        (note_ink_left - lyric_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} lyric_ink_left={lyric_ink_left} should coincide: a note \
         and a CJK lyric syllable sharing a column should have perfectly aligned left ink edges"
    );
}

#[test]
fn note_head_and_chord_symbol_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let chord_anchor = resolve_anchor_x(GridContent::ChordSymbol {
        text: "1m".to_string(),
        dotted: false,
        double_dotted: false,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        glyph_left_bearing_for_family(FontFamily::Monospace, '1', NOTES_FONT_SIZE),
    );
    let chord_ink_left = ink_left_edge(
        chord_anchor,
        glyph_left_bearing_for_family(FontFamily::Monospace, '1', CHORDS_FONT_SIZE),
    );

    assert!(
        (note_ink_left - chord_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} chord_ink_left={chord_ink_left} should coincide: a note \
         and a chord symbol sharing a column should have perfectly aligned left ink edges"
    );
}

#[test]
fn latin_and_cjk_lyric_syllables_have_the_same_ink_left_edge_in_one_column() {
    let latin_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "la".to_string(),
        ruby: None,
        ruby_band_pt: 0.0,
        source_part_index: 0,
        note_id: 0,
    });
    let cjk_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "春".to_string(),
        ruby: None,
        ruby_band_pt: 0.0,
        source_part_index: 0,
        note_id: 0,
    });

    let latin_ink_left = ink_left_edge(
        latin_anchor,
        cjk_glyph_left_bearing('l', LYRIC_FONT_SIZES.base),
    );
    let cjk_ink_left = ink_left_edge(
        cjk_anchor,
        cjk_glyph_left_bearing('春', LYRIC_FONT_SIZES.cjk),
    );

    assert!(
        (latin_ink_left - cjk_ink_left).abs() < 0.01,
        "latin_ink_left={latin_ink_left} cjk_ink_left={cjk_ink_left} should coincide: a Latin \
         and a CJK lyric syllable sharing a column should have perfectly aligned left ink edges"
    );
}
