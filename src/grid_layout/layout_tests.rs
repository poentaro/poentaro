//! `layout()`'s own test submodules, split out of `layout.rs` to keep that
//! file under the max line-count lint.

#[path = "tests_layout.rs"]
mod tests_layout;

#[path = "tests_layout_directives.rs"]
mod tests_layout_directives;

#[path = "tests_highlight.rs"]
mod tests_highlight;

#[path = "tests_lyric_links.rs"]
mod tests_lyric_links;

#[path = "tests_playback_cursor.rs"]
mod tests_playback_cursor;

#[path = "tests_lyrics_only_part.rs"]
mod tests_lyrics_only_part;

#[path = "tests_chord_lyric_click_targets.rs"]
mod tests_chord_lyric_click_targets;
#[path = "tests_lyric_click_targets.rs"]
mod tests_lyric_click_targets;

#[path = "tests_lone_resting_row_label.rs"]
mod tests_lone_resting_row_label;

#[path = "tests_lyric_highlight_properties.rs"]
mod tests_lyric_highlight_properties;
