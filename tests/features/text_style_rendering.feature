Feature: Text style components have real rendering effects

  Each of the three TextStyle components (font_size, horizontal_padding_pt,
  vertical_padding_pt — see text_style_metadata_syntax.feature) must
  visibly affect layout/rendering for every kind, including kinds that had
  no equivalent setting before this change:
    - note_dash.font_size: `RenderConfig::note_dash_font_size` (sourced from
      `Metadata::note_dash.font_size`) is threaded through
      `new_renderer.rs`'s `RenderElementParams` into the `NoteDash` arm,
      and into `layout_spacing_weights.rs`'s column-width math, so the
      rendered dash and the column it reserves agree.
    - <kind>.vertical_padding_pt: generalized across three kinds, each in
      its own layout subsystem — `notes` grows the note-head sub-row
      additively (`grid_layout::layout_heights::note_part_sub_row_heights`);
      `section_label` grows the rendered background box
      (`AbsoluteContent::DirectiveLine::label_box_height`, consumed by
      `render_section_label_group`); `page_number` offsets the footer text
      upward from the page's bottom edge instead of growing the footer row
      (which already fills all remaining page height regardless — see
      `resolve_row_element`'s `bottom_padding`); `part_legend` and
      `sequence` grow their header rows by the padding above and below
      (`grid_layout::layout_decoration::make_header_rows`), and so do
      `title` and `subtitle`/`author` (which share one row, taking the
      larger padding).

  Verified end-to-end by `tests/text_style_rendering_cucumber.rs`
  (registered in `Cargo.toml`), which runs each scenario's `metadata`
  overrides through the full pipeline (`compile` → `compiler::compile` →
  `consolidator::consolidate` → `grid_layout::layout` →
  `coordinate_resolver::resolve` → `renderer::new_renderer::render_new`)
  and asserts on the real resolved/rendered output — not just that parsing
  succeeded.

  Scenario: note_dash.font_size changes the rendered dash width
    Given "metadata" sets "note_dash" to "{ font_size: 12 }"
    And the score has a note followed by a dash
    When it is rendered
    Then the rendered dash width at font_size 12 differs from its width at the default note_dash font size

  Scenario Outline: <kind>.vertical_padding_pt adds vertical space without moving other elements
    Given "metadata" sets "<kind>" to "{ vertical_padding_pt: <padding> }"
    And a minimal score using <kind>
    When it is rendered
    Then the <kind> element's rendered box height increases by at least <padding>
    And unrelated elements keep their original position

    Examples:
      | kind          | padding |
      | notes         | 5       |
      | section_label | 8       |
      | page_number   | 4       |
      | part_legend   | 6       |
      | sequence      | 10      |

  Scenario Outline: <kind>.vertical_padding_pt grows the header row holding it
    Given "metadata" sets "<kind>" to "{ vertical_padding_pt: <padding> }"
    And a score with a title, subtitle and author
    When it is rendered
    Then the header row holding the <kind> grows by <padding> above and below

    Examples:
      | kind     | padding |
      | title    | 7       |
      | subtitle | 9       |
      | author   | 5       |
