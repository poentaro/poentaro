use crate::ast::parsed::Offset;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage, FontFamily};
use crate::render_config::RenderConfig;
use crate::renderer::new_types::{SvgDocument, SvgElement};
use glyph_renderers::{
    render_bar_line, render_chord_symbol, render_guitar_diagram, render_horizontal_line,
    render_low_octave_dots, render_lyric, render_lyric_line, render_multi_measure_rest,
    render_note_dash, render_note_head, render_percussion_hit, render_rest, render_tie_or_slur,
    render_tuplet_bracket, render_underline, DotState, NoteHeadGlyph, NoteRenderParams,
};
use overlay::render_overlay_element;

mod click_targets;
mod directive_line;
mod glyph_renderers;
mod overlay;

pub fn render_new(pages: &[AbsolutePage], config: &RenderConfig) -> Vec<SvgDocument> {
    pages.iter().map(|page| render_page(page, config)).collect()
}

fn render_page(page: &AbsolutePage, config: &RenderConfig) -> SvgDocument {
    let row_height = config.row_height as f32;
    let lyric_font_size = config.lyric_font_size();
    let cjk_font_size = config.lyric_cjk_font_size();
    let notes_font_size = config.notes_font_size();
    let chords_font_size = config.chords_font_size();
    let note_dash_font_size = config.note_dash_font_size();
    let note_number_width = config.note_number_width as f32;

    let params = RenderElementParams {
        row_height,
        lyric_font_size,
        cjk_font_size,
        notes_font_size,
        chords_font_size,
        note_dash_font_size,
        note_number_width,
        directive_row_offset: config.directive_row_offset,
        notes_style: GlyphStyle {
            bold: config.notes_bold,
            italic: config.notes_italic,
            underline: config.notes_underline,
            font_family: config.glyph_font_families.notes,
        },
        chords_style: GlyphStyle {
            bold: config.chords_bold,
            italic: config.chords_italic,
            underline: config.chords_underline,
            font_family: config.glyph_font_families.chords,
        },
        lyrics_style: GlyphStyle {
            bold: config.lyrics_bold,
            italic: config.lyrics_italic,
            underline: config.lyrics_underline,
            // Lyrics' font family is threaded separately as
            // `lyrics_font_family` below (`render_lyric`/`render_lyric_line`
            // take it as a standalone argument, not via `GlyphStyle`), so
            // this field is unused for the lyrics glyph — set to match
            // anyway for `GlyphStyle`'s own consistency.
            font_family: config.lyrics_font_family,
        },
        note_dash_style: GlyphStyle {
            bold: config.note_dash_bold,
            italic: config.note_dash_italic,
            underline: config.note_dash_underline,
            font_family: config.glyph_font_families.note_dash,
        },
        lyrics_font_family: config.lyrics_font_family,
    };

    let elements = page
        .elements
        .iter()
        .flat_map(|elem| render_element(elem, &params))
        .collect();

    SvgDocument {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}

/// A glyph kind's bold/italic/underline/font_family, read from `RenderConfig`
/// (see `Metadata::notes_style`/`chords_style`/`lyrics_style`/`note_dash_style`).
/// `font_family` is unused for `lyrics_style` specifically — `render_lyric`/
/// `render_lyric_line` take `RenderElementParams::lyrics_font_family` as its
/// own argument instead (see `render_page`).
#[derive(Clone, Copy)]
struct GlyphStyle {
    bold: bool,
    italic: bool,
    underline: bool,
    font_family: FontFamily,
}

struct RenderElementParams {
    row_height: f32,
    lyric_font_size: f32,
    cjk_font_size: f32,
    notes_font_size: f32,
    chords_font_size: f32,
    note_dash_font_size: f32,
    note_number_width: f32,
    directive_row_offset: Offset,
    notes_style: GlyphStyle,
    chords_style: GlyphStyle,
    lyrics_style: GlyphStyle,
    note_dash_style: GlyphStyle,
    /// See `Metadata::lyrics.font_family`.
    lyrics_font_family: FontFamily,
}

fn render_element(elem: &AbsoluteElement, params: &RenderElementParams) -> Vec<SvgElement> {
    let RenderElementParams {
        row_height,
        lyric_font_size,
        cjk_font_size,
        notes_font_size,
        chords_font_size,
        directive_row_offset,
        notes_style,
        chords_style,
        lyrics_style,
        lyrics_font_family,
        ..
    } = params;
    match &elem.content {
        AbsoluteContent::NoteHead { .. }
        | AbsoluteContent::Rest { .. }
        | AbsoluteContent::NoteDash { .. }
        | AbsoluteContent::LowOctaveDots { .. } => render_note_glyph(elem, &elem.content, params),
        AbsoluteContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => render_chord_symbol(
            elem,
            text,
            &DotState::new(*dotted, *double_dotted),
            chords_font_size,
            *chords_style,
        ),
        AbsoluteContent::GuitarDiagram { voicing } => render_guitar_diagram(elem, voicing),
        AbsoluteContent::PercussionHit => {
            render_percussion_hit(elem, notes_font_size, *notes_style)
        }
        AbsoluteContent::Lyric { text, .. } => render_lyric(
            elem,
            text,
            lyric_font_size,
            cjk_font_size,
            *lyrics_style,
            *lyrics_font_family,
        ),
        AbsoluteContent::LyricLine(s) => render_lyric_line(
            elem,
            s,
            lyric_font_size,
            cjk_font_size,
            *lyrics_style,
            *lyrics_font_family,
        ),
        AbsoluteContent::MultiMeasureRest { .. }
        | AbsoluteContent::Underline { .. }
        | AbsoluteContent::TieOrSlur { .. }
        | AbsoluteContent::TupletBracket { .. }
        | AbsoluteContent::BarLine { .. }
        | AbsoluteContent::HorizontalLine { .. } => {
            render_simple_glyph(elem, &elem.content, row_height, notes_font_size)
        }
        AbsoluteContent::Text { .. }
        | AbsoluteContent::MeasureHighlight { .. }
        | AbsoluteContent::ErrorHighlight { .. }
        | AbsoluteContent::MeasureClickTarget { .. }
        | AbsoluteContent::BarNumberClickTarget { .. }
        | AbsoluteContent::BarLineClickTarget { .. }
        | AbsoluteContent::PlaybackCursorTarget { .. }
        | AbsoluteContent::NoteClickTarget { .. }
        | AbsoluteContent::PartLabelClickTarget { .. }
        | AbsoluteContent::LyricClickTarget { .. }
        | AbsoluteContent::LyricLabelClickTarget { .. }
        | AbsoluteContent::DirectiveLine { .. } => {
            render_overlay_element(elem, &elem.content, *directive_row_offset)
        }
    }
}

/// The `NoteHead`/`Rest`/`NoteDash` arms of [`render_element`]'s dispatch —
/// split out to keep that function under clippy's line-count limit.
fn render_note_glyph(
    elem: &AbsoluteElement,
    content: &AbsoluteContent,
    params: &RenderElementParams,
) -> Vec<SvgElement> {
    let mut glyphs = render_note_body(elem, content, params);
    let octave = match content {
        AbsoluteContent::NoteHead {
            fermata: true,
            octave,
            ..
        } => Some(*octave),
        AbsoluteContent::Rest { fermata: true, .. } => Some(0),
        _ => None,
    };
    if let Some(octave) = octave {
        glyphs.extend(glyph_renderers::render_fermata(
            elem,
            octave,
            params.notes_font_size,
            params.note_number_width,
        ));
    }
    glyphs
}

fn render_note_body(
    elem: &AbsoluteElement,
    content: &AbsoluteContent,
    params: &RenderElementParams,
) -> Vec<SvgElement> {
    let RenderElementParams {
        notes_font_size,
        note_dash_font_size,
        note_number_width,
        notes_style,
        note_dash_style,
        ..
    } = params;
    match content {
        AbsoluteContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
            head_label,
            ..
        } => render_note_head(
            elem,
            &NoteHeadGlyph {
                pitch,
                accidental,
                head_label: head_label.as_ref(),
            },
            *octave,
            &DotState::new(*dotted, *double_dotted),
            &NoteRenderParams {
                base_font_size: notes_font_size,
                note_number_width,
                bold: notes_style.bold,
                italic: notes_style.italic,
                underline: notes_style.underline,
                font_family: notes_style.font_family,
            },
        ),
        AbsoluteContent::Rest {
            dotted,
            double_dotted,
            implicit_fill,
            ..
        } => render_rest(
            elem,
            &DotState::new(*dotted, *double_dotted),
            notes_font_size,
            *implicit_fill,
            *notes_style,
        ),
        AbsoluteContent::NoteDash {
            dotted,
            double_dotted,
        } => render_note_dash(
            elem,
            &DotState::new(*dotted, *double_dotted),
            *note_dash_font_size,
            *note_dash_style,
        ),
        AbsoluteContent::LowOctaveDots { count } => {
            render_low_octave_dots(elem, *count, notes_font_size, note_number_width)
        }
        // `render_element`'s outer match exhaustively lists every variant
        // routed to `render_note_glyph` (just `NoteHead`/`Rest`/`NoteDash`/
        // `LowOctaveDots`),
        // so a future `AbsoluteContent` variant fails to compile there
        // before it could ever reach this arm.
        _ => Vec::new(),
    }
}

