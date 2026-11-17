import type {
  MetadataEdit,
  MetadataFlagKey,
  MetadataNumberKey,
  TextStyleKind,
} from '../poentaroWasm'

/** Markdown help text shown in the field-help modal, keyed by the generated
 * wasm key types: the 13 `TextStyleKind`s (shared by both the content row
 * and its style row, for `title`/`subtitle`/`author`, which have both), the
 * numeric fields, the checkbox fields, and `directive-row-offset` —
 * describing precisely which rendering/layout aspects each Edit Metadata
 * field affects. */
export const metadataFieldHelp: Record<
  | TextStyleKind
  | MetadataNumberKey
  | MetadataFlagKey
  | Extract<MetadataEdit['tag'], 'directive-row-offset'>,
  string
> = {
  title: `Rendered as the large heading at the top of the score's first page.`,

  subtitle: `Rendered as a smaller line beneath the title in the header.`,

  author: `Rendered in the header, below the title/subtitle.`,

  sequence: `Style of the \`sequence\` summary line rendered near the top of the score.`,

  'part-legend': `Style of the part-name legend entries shown in the header.`,

  'measure-number': `Style of each measure's bar number.`,

  'section-label': `Style of an inline section label (the \`label="..."\` on a measure's directive line).`,

  'part-label': `Style of a part's row label (e.g. "Soprano"), shown at the start of each system row.

See **Part Label Width** below for the column's reserved width.`,

  'part-label-width-pt': `Fixed width (points) of the part-label column at the start of each system, shared by every system in the score regardless of how many measures/columns that system's music needs.`,

  'page-margin-pt': `Blank space (points) on every side of the page, between the paper's edge and the header, music and page number. A wider margin also narrows each system, so measures may wrap differently.`,

  'page-number': `Style of the page number shown in the footer.

**V. Padding** pushes the page number upward from the page's bottom edge, without moving anything else.`,

  'running-title': `Style of the small title shown at the top of every page after the first (the subtitle and author appear on the first page only).`,

  lyrics: `Style of lyric syllable text under notes.

**Font Size** also affects how far a syllable is allowed to shift horizontally to avoid overlapping its neighbors. **H. Padding** widens spacing between syllables. **V. Padding** is extra padding around a lyric syllable's hover/click-target box, on top of the lyric font's own measured height.`,

  notes: `Style of note heads, rests, percussion hits, and tuplet brackets.

**Font Size** also affects the width allotted to a note column, since the note column's width is measured against these glyphs' own font (see **Font**). **H. Padding** is also used for the multi-measure-rest bar's end insets and the tie/slur/underline/tuplet-bracket markings, which all key off a note column. **V. Padding** adds vertical space around the note-head row.`,

  chords: `Style of chord symbol text.

**Font Size** also affects the width allotted to a chord symbol's column, measured against its own font (see **Font**).`,

  'note-dash': `Style of a note dash (the sustain-beat \`-\` extension).

**Font Size** scales the rendered dash's width, measured against its own font (see **Font**).`,

  'row-height': `Vertical spacing (points) of one part row.

Affects:
- Note heads and rests
- Octave and duration dots
- Tie/slur arc height
- Bar-line and multi-measure-rest thickness
- The default **Font Size** of every text style whose default is a percentage of the row height (unless set explicitly)`,

  'max-measures-per-system': `Maximum number of measures placed on one system (row) before wrapping to a new system line.`,

  'note-number-width': `Offset (points) used to:
- Place an accidental (♯/♭) away from its note head
- Place a duration dot away from its note head
- Set the width of the underline drawn beneath eighth/sixteenth notes

**Has no visible effect** on a note with no accidental, no dot, and no underline.

Does **not** change the spacing between note columns — that comes from the available page width instead.`,

  'parts-list-columns': `Number of columns used to lay out the part-name legend shown in the header.`,

  'merge-duplicate-measures-across-parts': `When on, measures with identical content across different parts are drawn as a single merged row instead of one row per part.

Can be overridden from a specific measure onward with a
\`merge_duplicate_measures_across_parts=yes\`/\`no\` directive line.`,

  'hide-resting-parts': `When on, a part that is entirely rests in a measure is omitted from that measure's system whenever at least one other part has real content.

Can be overridden from a specific measure onward with a
\`hide_resting_parts=yes\`/\`no\` directive line.`,

  'hide-system-dividers': `When on, the horizontal divider line normally drawn between consecutive systems (rows of measures) is omitted.`,

  'directive-row-offset': `Translation (points, \`"x y"\`) applied to every rendered directive row — bar number, section label, key, bpm, and time signature.

Moves that row's text without affecting the layout or spacing of anything else on the page.

Not applied to the \`sequence\` summary header line.`,
}
