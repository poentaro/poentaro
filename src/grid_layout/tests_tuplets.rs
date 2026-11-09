use crate::ast::parsed::NumberedPitch;
use crate::grid_layout::types::GridContent;

use super::cfg;

#[test]
fn tuplet_notes_align_with_plain_beat_notes_in_sibling_part() {
    // [B] plays plain quarter notes 1 2 3 4; [M] plays a triplet of eighth
    // notes (filling beat 1) followed by plain quarters 2 3 4 on beats 2-4.
    // The plain "2 3 4" in each part falls on the same beats, so they must
    // land on the same grid columns — regardless of the triplet squeezed
    // into the other part's beat 1.
    let source = r#"metadata { title "Test" },

parts ["Melody" [M]: notes, "Beat" [B]: notes],

score """
    time=4/4
    [B] 1 2 3 4
    [M] 3:{1_1_1_} 2 3 4
    """,
"#;
    let score = crate::compile(source, "test.poentaro", &[]).expect("should compile");
    let compile_result = crate::compiler::compile(&score);
    let compile_result = crate::consolidator::consolidate(compile_result);
    let header = crate::grid_layout::types::Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 1,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    };
    let config = cfg();
    let pages =
        crate::grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;

    let note_heads: Vec<(NumberedPitch, u32)> = pages
        .iter()
        .flat_map(|page| page.rows.iter())
        .flat_map(|row| row.elements.iter())
        .filter_map(|element| match &element.content {
            GridContent::NoteHead { pitch, .. } => Some((pitch.clone(), element.column)),
            _ => None,
        })
        .collect();

    for pitch in [
        NumberedPitch::Two,
        NumberedPitch::Three,
        NumberedPitch::Four,
    ] {
        let columns: Vec<u32> = note_heads
            .iter()
            .filter(|(p, _)| *p == pitch)
            .map(|(_, col)| *col)
            .collect();
        assert_eq!(
            columns.len(),
            2,
            "expected NoteHead({pitch:?}) once in [B] and once in [M], got columns={columns:?}"
        );
        assert_eq!(
            columns[0], columns[1],
            "NoteHead({pitch:?}) should be at the same grid column in both parts \
             (same beat), but got columns={columns:?}"
        );
    }
}
