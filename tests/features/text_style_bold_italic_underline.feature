Feature: Every TextStyle kind's bold/italic/underline toggles render

  Each of the 13 text-style kinds (see text_style_metadata_syntax.feature)
  accepts `bold`/`italic`/`underline` toggles on its metadata object. This
  must produce a real rendering effect for every kind, not just for `notes`
  — see `tests/text_style_bold_italic_underline_cucumber.rs`, which runs
  each scenario's `metadata` override through the full public
  `render_svgs_from_source` entry point and inspects the raw serialized SVG
  for the configured `<kind>`'s own text element, asserting it carries
  `font-weight="bold"`, `font-style="italic"`, and
  `text-decoration="underline"`.

  `sequence` is included deliberately: its toggles previously had no
  rendering effect at all (the `sequence` summary line's label spans were
  hardcoded to reuse `section_label`'s style) — this suite is a regression
  test for that fix, alongside the `notes` kind the original bug report
  named as the one that *did* work.

  Scenario Outline: <kind>'s bold/italic/underline toggles style its rendered text
    Given "metadata" sets "<kind>" to "{ bold: yes, italic: yes, underline: yes }"
    And a minimal score exercising <kind>
    When it is rendered to SVG
    Then the <kind> text renders bold, italic, and underlined

    Examples:
      | kind           |
      | title          |
      | subtitle       |
      | author         |
      | sequence       |
      | part_legend    |
      | measure_number |
      | section_label  |
      | page_number    |
      | part_label     |
      | lyrics         |
      | notes          |
      | chords         |
      | note_dash      |
