use crate::renderer::new_types::{SvgElement, Tag};

use super::{escape_xml, serialize_element};

pub(super) fn serialize_group(out: &mut String, children: &[SvgElement], tag: &Option<Tag>) {
    match tag {
        Some(Tag::Measure { index, end, .. }) => {
            out.push_str(&format!(
                r#"<g data-tag="measure" data-measure-index="{index}" data-measure-index-end="{end}">"#
            ));
        }
        Some(Tag::BarNumber { index, end }) => {
            out.push_str(&format!(
                r#"<g data-tag="bar-number" data-measure-index="{index}" data-measure-index-end="{end}">"#
            ));
        }
        Some(Tag::SectionLabel { label }) => {
            out.push_str(&format!(
                r#"<g data-tag="section-label" data-section-label="{}" style="cursor:pointer">"#,
                escape_xml(label)
            ));
        }
        Some(Tag::Note {
            source_part_index,
            note_id,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="note" data-part-index="{source_part_index}" data-note-id="{note_id}">"#
            ));
        }
        Some(Tag::PartLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="part-label" data-part-index="{source_part_index}" data-measure-index-start="{measure_index_start}" data-measure-index-end="{measure_index_end}" style="cursor:pointer">"#
            ));
        }
        Some(Tag::Lyric {
            source_part_index,
            note_id,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="lyric" data-part-index="{source_part_index}" data-note-id="{note_id}">"#
            ));
        }
        Some(Tag::LyricLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="lyric-label" data-part-index="{source_part_index}" data-measure-index-start="{measure_index_start}" data-measure-index-end="{measure_index_end}" style="cursor:pointer">"#
            ));
        }
        Some(Tag::GuitarDiagram { frets, base_fret }) => {
            out.push_str(&format!(
                r#"<g data-tag="guitar-diagram" data-guitar-frets="{frets}" data-guitar-base-fret="{base_fret}">"#
            ));
        }
        Some(Tag::BarLine {
            measure_index_next,
            measure_index_prev,
        }) => {
            out.push_str("<g data-tag=\"bar-line\"");
            if let Some(next) = measure_index_next {
                out.push_str(&format!(r#" data-measure-index-next="{next}""#));
            }
            if let Some(prev) = measure_index_prev {
                out.push_str(&format!(r#" data-measure-index-prev="{prev}""#));
            }
            out.push_str(" style=\"cursor:pointer\">");
        }
        None => {
            out.push_str("<g>");
        }
    }
    for child in children {
        serialize_element(child, out);
    }
    out.push_str("</g>");
}
