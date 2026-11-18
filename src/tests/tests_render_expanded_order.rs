use super::*;
use itertools::Itertools;
use renderer::new_types::{SvgDocument, SvgElement, SvgKind, Tag};

const SOURCE: &str = r#"
metadata { title "Expanded sequence" },
parts ["Soprano" [S]: notes, "Alto" [A]: notes],
sequence [Chorus WITHOUT [S], Verse, Chorus WITH [S] 2],
score """
    label="Verse"
    [S] 1 2 3 4
    [A] 5 6 7 1

    label="Chorus"
    [S] 1 1 1 1
    [S] 2 2 2 2
    [A] 3 3 3 3
    """,
"#;

fn render(order: ScoreOrder) -> Vec<SvgDocument> {
    render_documents_from_source_with_visibility(
        SOURCE,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        RenderView {
            score_order: order,
            ..RenderView::default()
        },
    )
    .unwrap()
    .documents
}

fn tags(elements: &[SvgElement]) -> Vec<&Tag> {
    elements
        .iter()
        .flat_map(|element| match &element.kind {
            SvgKind::Group { tag, children } => tag.iter().chain(tags(children)).collect(),
            _ => vec![],
        })
        .collect()
}

fn note_targets(elements: &[SvgElement]) -> Vec<(usize, usize)> {
    elements
        .iter()
        .flat_map(|element| match &element.kind {
            SvgKind::Group {
                tag:
                    Some(Tag::Note {
                        source_part_index,
                        note_id,
                    }),
                children,
            } if children
                .iter()
                .any(|child| matches!(child.kind, SvgKind::PlaybackCursorRect { .. })) =>
            {
                vec![(*source_part_index, *note_id)]
            }
            SvgKind::Group { children, .. } => note_targets(children),
            _ => vec![],
        })
        .collect()
}

#[test]
fn expanded_order_preserves_written_measure_and_note_ids() {
    let written = render(ScoreOrder::Written);
    let expanded = render(ScoreOrder::Expanded);
    let expanded_tags = expanded.iter().flat_map(|page| tags(&page.elements));
    let measures = expanded_tags
        .clone()
        .filter_map(|tag| match tag {
            Tag::Measure { index, .. } => Some(*index),
            _ => None,
        })
        .collect_vec();
    assert_eq!(measures, vec![1, 0, 1]);
    let labels = expanded_tags
        .clone()
        .filter_map(|tag| match tag {
            Tag::SectionLabel { label } => Some(label.as_str()),
            _ => None,
        })
        .collect_vec();
    assert_eq!(labels, vec!["Chorus", "Verse", "Chorus"]);

    let note_keys = |documents: &[SvgDocument]| {
        documents
            .iter()
            .flat_map(|page| note_targets(&page.elements))
            .collect_vec()
    };
    let written_notes = note_keys(&written);
    let expanded_notes = note_keys(&expanded);
    assert_eq!(expanded_notes.len(), 20);
    assert!(expanded_notes.iter().all(|key| written_notes.contains(key)));
    assert_eq!(expanded_notes.into_iter().unique().count(), 16);
    let svg = serializer::serialize(&expanded, None).concat();
    assert!(!svg.contains("Sequence: "));
}

#[test]
fn expanded_staff_order_keeps_omitted_parts_as_rests_and_maps_measure_ids() {
    let document = render_mei_in_order(
        SOURCE,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        mei_export::ClefChoice::Violin,
        ScoreOrder::Expanded,
    )
    .unwrap();
    let indices = document
        .measures
        .iter()
        .map(|measure| measure.measure_index)
        .collect_vec();
    assert_eq!(indices, vec![1, 0, 1]);
    let staves = document.mei.matches("<staffDef ").count();
    assert_eq!(staves, 2);
    assert_eq!(document.mei.matches("<mRest/>").count(), 1);
    assert!(document.mei.contains("xml:id=\"measure-2\" n=\"3\""));
}