/// The glyph-shaped variants of [`render_element`]'s dispatch that need only
/// `row_height`/`notes_font_size` (no per-kind bold/italic/underline style) —
/// split out to keep `render_element` under clippy's line-count limit.
fn render_simple_glyph(
    elem: &AbsoluteElement,
    content: &AbsoluteContent,
    row_height: &f32,
    notes_font_size: &f32,
) -> Vec<SvgElement> {
    match content {
        AbsoluteContent::MultiMeasureRest { count, width } => {
            render_multi_measure_rest(elem, *count, *width, row_height, notes_font_size)
        }
        AbsoluteContent::Underline { width, level: _ } => render_underline(elem, width),
        AbsoluteContent::TieOrSlur { kind: _, width } => {
            render_tie_or_slur(elem, width, row_height)
        }
        AbsoluteContent::TupletBracket { label, width } => {
            render_tuplet_bracket(elem, label, *width, row_height, notes_font_size)
        }
        AbsoluteContent::BarLine { height, kind } => render_bar_line(elem, height, kind),
        AbsoluteContent::HorizontalLine { width } => render_horizontal_line(elem, width),
        // See the matching comment in `render_note_glyph`: `render_element`'s
        // outer match exhaustively routes this fixed set of variants here.
        _ => Vec::new(),
    }
}
