use super::*;
use itertools::Itertools;

/// The `name="..."` attribute's value, parsed as a number.
fn numeric_attribute(tag: &str, name: &str) -> f32 {
    // Leading space so e.g. `x1` can't match inside some `…x1` attribute;
    // `tags` strips the space before the first attribute, so restore it.
    let tag = format!(" {tag}");
    let prefix = format!(" {name}=\"");
    let start = tag.find(&prefix).expect("attribute should exist") + prefix.len();
    let end = start + tag[start..].find('"').expect("attribute should be closed");
    tag[start..end]
        .parse()
        .expect("attribute should be numeric")
}

/// Every `<{element} ... data-variant="{variant}" ...>` tag in `svg`.
fn tags<'a>(svg: &'a str, element: &str, variant: &str) -> Vec<&'a str> {
    let variant_attribute = format!("data-variant=\"{variant}\"");
    svg.split(&format!("<{element} "))
        .skip(1)
        .map(|rest| &rest[..rest.find('>').unwrap_or(rest.len())])
        .filter(|tag| tag.contains(&variant_attribute))
        .collect()
}

struct Dot {
    x: f32,
    y: f32,
}

struct Underline {
    y: f32,
    x_start: f32,
    x_end: f32,
}

fn render_single_page(score: &str) -> String {
    render_svgs_from_source(score, "test.poentaro", &[])
        .expect("should render")
        .svgs
        .remove(0)
}

fn octave_dots(svg: &str) -> Vec<Dot> {
    tags(svg, "circle", "note-head")
        .into_iter()
        .map(|tag| Dot {
            x: numeric_attribute(tag, "cx"),
            y: numeric_attribute(tag, "cy"),
        })
        .sorted_by(|a, b| a.x.total_cmp(&b.x))
        .collect()
}

fn underlines(svg: &str) -> Vec<Underline> {
    tags(svg, "line", "underline")
        .into_iter()
        .map(|tag| Underline {
            y: numeric_attribute(tag, "y1"),
            x_start: numeric_attribute(tag, "x1"),
            x_end: numeric_attribute(tag, "x2"),
        })
        .collect()
}

/// The lowest underline drawn beneath the note whose center is at `x`.
fn lowest_underline_under(underlines: &[Underline], x: f32) -> Option<f32> {
    underlines
        .iter()
        .filter(|underline| (underline.x_start..=underline.x_end).contains(&x))
        .map(|underline| underline.y)
        .max_by(f32::total_cmp)
}

#[test]
fn below_octave_dots_hang_beneath_the_notes_own_underlines() {
    let svg = render_single_page(
        r#"metadata { title "t" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1, 5,_ 6,_ 5,= 6,= 7,_ 1
    """,
"#,
    );
    let dots = octave_dots(&svg);
    let underlines = underlines(&svg);
    assert_eq!(dots.len(), 6, "one dot per below-octave note");

    let quarter_note_dot = &dots[0];
    assert_eq!(
        lowest_underline_under(&underlines, quarter_note_dot.x),
        None,
        "the quarter note has no underline"
    );
    let underlined_dots = &dots[1..];
    for dot in underlined_dots {
        let underline_y =
            lowest_underline_under(&underlines, dot.x).expect("every other low note is underlined");
        assert!(
            dot.y > underline_y,
            "dot at x={} (y={}) should sit below its lowest underline (y={underline_y})",
            dot.x,
            dot.y
        );
    }
    assert!(
        underlined_dots.iter().all(|dot| quarter_note_dot.y < dot.y),
        "a note without underlines keeps its dot directly under the digit, \
         above where any underline would be"
    );

    let eighth_note_dot = &dots[1];
    let sixteenth_note_dot = &dots[3];
    assert!(
        sixteenth_note_dot.y > eighth_note_dot.y,
        "a sixteenth note's dot clears both its underlines, so it sits lower \
         than an eighth note's"
    );
}
