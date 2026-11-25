use crate::ast::parsed::Offset;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage};
use crate::render_config::RenderConfig;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_types::{SvgKind, SvgVariant};

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 16,
        lyrics_font_size: 18,
        notes_font_size: 18,
        note_dash_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
        measure_number_font_size: 10,
        section_label_font_size: 12,
        part_label_font_size: 12,
        page_number_font_size: 18,
        lyric_click_target_padding_pt: 12,
        notes_vertical_padding_pt: 0,
        section_label_vertical_padding_pt: 0,
        page_number_vertical_padding_pt: 0,
        notes_horizontal_padding_pt: 4,
        chords_horizontal_padding_pt: 4,
        lyrics_horizontal_padding_pt: 4,
        note_dash_horizontal_padding_pt: 4,
        ..Default::default()
    }
}

fn make_page(content: AbsoluteContent) -> AbsolutePage {
    AbsolutePage {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![AbsoluteElement {
            x: 100.0,
            y: 200.0,
            content,
        }],
    }
}

#[test]
fn implicit_fill_rest_draws_a_vector_glyph_instead_of_zero_text() {
    // An omitted part's filled-in rest must not read as an ordinary written
    // `0` — see `render_omitted_part_rest` in `glyph_renderers.rs`.
    let page = make_page(AbsoluteContent::Rest {
        dotted: false,
        double_dotted: false,
        fermata: false,
        implicit_fill: true,
    });
    let docs = render_new(&[page], &cfg());

    let has_zero_text = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content.starts_with('0')));
    assert!(!has_zero_text, "should not render as a written \"0\"");

    let omitted_part_rest_lines = docs[0]
        .elements
        .iter()
        .filter(|e| {
            e.variant == Some(SvgVariant::OmittedPartRest) && matches!(e.kind, SvgKind::Line { .. })
        })
        .count();
    assert_eq!(
        omitted_part_rest_lines, 2,
        "the inverted-hat glyph is drawn as two line strokes"
    );
}
