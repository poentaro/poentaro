use crate::ast::parsed::Soundfont;
use crate::error::{RecoverableError, Span};
use crate::sound_label::{instrument_label, parse_label_program};

/// One GM instrument catalog entry. `name` is the bare instrument name
/// (e.g. `"Violin"`); the quoted `"N: Name"` form is built only by
/// [`crate::sound_label`].
#[derive(serde::Deserialize)]
pub struct InstrumentInfo {
    pub name: String,
    pub program: u8,
    pub category: String,
    pub source: String,
    pub role: String,
    pub articulation: String,
}

fn fuzzy_score(query: &str, target: &str) -> u32 {
    let q = query.to_lowercase();
    let t = target.to_lowercase();
    if t.contains(q.as_str()) {
        return 1000;
    }
    let mut score: u32 = 0;
    let mut chars_q = q.chars().peekable();
    let mut consecutive: u32 = 0;
    for tc in t.chars() {
        if chars_q.peek() == Some(&tc) {
            chars_q.next();
            score += 1 + consecutive * 2;
            consecutive += 1;
        } else {
            consecutive = 0;
        }
    }
    if chars_q.peek().is_none() {
        score
    } else {
        0
    }
}

fn instrument_fuzzy_score(query: &str, instrument: &InstrumentInfo) -> u32 {
    [
        fuzzy_score(query, &instrument_label(instrument)),
        fuzzy_score(query, &instrument.category),
        fuzzy_score(query, &instrument.source),
        fuzzy_score(query, &instrument.role),
        fuzzy_score(query, &instrument.articulation),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}

pub(crate) fn validate_soundfont(
    inner: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
    is_percussion: bool,
) -> Option<Soundfont> {
    if !is_percussion
        && !instruments.is_empty()
        && !instruments.iter().any(|i| instrument_label(i) == inner)
    {
        let mut scored: Vec<(&InstrumentInfo, u32)> = instruments
            .iter()
            .filter_map(|instrument| {
                let score = instrument_fuzzy_score(inner, instrument);
                if score > 0 {
                    Some((instrument, score))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|left, right| right.1.cmp(&left.1));
        let suggestions: Vec<String> = scored
            .iter()
            .take(5)
            .map(|(instrument, _)| instrument_label(instrument))
            .collect();
        errors.push(RecoverableError::parts_unknown_soundfont(
            span,
            inner,
            suggestions,
        ));
    }

    let soundfont = parse_label_program(inner).map(Soundfont);
    if soundfont.is_none() {
        errors.push(RecoverableError::parts_invalid_columns(span, inner));
    }
    soundfont
}
