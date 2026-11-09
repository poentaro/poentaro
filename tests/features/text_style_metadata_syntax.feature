Feature: Text style metadata object syntax

  Every text kind (title, subtitle, author, sequence, part_legend,
  measure_number, section_label, page_number, part_label, lyrics, notes,
  chords, note_dash) is configured as a single object-valued metadata key
  with the same three components: font_size, horizontal_padding_pt,
  vertical_padding_pt (`Metadata::TextStyle` /
  `ParsedMetadata::TextStyle`, src/ast/grouped.rs / src/ast/parsed/mod.rs).

  Syntax: `<kind> = { field: value, field: value }` — unquoted keys, `{}`
  and `:` as separators, comma-separated fields, any subset of the three
  fields in any order.

  The old flat per-component keys (lyrics_font_size,
  lyric_click_target_padding_pt, notes_horizontal_padding_pt, etc.) are
  retired outright — no backward compatibility is preserved. Those keys
  now fail as unknown metadata fields (regression coverage for that lives
  in metadata_parser_tests.rs, not here).

  Scenario Outline: A single component parses into the kind's TextStyle
    Given "metadata" sets "<kind>" to "{ <field>: <value> }"
    When it is compiled
    Then the resolved <kind> TextStyle has <field> equal to <value>

    Examples:
      | kind          | field                 | value |
      | title         | font_size             | 32    |
      | subtitle      | font_size             | 20    |
      | author        | font_size             | 16    |
      | sequence      | font_size             | 14    |
      | part_legend   | font_size             | 14    |
      | measure_number| font_size             | 12    |
      | section_label | font_size             | 16    |
      | page_number   | font_size             | 20    |
      | part_label    | font_size             | 18    |
      | lyrics        | font_size             | 18    |
      | lyrics        | horizontal_padding_pt | 4     |
      | lyrics        | vertical_padding_pt   | 6     |
      | notes         | font_size             | 24    |
      | notes         | horizontal_padding_pt | 3     |
      | notes         | vertical_padding_pt   | 5     |
      | chords        | font_size             | 16    |
      | chords        | horizontal_padding_pt | 3     |
      | note_dash     | font_size             | 12    |
      | note_dash     | horizontal_padding_pt | 2     |

  Scenario: Multiple components on one kind combine into a single object
    Given "metadata" sets "chords" to "{ font_size: 16, horizontal_padding_pt: 3 }"
    When it is compiled
    Then the resolved chords TextStyle has font_size equal to 16
    And the resolved chords TextStyle has horizontal_padding_pt equal to 3

  Scenario: Field order inside the object does not matter
    Given "metadata" sets "lyrics" to "{ vertical_padding_pt: 6, font_size: 18 }"
    When it is compiled
    Then the resolved lyrics TextStyle has font_size equal to 18
    And the resolved lyrics TextStyle has vertical_padding_pt equal to 6

  Scenario: An unset component falls back to its row-height-derived default
    Given "metadata" sets "row_height" to "40"
    And "metadata" sets "lyrics" to "{ horizontal_padding_pt: 4 }"
    When it is compiled
    Then the resolved lyrics TextStyle's font_size equals the default lyrics font size for row_height 40

  Scenario: An unknown field inside the object is a recoverable error
    Given "metadata" sets "lyrics" to "{ bogus_field: 4 }"
    When it is compiled
    Then compiling reports an unknown metadata field "lyrics style bogus field"

  Scenario Outline: A malformed object literal is a recoverable parse error
    Given "metadata" sets "notes" to "<malformed>"
    When it is compiled
    Then compiling reports a metadata parse error on the "notes" line

    Examples:
      | malformed        |
      | { font_size: 16  |
      | font_size: 16 }  |
      | { font_size: }   |
