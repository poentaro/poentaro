use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::compiler::NoteHeadLabel;
use crate::compositor::types::FontFamily;
use crate::font_metrics;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

/// What a note head reads as: its numbered degree (with any written
/// sharp/flat), or, in the letter-name and harmonica-tab views, `head_label`
/// in its place, or, in the guitar chord view, the degree followed by
/// `head_label`.
pub(in crate::renderer::new_renderer) struct NoteHeadGlyph<'a> {
    pub(in crate::renderer::new_renderer) pitch: &'a NumberedPitch,
    pub(in crate::renderer::new_renderer) accidental: &'a Accidental,
    pub(in crate::renderer::new_renderer) head_label: Option<&'a NoteHeadLabel>,
}

impl NoteHeadGlyph<'_> {
    /// The text run's content: the label, or the accidental and digit
    /// (followed by the label in the guitar chord view).
    pub(super) fn content(&self) -> String {
        let digit = self.pitch.to_digit();
        match self.head_label {
            Some(head_label) if head_label.replaces_degree() => head_label.run_text(digit),
            Some(head_label) => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                head_label.run_text(digit)
            ),
            None => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                digit
            ),
        }
    }

    /// The text run's styling hook: an unplayable label is flagged as such.
    pub(super) fn variant(&self) -> SvgVariant {
        match self.head_label {
            Some(NoteHeadLabel {
                unplayable: true, ..
            }) => SvgVariant::UnplayableNoteHead,
            _ => SvgVariant::NoteHead,
        }
    }

    /// Where the text run starts, given the glyph's anchor `x`. A digit's
    /// accidental leads it, so its run starts one accidental-width earlier; a
    /// label drawn in its place has its accidental trail its letter (`F♯`),
    /// so its run starts at `x` itself.
    pub(super) fn run_x(&self, x: f32, font_family: FontFamily, font_size: f32) -> f32 {
        match self.head_label {
            Some(head_label) if head_label.replaces_degree() => x,
            _ => {
                x - font_metrics::accidental_width_for_family(
                    font_family,
                    self.accidental,
                    font_size,
                )
            }
        }
    }

    /// The arrow above a harmonica tab played with the slide button pressed
    /// in, pointing left and centered over the text run that starts at
    /// `run_x` around the vertical center `y`, where an octave-up dot would
    /// otherwise sit. Nothing for any other note head.
    pub(super) fn slide_arrow(
        &self,
        run_x: f32,
        y: f32,
        font_family: FontFamily,
        font_size: f32,
    ) -> Vec<SvgElement> {
        if !self.head_label.is_some_and(|label| label.slide_button) {
            return Vec::new();
        }
        let center = run_x
            + font_metrics::text_width_for_family(font_family, &self.content(), font_size) / 2.0;
        let arrow_y = y - font_size * 0.8;
        let half_length = font_size * 0.3;
        let head = font_size * 0.15;
        let tip = center - half_length;
        let stroke = |x: f32, y: f32, x2: f32, y2: f32| SvgElement {
            x,
            y,
            variant: Some(SvgVariant::HarmonicaSlide),
            kind: SvgKind::Line {
                x2,
                y2,
                stroke_width: font_size * 0.07,
            },
        };
        vec![
            stroke(center + half_length, arrow_y, tip, arrow_y),
            stroke(tip, arrow_y, tip + head, arrow_y - head),
            stroke(tip, arrow_y, tip + head, arrow_y + head),
        ]
    }
}
