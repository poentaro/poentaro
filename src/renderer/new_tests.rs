use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ArcKind, BarLineKind};
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, TextAnchor, TextSpan,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(super) fn cfg() -> RenderConfig {
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

pub(super) fn cfg_with_directive_row_offset(offset: Offset) -> RenderConfig {
    RenderConfig {
        directive_row_offset: offset,
        ..cfg()
    }
}

pub(super) fn bpm_span() -> TextSpan {
    TextSpan {
        content: "120".to_string(),
        bold: false,
        italic: false,
        underline: false,
        font_size: 12.0,
    }
}

pub(super) fn make_page(content: AbsoluteContent) -> AbsolutePage {
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
fn note_head_produces_text_element() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let docs = render_new(&[page], &cfg());
    assert_eq!(docs.len(), 1);
    let has_text = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "1"));
    assert!(has_text);
}

#[test]
fn bar_line_produces_vertical_line() {
    let page = make_page(AbsoluteContent::BarLine {
        height: 60.0,
        kind: BarLineKind::Single,
    });
    let docs = render_new(&[page], &cfg());
    let has_line = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Line { .. }));
    assert!(has_line);
}

#[test]
fn final_bar_line_produces_thin_then_thick_line_ending_at_x() {
    let page = make_page(AbsoluteContent::BarLine {
        height: 60.0,
        kind: BarLineKind::Final,
    });
    let x = page.elements[0].x;
    let docs = render_new(&[page], &cfg());
    let stroke_width = |e: &SvgElement| match e.kind {
        SvgKind::Line { stroke_width, .. } => stroke_width,
        _ => panic!("expected only lines, got {e:?}"),
    };
    let [thin, thick] = &docs[0].elements[..] else {
        panic!("expected two lines, got {:?}", docs[0].elements);
    };
    assert!(thin.x < thick.x && stroke_width(thin) < stroke_width(thick));
    assert!((thick.x + stroke_width(thick) / 2.0 - x).abs() < 0.01);
}

#[test]
fn tie_produces_path() {
    let page = make_page(AbsoluteContent::TieOrSlur {
        kind: ArcKind::Slur,
        width: 40.0,
    });
    let docs = render_new(&[page], &cfg());
    let has_path = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Path { .. }));
    assert!(has_path);
}

#[test]
fn rest_produces_zero_text() {
    let page = make_page(AbsoluteContent::Rest {
        dotted: false,
        double_dotted: false,
        implicit_fill: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_zero = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "0"));
    assert!(has_zero);
}

#[test]
fn sharp_accidental_renders_left_of_the_digit_in_the_note_s_own_text_run() {
    // The accidental leads the note digit's own text run (see
    // `render_note_head`) rather than being drawn as its own
    // separately-positioned glyph, and the run starts one accidental-width
    // before `elem.x`, so the digit itself still lands on the note's anchor.
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: crate::ast::parsed::Accidental::Sharp,
        octave: 0,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let note_x = 100.0_f32;
    let config = cfg();
    let docs = render_new(&[page], &config);
    let note_head = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::NoteHead))
        .expect("note head element should be present");
    let sharp_width = crate::font_metrics::accidental_width_for_family(
        config.glyph_font_families.notes,
        &crate::ast::parsed::Accidental::Sharp,
        config.notes_font_size(),
    );
    assert!(sharp_width > 0.0);
    assert!((note_head.x - (note_x - sharp_width)).abs() < 0.001);
    assert!(
        matches!(&note_head.kind, SvgKind::Text { content, anchor, .. }
            if content == "\u{266F}1" && *anchor == TextAnchor::Start),
        "note head should render accidental then digit as one flush-left text run"
    );
}

#[test]
fn chord_symbol_s_leading_accidental_hangs_left_of_its_degree() {
    // Like a note head, a chord symbol leading with a sharp/flat starts its
    // text run one accidental-width before `elem.x`, so its degree lands on
    // the anchor, aligned with other parts' note digits on the same beat.
    let page = make_page(AbsoluteContent::ChordSymbol {
        text: "\u{266D}3m".to_string(),
        dotted: false,
        double_dotted: false,
    });
    let chord_x = 100.0_f32;
    let config = cfg();
    let docs = render_new(&[page], &config);
    let chord = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::ChordSymbol))
        .expect("chord symbol element should be present");
    let flat_width = crate::font_metrics::text_width_for_family(
        config.glyph_font_families.chords,
        "\u{266D}",
        config.chords_font_size(),
    );
    assert!(flat_width > 0.0);
    assert!((chord.x - (chord_x - flat_width)).abs() < 0.001);
}

#[test]
fn chord_symbol_without_leading_accidental_draws_at_its_anchor() {
    let page = make_page(AbsoluteContent::ChordSymbol {
        text: "1/\u{266F}4".to_string(),
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    let chord = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::ChordSymbol))
        .expect("chord symbol element should be present");
    assert_eq!(chord.x, 100.0);
}

#[test]
fn note_dash_renders_at_its_own_font_size_not_notes_font_size() {
    // note_dash and notes are configured with distinct font sizes here so
    // the assertion can't pass by coincidence: the dash must use
    // `note_dash_font_size`, never fall back to `notes_font_size`.
    let config = RenderConfig {
        notes_font_size: 18,
        note_dash_font_size: 30,
        ..cfg()
    };
    let page = make_page(AbsoluteContent::NoteDash {
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &config);
    let dash = docs[0]
        .elements
        .iter()
        .find(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "\u{2014}"))
        .expect("note dash element should be present");
    assert!(
        matches!(&dash.kind, SvgKind::Text { font_size, .. } if *font_size == 30.0),
        "note dash should render at note_dash_font_size (30), not notes_font_size (18)"
    );
}

#[test]
fn upper_octave_note_produces_dot_glyph() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 1,
        dotted: false,
        double_dotted: false,
        head_label: None,
    });
    let docs = render_new(&[page], &cfg());
    let has_dot = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Circle { .. }));
    assert!(
        has_dot,
        "upper octave note should produce an octave dot glyph"
    );
}

#[test]
fn dotted_note_produces_dot_circles_not_appended_text() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 0,
        dotted: true,
        double_dotted: true,
        head_label: None,
    });
    let docs = render_new(&[page], &cfg());
    let note_head = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::NoteHead) && matches!(e.kind, SvgKind::Text { .. }))
        .expect("note head text element should be present");
    assert!(
        matches!(&note_head.kind, SvgKind::Text { content, .. } if content == "1"),
        "digit's own text run should not carry the augmentation dot(s) as appended characters"
    );
    let dot_circle_count = docs[0]
        .elements
        .iter()
        .filter(|e| {
            e.variant == Some(SvgVariant::NoteHead) && matches!(e.kind, SvgKind::Circle { .. })
        })
        .count();
    assert_eq!(
        dot_circle_count, 2,
        "double-dotted note should produce exactly two augmentation dot circles"
    );
}
