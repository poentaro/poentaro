//! The parts a song's singing needs, beyond the ones chosen to play.

use itertools::Itertools;

use crate::ast::grouped::Score;
use crate::ast::parsed::AlternativeLine;

/// `enabled` widened so that what plays can sing:
/// - a part a lyric part is sung on brings that lyric part, or it would sing
///   no words;
/// - a lyric part none of whose singers is enabled brings its `visible`
///   singers, so playing selected lyrics alone sings them.
///
/// `None` (every part) stays `None`, as does `visible`'s meaning: every part
/// is visible.
pub fn singing_tracks(
    score: &Score,
    enabled: Option<&[String]>,
    visible: Option<&[String]>,
) -> Option<Vec<String>> {
    let enabled = enabled?;
    let is_visible = |name: &String| visible.is_none_or(|visible| visible.contains(name));
    let widened = lyric_parts(score).into_iter().flat_map(|lyric| {
        if lyric.singers.iter().any(|singer| enabled.contains(singer)) {
            vec![lyric.name]
        } else if enabled.contains(&lyric.name) {
            lyric.singers.into_iter().filter(is_visible).collect()
        } else {
            Vec::new()
        }
    });
    Some(enabled.iter().cloned().chain(widened).unique().collect())
}

struct LyricPart {
    name: String,
    singers: Vec<String>,
}

/// Every lyric part in `score` with the parts it is sung on, by their base
/// names (an alternative line is its part).
fn lyric_parts(score: &Score) -> Vec<LyricPart> {
    score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter())
        .filter_map(|part| {
            let lyrics = part.slice().lyrics.as_ref()?;
            Some(LyricPart {
                name: base_name(part.name()?),
                singers: lyrics.sung_on.iter().map(|name| base_name(name)).collect(),
            })
        })
        .unique_by(|lyric| lyric.name.clone())
        .collect()
}

fn base_name(name: &str) -> String {
    AlternativeLine::of(name).map_or_else(|| name.to_string(), |line| line.base)
}
