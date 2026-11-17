mod click_targets;
mod content_conversion;
mod directive_line_conversion;
mod flush_left;
mod highlight_click_targets;
mod highlights;
mod page_geometry;
mod post_arc_conversion;
pub mod resolve;
mod rest_run;
mod span_marking;
pub use resolve::{resolve, ElementPaddings, LabelFontSizes, LyricFontSizes, ResolveFontSizes};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bar_number_click_target;
#[cfg(test)]
mod tests_basic_resolve;
#[cfg(test)]
mod tests_directive_line_gap;
#[cfg(test)]
mod tests_highlights;
#[cfg(test)]
mod tests_left_edge_alignment;
#[cfg(test)]
mod tests_lyrics;
#[cfg(test)]
mod tests_multi_measure_rest;
#[cfg(test)]
mod tests_sequence_line;
