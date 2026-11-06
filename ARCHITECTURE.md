# Architecture

## Pipeline Overview

```
source (&str)
  → [parser]              → ParsedDocument
                             Raw event stream: notes, rests, chords, directives,
                             lyrics syllables per measure. No note measure grouping yet.
  → [grouper]             → Score
                             Notes grouped into measures; lyrics paired to each
                             measure's lyric slots (tie-aware);
                             parts organized into MultiPartMeasure slices;
                             optional `sequence` section resolved into
                             Score.sequence spans.
  → [compiler]            → CompileResult
                             Logical grid: each note/rest assigned to a column,
                             underlines computed, slur spans recorded.
  → [consolidator]        → CompileResult
                             Duplicate rows with
                             identical content suppressed per measure.
  → [grid_layout]         → Vec<GridPage>
                             Grid elements placed into rows with heights and column
                             counts; rows wrapped across pages; slur arcs resolved
                             to same-system or cross-system variants.
  → [coordinate_resolver] → Vec<AbsolutePage>
                             Every element assigned absolute x/y coordinates in
                             points; grid geometry collapsed to flat element list.
  → [renderer]            → Vec<SvgDocument>
                             SVG primitives (Text, Line, Circle, Path) produced
                             for each absolute element.
  → [serializer]          → Vec<String>
                             SVG strings, one per page, ready to write to disk.
```

## Layer Details

### Parser
- Module: `src/parser/`
- Entry: `parser::parse(source: &str, filename: &str) -> Result<ParsedDocument, IrrecoverableError>`
- Anyfix surface syntax (see `syntax.md`): `parser::read_document(input: &str, instruments: &[InstrumentInfo]) -> ReadDocument` reads a document's text through `parser::anyfix::read(source, instruments)` (`src/parser/anyfix/read/`), which decodes `anyfix_expr`'s tree (`parser::anyfix::tree::Sections`) directly, and `parser::parse` builds on it. Text in any other form is read the same way and comes back with syntax errors, so it still opens. `ReadDocument { metadata, metadata_parse_errors, declarations, parts_parse_errors, section_structure_errors, score: ScoreText, presets, sequence, sequence_parse_errors }` is what it produces, and `ScoreText { text, offset }` is the score body the score parser reads, so every span is a span in the file. `document::anyfix_errors(text, instruments) -> Vec<RecoverableError>` lists what doesn't fit a document's shape (`RecoverableErrorKind::AnyfixInvalid`): anything that writes a file from it stops on them, while a render keeps going so a typo doesn't blank the preview. `format_anyfix::format_anyfix(source) -> String` formats a document, and returns it unchanged when `anyfix_errors` finds anything. `source_edit`'s edits go through `source_edit/anyfix_edit.rs`, which rewrites the node it changes and formats that section only, keeping other sections' comments and layout; `parser::anyfix::write` spells the fields and part kinds they write. `symbols::rename_edits_in_text` spells a renamed symbol for where it sits. In the wasm crate, `crates/poentaro-wasm/src/opened_document.rs` has `open_checked`, which turns `anyfix_errors` into diagnostics. See **Anyfix document** in the glossary.
- Presets: `parser::preset_parser::collect_presets(sections, name_of, read) -> ParsedPresets`, called from the anyfix reader with one `finish_preset` per section, fills `ParsedDocument::presets: Vec<ParsedPreset>` and `preset_diagnostics: Vec<Diagnostic>`. `ParsedPreset { name, name_span, header_span, overrides: ParsedMetadata, visible_parts: Option<Vec<AbbreviationReference>>, soloed_parts: Vec<AbbreviationReference>, notation: Option<Notation> }` (`src/ast/parsed/preset.rs`). `ParsedDocument::with_preset(name)` overlays `metadata` with that preset's overrides (`ParsedMetadata::overlaid_with`, field by field; an unknown name changes nothing); `compile_with_preset(source, filename, instruments, preset_name: Option<&str>)` is `compile` with that overlay. See **Preset** in the glossary.
- Key types: `ParsedDocument` (also carries `sequence: Option<SequenceSection>` and `sequence_parse_errors: Vec<RecoverableError>` from the optional `sequence` section — see **Sequence resolution** in the glossary; also carries `abbreviation_references: Vec<AbbreviationReference>` — every `[Abbrev]` key-prefix reference found in `score`, in file order, each `{ abbreviation, span }` with `span` covering just the trimmed abbreviation text (excluding brackets/whitespace) — see **Rename-symbol support**), `ParsedTimedTrack` (also carries `slur_groups: Vec<ParsedSlurGroup>` — every closed `(…)` group on the track, recorded by `GroupStack::close` as each `)` is parsed), `ParsedSlurGroup` (`{ open_paren_span, close_paren_span }` — lets source edits locate a group's delimiters, see **Source editing**), `ParsedScore`, `ScoreEvent` (includes `PercussionHit(ParsedPercussionHit)`; `LabelChange(String)`'s `Spanned` wrapper carries a span narrowed to just the quoted label text, not the whole `label="..."` token), `ParsedNote` (carries `Accidental` for melody notes; tie intent via `tie_to_next_span: Option<Span>` with `tie_to_next()` accessor; also carries `tuplet: Option<TupletInfo>` — see **Tuplet** in the glossary), `ParsedRest` (carries `implicit_fill: bool` — true when this rest was synthesized by `desugar::implicit_fill` to stand in for a declaration absent from this measure group, rather than written by the composer as an explicit `0`; copied through `GroupedRest`/`ElementContent::Rest`/`GridContent::Rest`/`PostArcGridContent::Rest`/`AbsoluteContent::Rest` to `SvgVariant::OmittedPartRest`, which `render_omitted_part_rest` draws as a vector glyph — mirroring the conventional Western "whole rest": a thin horizontal cap line, wider than and sitting flush above, a short solid block — instead of `render_rest`'s ordinary `0` text — see **Omitted-part rest** in the glossary), `ParsedChordNote` (also carries `Accidental` and `tie_to_next_span`), `ParsedPercussionHit` (`ParsedNote` minus pitch/accidental — carries duration, dotted, group membership, tie/slur fields), `TupletInfo` (`{ num, den }` — see **Tuplet** in the glossary), `ParsedMetadata` (key-addressed via `style`/`style_mut`/`number`/`number_mut`/`flag`/`flag_mut`/`text`/`text_mut` — see **Metadata keyword enum** in the glossary), `NumberedPitch`, `Accidental` (`Sharp`/`Flat`/`Natural`; applies to both melody notes and chord notes), `Syllable`, `Soundfont` (vocal/piano/string; selects MIDI channel+program — on a `PartKind::Percussion` part, the number is instead a fixed GM percussion key rather than a GM program number, and is not validated against the melodic instrument catalog), `PartDecl` (carries `soundfont`, `volume`, `octave_offset`, `range_instrument: Option<RangeInstrument>` — see **Range instrument** in the glossary, `kind: PartKind`, `abbreviation_span: Span` — the abbreviation token's own byte span on its declaration line, used by rename-symbol — `PartKind::Percussion` reuses `&[ScoreLineRole::Notes]` for its score line roles), `SequenceSection` (decoded by `src/parser/anyfix/read/sequence.rs` — a list of section-label references, each a `SequenceEntry { label, label_span, part_filter, span }` where `part_filter: Option<PartFilter>` comes from an optional `WITHOUT [...]` or `ONLY [...]` suffix; `PartFilter { kind, parts, part_spans }` with `kind: PartFilterKind` either `Omit` (a `(-abbrev -abbrev ...)` suffix naming part abbreviations to omit from that occurrence's playback, e.g. `Chorus(-S -A2)`) or `Only` (an `(abbrev abbrev ...)` suffix, no leading `-`, naming the only parts to keep for that occurrence — every other declared part is omitted — e.g. `Chorus(S A2)`); a suffix mixing `-abbrev` and `abbrev` tokens is a recoverable error, resolving to `part_filter: None`; `label_span`/`part_spans` are the byte spans of just the label text and each filtered abbreviation (used by rename-symbol) — see **Sequence resolution** in the glossary).

### Grouper
- Module: `src/grouper/`
- Entry: `grouper::group(doc: ParsedDocument) -> Result<Score, IrrecoverableError>`
- Also: `grouper::metadata_defaults(inherited: &ParsedMetadata, own: &ParsedMetadata) -> Metadata` — what each field resolves to when left unset, given the rest of the metadata as written (`own`) on top of what it falls back to (`inherited`: the song's `metadata` when editing a preset, empty otherwise) (each kind's style is found by clearing it and running the same `resolve_metadata`); backs the Edit Metadata modal's placeholders. `Metadata::style(kind)`/`style_mut(kind)` address the resolved per-kind `TextStyle` by `TextStyleKind`.
- Key types: `Score` (carries `sequence: Option<Vec<SequenceSpan>>`, resolved by `resolve_sequence()` in `src/grouper/sequence_resolution.rs` — see **Sequence resolution** in the glossary), `SequenceSpan` (`label`, `start`, `end`, `omit_parts`, `part_filter_display`: an inclusive resolved measure-index range for one `sequence` entry, plus `omit_parts: Vec<String>` — the concrete part abbreviations to omit from that occurrence's MIDI/WAV playback, always an omit-shaped list even when the entry wrote an `Only` filter (resolved to its complement against `parts`) — and `part_filter_display: Option<PartFilterDisplay>` (`{ kind, parts }`, reusing `PartFilterKind`) carrying the suffix as written, for the "Sequence: ..." summary line, `None` when the entry has no suffix — see **Sequence resolution**), `MultiPartMeasure` (carries `bpm`/`key`/`time_signature`/`label` (all non-persisting on this type — see below) plus the always-resolved, persisting `merge_duplicate_measures_across_parts`/`hide_resting_parts` booleans), `PartRow` (Timed), `PartSlice` (carries `soundfont`, `volume`, `octave_offset`, `resolution_multiplier: u32`, copied unchanged from the source `GroupedMeasure::resolution_multiplier`; see **Tuplet** — and `beat_group_size: u32`, copied unchanged from the source `GroupedMeasure::beat_group_size`), `Notes`, `NoteEvent` (includes `Percussion(GroupedPercussionHit)`), `GroupedNote`, `GroupedRest` (carries `implicit_fill: bool`, copied unchanged from `ParsedRest::implicit_fill` — see **Omitted-part rest** in the glossary), `GroupedChordNote` (`GroupedNote`/`GroupedChordNote` use `tie_to_next_span` + `tie_to_next()` accessor; all four `Grouped*` note types also carry `tuplet: Option<TupletInfo>`, copied through unchanged from the corresponding `Parsed*` field — see **Tuplet** in the glossary), `GroupedMeasure` (intermediate: notes + `paired_lyrics: Vec<Syllable>` — the syllables of a lyric part, tie-paired against its target's notes — plus `resolution_multiplier: u32` — see **Tuplet** — and `beat_group_size: u32`, the quarter-beat width of one beam group under the time signature in effect for that measure: `4` for simple meters, `6` for compound meters (6/8, 9/8, 12/8, ...), computed by `PartGrouper::beat_group_size` from the time signature tracked via `TimeSignatureChange` events and unscaled by `resolution_multiplier`). `PartSlice.kind: PartKind` (`Chords`/`Notes`/`Percussion`/`Lyrics { target_part_index }`) says what a part is; a lyric part (`lyrics [X]`) is its own `PartSlice` with `PartSlice.lyrics: Option<LyricsSlice>` (`LyricsSlice { target_name, target_events, syllables, targets_chords }` — the target's events this measure, which fix where syllables sit, plus the tie-aware syllables, one per lyric slot from `lyric_slots::measure_lyric_slots`; for a `chords` target (`targets_chords`) the syllables are instead the raw bracketed/unbracketed items — see **Bracketed lyric item**) and no notes of its own — see **Lyric part** in the glossary.
- `src/tuplet.rs`'s `resolution_multiplier_of(events: &[Spanned<ScoreEvent>]) -> u32` / `apply_resolution_multiplier(events: Vec<Spanned<ScoreEvent>>, resolution_multiplier: u32) -> Vec<Spanned<ScoreEvent>>` is a stage-agnostic pair used by both the parser and the grouper (see **Tuplet** in the glossary for the parser-side use). On the grouper side, `part_grouper_group::group_timed_track` calls both once per measure (one `ParsedMeasureSlot::Real`'s event list), before that measure's events reach `PartGrouper`: `resolution_multiplier_of` scans for `TupletInfo` tags and computes `resolution_multiplier = lcm(every tagged event's num)`, then `apply_resolution_multiplier` multiplies every event's `duration` by it (plain notes too, so the measure's grid stays proportionally consistent) and further multiplies tuplet-tagged events by `den / num` (exact, since `resolution_multiplier` is a multiple of `num`). `PartGrouper::begin_measure_slot(resolution_multiplier)` (called once per `ParsedMeasureSlot::Real`, before its rescaled events are pushed) scales the capacity check (`PartGrouper::effective_capacity() = capacity * resolution_multiplier`) so a measure's flush boundary still lands correctly post-rescale; the multiplier resets back to `1` only once that measure actually flushes (`flush_measure`), since the accumulating measure isn't always flushed by the time all its events have been pushed (e.g. the score's trailing measure, flushed later by `finish`). `resolution_multiplier` defaults to `1` and the rescale is then a no-op, so non-tuplet music is unaffected. `midi::midi_notes::duration_to_ticks(quarter_beats, multiplier)` divides `multiplier` back out when converting a (possibly rescaled) duration to MIDI ticks; grid-column pixel width needs no equivalent division — see **Tuplet**.
- `range_validation::validate_ranges(score: &mut Score, declarations: &[PartDecl])` (`src/grouper/range_validation.rs`) runs after `validate_ties` and pushes a `Diagnostic::Warning` (`WarningKind::NoteOutOfRange`) into a measure's `diagnostics` for each note outside its part's **Range instrument**, so it reaches the editor's warning zone and the preview's error highlight like any measure diagnostic. Sounding pitch comes from `src/sounding_pitch.rs` (`resolve_midi_note_with_accidental` and friends — ungated, unlike the `midi` feature that re-exports them).
- `merge_duplicate_measures_across_parts`/`hide_resting_parts` on `MultiPartMeasure` are resolved (not `Option`) by `DirectiveGrouper` (`src/grouper/directive_grouper.rs`): seeded from `Metadata::merge_duplicate_measures_across_parts`/`Metadata::hide_resting_parts` (the `metadata` default) and overridden from a given measure onward by a `merge_duplicate_measures_across_parts=`/`hide_resting_parts=` directive line, persisting like `bpm`/`key`/`time` until the next override — unlike those three, they carry the fully resolved value on every measure rather than `Some` only where changed, since downstream consumers (`compiler::compile_measure`, `consolidator::consolidate_block`) need a plain value per call and have no rendering reason to know whether it just changed.

### Compiler
- Module: `src/compiler/`
- Entry: `compiler::compile(score: &Score) -> CompileResult`
- Letter names and harmonica tabs: `compiler::label_note_heads(result: CompileResult, score: &Score, labels: NoteLabels) -> CompileResult` (`src/compiler/letter_names.rs`) runs right after `compile` in `document_render`. `NoteLabels::Degrees` (the default) returns `result` unchanged; `NoteLabels::LetterNames` sets every `ElementContent::NoteHead`'s `head_label: Option<NoteHeadLabel>` (`NoteHeadLabel { text, unplayable, slide_button, placement }`, in `src/compiler/types.rs`; `placement: HeadLabelPlacement::InPlaceOfDegree` for every view but the guitar chord view, see below) to its pitch spelled against the key in effect at its block (`pitch_description::spelling::SpelledPitch::engraved_name`, e.g. `F♯`), and rewrites every `ElementContent::ChordSymbol`'s `text` from its `degrees: ChordDegrees` (`src/compiler/chord_degrees.rs` — the chord's root, quality and slash bass as written; `degree_symbol()` gives the numbered text `♯1m⁷/5`, `letter_symbol(tonic)` the letter-name text `F♯m⁷/C♯`). Since a chord's width, accidental lead and bearing all derive from its `text`, nothing downstream changes for chords; walking blocks by `represents_measures` so merged rest runs keep the key in step. `NoteLabels::HarmonicaTabs` instead sets it to the note's harmonica tab (`src/compiler/harmonica_tabs.rs`, `place_harmonica_tabs`), picking each part's holes along its least-movement path (`src/compiler/least_movement_path.rs`, the same dynamic program the guitar chord view uses; `src/compiler/element_position.rs` addresses and writes back the picks for both), with `NoteHeadLabel::slide_button` set when the note needs the slide, flagging a pitch with no hole as `unplayable` with the text `?`, sets the head's `octave` to `0` (the hole already gives the octave, so no octave dots are drawn) and leaves chord symbols as degrees — see **Harmonica-tab view** in the glossary. `head_label` is copied through `GridContent::NoteHead`/`LowOctaveDots`, `PostArcGridContent::NoteHead` and `AbsoluteContent::NoteHead`; grid layout measures the column from the label's text width (no accidental lead, since its ♯/♭ trails), flush-left alignment uses its first character's bearing, and `render_note_head` draws its `text` in place of the digit, as `SvgVariant::UnplayableNoteHead` (`data-variant="unplayable-note-head"`, drawn red: the serializer writes `fill="#d32f2f"` for PDF, the web preview styles the variant in `web/src/injectPreviewInteractionStyles.ts`) when `unplayable`, else `SvgVariant::NoteHead`, and a `slide_button` label also gets a left arrow of three `Line` strokes above it, as `SvgVariant::HarmonicaSlide` (`data-variant="harmonica-slide"`). `NoteHeadLabel::run_text(digit)` gives the run's text after any leading ♯/♭: the label alone, or `digit (label)` for `HeadLabelPlacement::AfterDegree`. Percussion hits are untouched. See **Letter-name view** in the glossary.
- Guitar chords: `NoteLabels::GuitarChords` makes `label_note_heads` call `guitar_chords::place_guitar_voicings(result, score)` (`src/compiler/guitar_chords.rs`), which sets each `ElementContent::ChordSymbol`'s `guitar_voicing: Option<GuitarVoicing>`, and then spells letter names as for `LetterNames` but after the degrees: each note head's `head_label` gets `placement: HeadLabelPlacement::AfterDegree` (drawn as one run `♯4 (F♯)`, the ♯/♭ still leading the digit, so the column keeps its accidental lead, flush-left alignment uses the digit's bearing, and octave dots stay over the digit), and each chord text becomes `degree_symbol (letter_symbol)` (`6m (Bm)`). Every chords-db voicing of each chord (`pitch_description::guitar_voicings`, slash-specific voicings first) becomes one layer per chord part, in written order, and `guitar_voicing_path::least_movement_path(layers: &[Vec<&GuitarVoicing>]) -> Vec<usize>` picks one voicing per layer through the generic `compiler::least_movement_path::least_movement_path(layers, &PathCosts { node, movement })`, a Viterbi dynamic program (see **Voicing path**). A chord chords-db has no voicing for keeps `None` and is left out of the path. The voicing is copied into `GridContent::GuitarDiagram { voicing }` / `PostArcGridContent::GuitarDiagram` / `AbsoluteContent::GuitarDiagram` (its `x` the diagram's left edge, its `y` its bottom edge), drawn in the part's **Guitar-diagram band**, and `layout_spacing::column_rod` widens a chord's column to at least `font_metrics::guitar_diagram::width_pt(voicing)`. See **Guitar chord view** in the glossary.
- Key types: `CompileResult` (carries `tuplet_spans: Vec<TupletSpan>` alongside `slur_spans`, populated the same way — see below), `LyricLink` (`lyric_row_id`/`target_row_id` as `RowId`s, kept in `CompileResult::lyric_links` and built from the part-level `LyricsSlice::target_name`; rows carry no target, and grid layout reads the links via `sings_along_to` to group a note row with the lyric rows under it for click targets and the playback cursor), `MeasureBlock` (carries `represents_measures: usize`, the number of original source measures the block stands in for — `1` normally, `> 1` when a rest run was folded; consumers that walk blocks and assume one block == one measure must advance their measure counter by this field, not by `1`; also carries `merge_duplicate_measures_across_parts: bool`, copied from the originating measure's `MultiPartMeasure::merge_duplicate_measures_across_parts` — see Consolidator below; also carries `source_span: Span`, copied from the originating `MultiPartMeasure::source_span` (a merged rest-run block instead takes the min/max union of every measure's own span it stands in for, mirroring how `combiner` computes `MultiPartMeasure::source_span` itself) — this is what lets a layout-stage diagnostic like `WarningKind::MeasureOverflow` (see **Rod and spring** in the glossary) point at real source text even though grid layout runs after grouping/compiling), `MeasureRow` (carries `kind: PartKind`, the kind of the part it was compiled from — a lyric part's row is `PartKind::Lyrics { .. }` and `grid_layout::is_lyric_row` is a check on it — and `absorbed_rows: Vec<MeasureRow>`, empty until the consolidator merges another row's content into this one — see Consolidator below), `ColumnElement` (carries `note_id: Option<usize>` — see **Note identity** in the glossary), `ElementContent` (includes `PercussionHit`, threaded through `GridContent`/`PostArcGridContent`/`AbsoluteContent`/`SvgVariant::PercussionHit` and rendered centered via `render_percussion_hit`; also includes `MultiMeasureRest { count }`, threaded the same way through `GridContent`/`PostArcGridContent`/`AbsoluteContent`/`SvgVariant::MultiMeasureRest` and rendered via `render_multi_measure_rest` — see **Multi-measure rest** in the glossary; `Rest` also carries `implicit_fill: bool`, threaded the same way through `GridContent`/`PostArcGridContent`/`AbsoluteContent`/`SvgVariant::OmittedPartRest` and rendered via `render_omitted_part_rest` — see **Omitted-part rest** in the glossary), `SlurSpan`, `TupletSpan` (`{ part_index, measure_index, from_column, to_column, label }` — like `SlurSpan` but a single `measure_index` instead of `from_measure`/`to_measure`, since a tuplet never crosses a measure boundary; `label` is the tuplet's printed digit, e.g. `"3"` — see **Tuplet** in the glossary), `ArcKind`, `Decoration`
- `compiler::visible_part_indices(measure: &MultiPartMeasure) -> Vec<usize>` resolves which of `measure.parts` are actually compiled/sounded for that measure (an all-rest part is hidden when `hide_resting_parts` is in effect and at least one other part has real content). `compile_measure` uses it directly; `midi::timing::note_timings_seconds` (see **Note identity**) also walks `measure.parts` filtered through this same function, so its per-part `note_id` counters advance in lockstep with `ColumnElement::note_id`.
- `compile()`'s main loop still produces one block per source measure (preserving per-measure `cross_states`/`slur_spans`/bar-number bookkeeping); a post-pass `merge_rest_runs()` then folds maximal runs of 2+ eligible all-rest measures (see **Multi-measure rest**) into single blocks. Grid layout stays unaware of this folding — it only ever sees already-resolved blocks and lays out whatever `ElementContent`/`GridContent` variant it's given.
- `hide_resting_parts` (from `MultiPartMeasure::hide_resting_parts`, resolved per measure — see Grouper above) gates `compile_measure`'s omission of an all-rest part's row when at least one other part in the same measure has real content; when `false`, every part's row is rendered even if it's all rests.
- `PartSlice` also carries `resolution_multiplier: u32` (copied from the source `GroupedMeasure::resolution_multiplier` by `combiner::build_part_rows`; `1` for measures with no tuplets). `compiler::part_slice::compile_part_slice` reads it once per slice into `PartState::multiplier`, and every quarter-beat-grid literal in `part_slice.rs`/`part_slice_unit.rs` — the underline-count thresholds (base `1`/`2`/`3`), the note-dash spacing (base `4`) — is expressed as `BASE_CONST * multiplier` so a rescaled measure still lays out correctly. The beam-flush check itself (`beat_position % (beat_group_size * multiplier) == 0`) instead uses `PartState::beat_group_size`, read from `PartSlice::beat_group_size` — `4` for simple meters, `6` for compound meters — so a run of beamed notes/rests groups by dotted-quarter (3 eighth notes) rather than quarter-note in 6/8, 9/8, 12/8, etc. This is exact for untagged (non-tuplet) notes/rests in a rescaled measure, since their `duration` was scaled by exactly `multiplier` too; a tuplet-tagged unit's `duration` is additionally ratio-compressed (`* den / num` by `rescale_tuplets`), so before comparing against the underline-count thresholds, `compile_unit` first undoes that compression (`* num / den`, exact since `duration` was constructed as a multiple of `den`) to recover the note's written duration scaled by `multiplier` — so underline count reflects each note's written value (e.g. sixteenth notes get a double underline) independent of the tuplet's rescaled duration; the tuplet bracket/number is a separate overlay on top (see **Tuplet** in the glossary). `column: u32` itself needs no scaling — it's a pure additive grid index that already accumulates whatever (possibly rescaled) `duration` each unit was given.
- `TupletSpan`s are accumulated during the same pass as `SlurSpan`s, but by a separate, simpler mechanism (`compiler::tuplet_spans`'s `record_tuplet_tag`/`finish_tuplet_spans`, called from `compile_timed_unit`/`compile_unit` via the new `TimedUnit::tuplet()` trait method, and directly off `GroupedRest.tuplet` in `compile_rest`) since a tuplet never carries across measures: `PartState` holds one `current_tuplet: Option<PendingTupletSpan>`, reset fresh per `compile_part_slice` call, tracking the first/most-recent column of the contiguous run of notes/rests sharing one `TupletInfo` value; any break (an untagged event, or a different ratio) flushes it into `CompileResult.tuplet_spans`, remapped through `merge_rest_runs`'s `measure_to_block` exactly like `slur_spans`.
- A change in `merge_duplicate_measures_across_parts`/`hide_resting_parts` between two otherwise-collapsible measures forces a new run boundary in `merge_rest_runs()`, the same way a `label` does, since a merged block carries one resolved setting pair (taken from the run's first measure) for every measure it stands in for.

### Consolidator
- Module: `src/consolidator/`
- Entry: `consolidator::consolidate(result: CompileResult) -> CompileResult`
- Removes duplicate rows within each measure when their `elements` are identical (labels and ids are not compared). `slur_spans` are passed through unchanged. Lyric parts need no splitting here: the compiler already emits each as its own `MeasureRow` with `kind: PartKind::Lyrics { .. }` (the row it sings along to is recorded in `CompileResult::lyric_links`; that row may be absent from the block when the target is hidden — the lyric row's elements still sit at the target's columns).
- `merge_duplicate_measures_across_parts` (read per block from `MeasureBlock::merge_duplicate_measures_across_parts`, resolved per measure by the grouper — see Grouper above) gates only merges between rows with different `source_part_index`; rows from the same part still merge regardless.
- Because each lyric part has its own `RowId`, the set of lyric parts written in a measure simply changes which rows exist for that measure's block — `grid_layout::layout_systems::pack_into_systems`'s union-of-parts packing (see **System**) resolves this by padding the missing lyric row(s) into the measure lacking them rather than breaking the system.
- When rows merge, the surviving row's own `label` is left untouched (it stays that row's own per-part identity, as compiled) — `consolidate_rows` only decides *which* rows to fold together, never what to call the result. That's deliberate: `consolidate_block` only ever sees one measure at a time, so it can't yet tell whether a content match holds for the whole system (in which case the merge should stick) or is a one-measure coincidence with a part that needs its own separate row elsewhere in the system (in which case it shouldn't). The *displayed* label is resolved later, by `grid_layout::layout_systems::resolve_label` (see Grid Layout below), once a system's full `RowId` membership is known.
- The merged-away row itself, in full — not just its `RowId` — (plus anything already in its own `absorbed_rows`, for a chain of three-or-more-way merges) is appended to the surviving row's `absorbed_rows` before it's discarded — otherwise that row's own real content (real `note_id`s, real click/playback targets) would vanish from the block with no trace, and a later measure in the same system whose parts *don't* merge would make `pack_into_systems`'s union-of-parts packing (see **System**) wrongly pad this block with a rest for a part whose notes are actually right here, merged into another row, or wrongly re-render it as a non-identity duplicate instead of its own genuine row. Each absorbed row's own `label` is likewise left untouched, so `resolve_label` can later read it as that part's true, unmerged identity.

### Grid Layout
- Module: `src/grid_layout/`
- Entry: `grid_layout::layout(result: &CompileResult, config: &RenderConfig, header: &Header, width_pt: f32, height_pt: f32, highlighted_measure_ranges: Option<Vec<MeasureRange>>) -> LayoutOutput` — `LayoutOutput { pages: Vec<GridPage>, diagnostics: Vec<Diagnostic> }`, where `diagnostics` currently only ever carries `WarningKind::MeasureOverflow` warnings (see **Rod and spring** in the glossary); both `document_render.rs` and `lib.rs`'s call sites merge these on top of `collect_measure_diagnostics`'s pre-layout, grouped-`Score` diagnostics before returning `RenderDocumentOutput`/`RenderOutput`. `highlighted_measure_ranges` is a list of disjoint, inclusive `global_measure_index` ranges (not a single range) so a `sequence` chain selection spanning out-of-order entries can highlight several disjoint measures at once — see `grid_layout::highlight::compute_measure_highlights_for_range`.
- Key types: `GridPage` (carries `playback_cursor_targets: Vec<PlaybackCursorTarget>`, `part_label_click_targets: Vec<PartLabelClickTarget>`, `lyric_click_targets: Vec<LyricClickTarget>`, `lyric_label_click_targets: Vec<LyricLabelClickTarget>`, and `bar_number_click_targets: Vec<BarNumberClickTarget>` alongside `measure_highlights`/`error_highlights`/`measure_click_targets` — see **Playback cursor**, **Part label click target**, **Lyric click target**, **Lyric label click target**, **Bar number click target**), `GridRow`, `GridElement`, `GridContent` (includes `TupletBracket { label }` — same-system-only counterpart of `TieOrSlur`, with no `Tail`/`Head` split since a tuplet never crosses a system — resolved by `grid_layout::tuplet_placement::resolve_tuplet_spans` from `CompileResult.tuplet_spans`, mirroring `slur_placement::resolve_slur_spans`; see **Tuplet** in the glossary), `HAlign`, `VAlign`, `ColumnGeometry`, `MeasureColumnLayout`, `MeasureRange` (`start`/`end` — one inclusive `global_measure_index` range to highlight; `layout`'s `highlighted_measure_ranges` takes a `Vec` of these rather than one range so disjoint sequence-chain selections can each get their own highlight), `PlaybackCursorTarget` (`row_start`/`row_end`/`column_start`/`column_end` like `MeasureHighlight`, plus `source_part_index`/`note_id`, plus `click_row_end` — the note's own click/selection target's bottom row, which (unlike `row_end`) never absorbs a following lyric row; see **Note click target**), `PartLabelClickTarget` (`row_start`/`row_end`/`source_part_index`/`measure_index_start`/`measure_index_end` — see **Part label click target**), `LyricClickTarget` (`row`, `column_start`, `column_end`, `source_part_index`, `note_id` — a single row/column-span pair, unlike `PlaybackCursorTarget`'s `row_start`/`row_end`, since one syllable never spans more than one row — see **Lyric click target**), `LyricLabelClickTarget` (`row`, `source_part_index`, `measure_index_start`, `measure_index_end` — a single row within the fixed-width label region, the lyric-side mirror of `PartLabelClickTarget` — see **Lyric label click target**)
- `grid_layout::system_walk::for_each_system(page_systems, tuplet_bracket_map, header, base, hide_system_dividers, visit)` is the shared page→system→row-offset walk behind every `compute_all_*` function below (and `compute_all_measure_click_targets` above): it threads `row_offset` across system-divider rows, decoration rows, and each system's own musical row count, invoking `visit(page_idx, system, row_offset, bands, musical_row_count)` once per non-empty system, where `bands: &SystemBands` (`layout_system_bands::system_bands`) names which parts of that system get the optional tuplet-bracket and guitar-diagram bands. Any state a caller accumulates across systems (a running `global_measure_index`, an output `Vec`) it owns itself and mutates from inside the closure.
- `grid_layout::playback_cursor::compute_all_playback_cursor_targets` (walks `for_each_system`) groups each `MeasureRow`'s elements by `note_id` per block and emits one `PlaybackCursorTarget` per contiguous run, scoped to that one part's own sub-rows (`note_row_spans`) rather than the whole system's row range `MeasureHighlight` uses. A tied note split across a system/line break, or a repeated measure position with no navigation-aware remapping, can legitimately produce more than one `PlaybackCursorTarget` sharing the same `(source_part_index, note_id)` — the frontend highlights all of them together. `playback_cursor::note_row_spans` (renamed from `part_row_ranges`) returns one `NoteRowSpan { row_start, playback_row_end, click_row_end }` per row template: `playback_row_end` absorbs any immediately-following lyric row(s) (feeding `PlaybackCursorTarget::row_end`), while `click_row_end` is captured before that absorption and never extends past the note row's own sub-rows (feeding `PlaybackCursorTarget::click_row_end`) — see **Note click target**.
- `grid_layout::click_targets::compute_all_part_label_click_targets` walks `for_each_system`, reusing `playback_cursor::note_row_spans`' `playback_row_end` (not `click_row_end`) for each labeled note row's `row_end` — a part label is a shortcut for "select everything under this label," lyrics included, so (unlike a note's own click target) it deliberately absorbs any following lyric row(s) — and accumulating the same running `global_measure_index` to give every part label in a system its `measure_index_start`/`measure_index_end` — that system's full measure range (including merged multi-measure rests, via `MeasureBlock::represents_measures`). Skips rows `expand::expand_note_part` never draws a `RowLabel` for (an empty `label`, or a lyric/absorbed row) so a click target only ever exists where the label itself is visibly drawn.
- `grid_layout::click_targets::compute_all_lyric_click_targets` walks `for_each_system`, but per-syllable rather than per-note: `lyric_row_absolute_indices` mirrors `note_row_spans`'s cursor walk except it records each lyric row's own single absolute row index instead of merging it into its note row's combined span, since a `LyricClickTarget` (unlike a `PlaybackCursorTarget`) never needs to span more than one row. One target is emitted per `(source_part_index, note_id)` pair actually present in that row's elements — see **Lyric click target**.
- `grid_layout::click_targets::compute_all_lyric_label_click_targets` walks `for_each_system`, reusing the same `lyric_row_absolute_indices` walk as `compute_all_lyric_click_targets` (rather than `note_row_spans`) to find each lyric row's own absolute row index, and the same running `global_measure_index` accumulation as `compute_all_part_label_click_targets` to give every lyric label in a system its `measure_index_start`/`measure_index_end`. One target is emitted per lyric row (`PartKind::Lyrics`) with a non-empty `label` — see **Lyric label click target**.
- `grid_layout::click_targets::compute_all_bar_number_click_targets` walks `for_each_system`, alongside the same `global_measure_index` accumulation `compute_all_measure_click_targets` performs. `MeasureClickTarget`'s row range never reaches into the shared directive row above the musical rows (`for_each_system` advances `row_offset` past it before `visit` runs), so a measure's own bar number — drawn there by `layout_decoration::make_decoration_row` — falls outside every existing click target; this is a second, narrower target (`row`/`column`, not a row/column *range*) sized to just that bar number's own grid column, one row up (`row_offset.saturating_sub(1)`). Emitted for exactly the blocks whose bar number `make_decoration_row` actually draws — `layout_decoration::directive_line_should_emit(index, dec)` (shared by both) — so an otherwise-plain mid-system measure's undrawn bar number gets no click target either. See **Bar number click target**.
- `layout::note_part_sub_row_heights(base) -> [f32; 7]` returns `[tuplet_bracket, arc, above_dot, note_head, half_ul, quarter_ul, below_dot]` (the underline bands sit directly under the note head so an underlined note's below-octave dots — a separate `GridContent::LowOctaveDots` element in its lowest underline's sub-row — hang beneath its underlines) — the tuplet-bracket row is reserved unconditionally for every note-part row (like the arc row already was), so it counts toward every fixed sub-row index/count derived from it: `expand::expand_note_part`'s `arc_sub`/`head_sub` (`(1, 3)` for a note row vs. `(0, 1)` for a chord-only row, which has no tuplet-bracket row and keeps the old 4-row `chord_part_sub_row_heights` layout), `highlight::system_musical_row_count`, and `playback_cursor::note_row_spans` (both `7` in place of the old `6` for a non-chord row). Because this row is unconditional, every note-part row is one sub-row taller than before Step 7 added it, even for a score with no tuplets at all — unlike Step 6's grid-width/MIDI-tick changes, this is *not* a pixel-identical no-op for existing (non-tuplet) content.
- `grid_layout::click_targets::targets_on_page<T: Clone>(targets: &[(usize, T)], page_idx: usize) -> Vec<T>` is the generic filter behind every `GridPage`-per-page split in `layout()` — measure/playback-cursor/part-label/lyric/lyric-label click targets alike — replacing four near-identical hand-copied `*_on_page` functions.
- `config.hide_system_dividers` (from `Metadata::hide_system_dividers`, `metadata` field `hide system dividers`, default `no`) gates `layout::build_page_rows`'s insertion of the full-width `GridContent::HorizontalLine` separator row between systems, and the corresponding gap in `layout()`'s page-break height accounting; when `true`, no divider row or gap is inserted between systems.
- `GridRow.has_label_region` (`true` for system rows built by `expand::expand_system_to_rows`/`layout_decoration::make_decoration_row`, `false` for header/footer/separator rows) marks which rows reserve `LABEL_COLS` columns for the part label. `GridRow::column_geometry(usable_width_pt, label_width_pt) -> ColumnGeometry` uses this flag to give the label region a fixed pixel width (`config.part_label_width_pt`) independent of that row's own `column_count`, while the remaining columns still split the rest of the width evenly — this is what keeps the part-label column the same rendered width across systems of differing musical density, rather than the old uniform `usable_width_pt / column_count` division which made the label's width vary with each system's content.
- `GridRow.measure_layout: Vec<MeasureColumnLayout>` (one entry per measure in the row's system, shared by every row of that system — built once by `layout::build_measure_column_layout(system, config)` and threaded into `expand::expand_system_to_rows`/`layout_decoration::make_decoration_row`) records each measure's `start_col`, `col_count` (`block_column_width`), `weight` (`layout_spacing::measure_note_weight`, the max summed real glyph width — measured via `font_metrics::advance_width_for_family`/`text_width_for_family` (family-aware since `notes`/`chords`/`note_dash` accept a `font_family` override — see **GlyphFontFamilies**) against `config`'s lyric font size, not a raw event count — of note/rest/percussion-hit/chord-symbol elements across the measure's part rows, so density scales with the busiest part rather than the part count; dashes and bar lines don't contribute here; a collapsed `MultiMeasureRest` gets `layout_spacing::multi_measure_rest_weight` instead — a real point width floored at `MULTI_MEASURE_REST_WIDTH` note glyphs, growing beyond that floor to match the count label's own rendered width once a run's count needs more digits than the floor allows), and `rod_pt`/`column_rods` — see **Rod and spring** in the glossary for what these are and how `column_geometry` splits width using them — plus `column_accidental_leads` (see **Accidental lead** in the glossary). Per-column weights within a measure (`layout_spacing::measure_column_weights`, also `config`-aware) are likewise real font-metric widths for glyph columns (notehead/rest/chord/dash/lyric), while `BarLine`/`Underline` columns keep their old arbitrary flat weights (`THIN_MARK_WEIGHT`/`0.0`) since they aren't glyphs with a measurable rendered width; a `MultiMeasureRest` column instead gets an even share of `multi_measure_rest_weight`'s real point total spread across the span. `layout_spacing::measure_column_sizes` computes `ColumnSizing { weight, rod_pt, accidental_lead_pt }` per column in one pass (max-across-parts, like the old weight-only version), and `measure_column_weights` is now a thin `#[cfg(test)]` wrapper over it that returns just the `weight`s. When `measure_layout` is empty (headers/footers/separators, or a row with no system), `column_geometry` falls back to the old single-scalar uniform division. Because `x_start`/`col_width` become piecewise-linear across segments, span-width consumers (`coordinate_resolver::resolve`'s arc/underline formulas, `coordinate_resolver::highlights`'s highlight/click-box formulas) must compute a span's width as `x_start(end) - x_start(start)` rather than `col_width(start) * span`, so a span crossing measures of different width still gets its true pixel extent. `ColumnGeometry::glyph_left_anchor_x(col, padding) -> f32` (`x_start(col) + accidental_lead(col) + padding` — see **Accidental lead** in the glossary) is the shared helper for note/rest/dash/chord/percussion-hit/lyric-syllable anchoring and arc/underline/tuplet-bracket endpoints: every glyph anchors at the column's left edge plus `padding`, the same regardless of the column's own width or what else shares it — replacing an earlier weighted-centering formula that scaled a glyph's anchor toward the column's true center in proportion to its own weight share of the column's total, which made two visually similar notes land at different offsets depending on what else shared their column. `padding` itself is no longer one flat constant: every flush-left renderer (note head, rest, percussion hit, chord symbol, note dash, lyric syllable) now draws `TextAnchor::Start` at this anchor, so `coordinate_resolver::resolve::flush_left_padding` computes `padding = element_padding - bearing(leading_char)` per element, where `element_padding` is that content type's own configured horizontal padding (`RenderConfig::notes_horizontal_padding_pt`/`chords_horizontal_padding_pt`/`lyrics_horizontal_padding_pt`/`note_dash_horizontal_padding_pt`, sourced from `Metadata::*_horizontal_padding_pt` — see `ElementPaddings`) — the leading character's own left-side bearing (`font_metrics::glyph_left_bearing_for_family`/`cjk_glyph_left_bearing`, measured against the pinned font actually used to render that content type) — so every glyph's *ink*-left-edge, not just its anchor, lines up flush regardless of which content type shares the column (see `font_metrics::is_cjk_char`/`lyric_font_size`, the shared CJK-detection/font-size-selection helpers behind the lyric case). The tie/slur/underline/tuplet-bracket span markings in `resolve_span_marking` still use one flat `padding` (`notes_horizontal_padding_pt + note_number_width * 0.5`, approximating a note's *center* rather than any one glyph's bearing, since a span can cover notes of differing pitches/widths) — the same nominal `note_number_width` box the renderer's own `center` reconstructs in `glyph_renderers.rs::render_note_head` for its octave-dot placement.

### Coordinate Resolver
- Module: `src/coordinate_resolver/`
- Entry: `coordinate_resolver::resolve(pages: &[GridPage], note_number_width: f32, page_frame: PageFrame, font_sizes: ResolveFontSizes) -> Vec<AbsolutePage>`
- `grid_layout::PageFrame { margin_pt, part_label_width_pt }` (built by `RenderConfig::page_frame()`, `margin_pt` from **Page margin**) is the page's frame: `usable_width_pt`/`usable_height_pt` subtract the margin from both sides, and `row_geometry(row, page_width_pt)` returns that row's `ColumnGeometry` already shifted by the margin (`ColumnGeometry::offset_by`, stored as `origin_x`), so every `x_start` is an absolute page x. The resolver wraps it with the page width in `coordinate_resolver::page_geometry::PageGeometry`, which every per-row resolver takes instead of a `(usable_width, part_label_width_pt)` pair, and starts each page's first row at `margin_pt`. Nothing in the resolver adds a margin by hand.
- `ResolveFontSizes { lyric: LyricFontSizes, notes: f32, chords: f32, labels: LabelFontSizes }` bundles the resolver's font-size inputs (kept under the 6-arg clippy limit); `LabelFontSizes { measure_number, section_label, part_label }` carries the three label-specific sizes.
- Key types: `AbsolutePage`, `AbsoluteElement`, `AbsoluteContent` (includes `PlaybackCursorTarget { width, height, source_part_index, note_id }`, resolved by `coordinate_resolver::highlights::resolve_playback_cursor_target` — see **Playback cursor**; also includes `NoteClickTarget { width, height, source_part_index, note_id }`, resolved by `coordinate_resolver::highlights::resolve_note_click_target` from the same `grid_layout::types::PlaybackCursorTarget` list, but using its `click_row_end` field rather than `row_end` as the row range's bottom edge — see **Note click target**; also includes `PartLabelClickTarget { width, height, source_part_index, measure_index_start, measure_index_end }`, resolved by `coordinate_resolver::highlights::resolve_part_label_click_target` from `grid_layout::types::PartLabelClickTarget`, fixed to the label region (columns `0..LABEL_COLS`) rather than a caller-supplied `column_start`/`column_end` — see **Part label click target**; also includes `LyricClickTarget { width, height, source_part_index, note_id }`, resolved by `coordinate_resolver::highlights::resolve_lyric_click_target` from `grid_layout::types::LyricClickTarget` — a single row's `column_start`/`column_end` looked up directly via `GridRow::column_geometry`, no bar-line-snapping needed since a syllable never touches a bar line — see **Lyric click target**; also includes `LyricLabelClickTarget { width, height, source_part_index, measure_index_start, measure_index_end }`, resolved by `coordinate_resolver::highlights::resolve_lyric_label_click_target` from `grid_layout::types::LyricLabelClickTarget`, fixed to the label region (columns `0..LABEL_COLS`) like `PartLabelClickTarget` but scoped to a single row (`target.row..=target.row`) rather than a part's whole span of sub-rows — see **Lyric label click target**; also includes `TupletBracket { label, width }`, resolved by `resolve::resolve_span_marking`'s `GridContent::TupletBracket` arm the same way as `TieOrSlur` — same-system glyph-anchor-to-glyph-anchor math (`ColumnGeometry::glyph_left_anchor_x`, see **Grid Layout**), no tail/head split; also includes `BarNumberClickTarget { width, height, measure_index, measure_index_end }`, resolved by `coordinate_resolver::highlights::resolve_bar_number_click_target` from `grid_layout::types::BarNumberClickTarget` — see **Bar number click target**), `PostArcGridContent`
- `coordinate_resolver::highlights::resolve_row_range_geometry(row_start, row_end, column_start, column_end, ctx: RowLayoutContext { rows, row_tops, page_geometry }) -> Option<RowRangeGeometry>` is the shared row-range→pixel-geometry math behind every row-range-shaped resolver above (measure/error highlight, playback cursor, note/measure/part-label click target) — each wraps its `Some(geometry)` result in its own `AbsoluteContent` variant. `resolve_lyric_click_target` doesn't use it: a lyric syllable's target is always exactly one row/one column, with no bar-line-snapping and no `rows.len()` bounds check. `resolve_bar_number_click_target` doesn't use it either — a bar number's width isn't a column-bounds span but its own rendered digits' width, measured via `font_metrics::span_width` against the identical `TextSpan` `content_conversion::bar_number_text_span` builds for `AbsoluteContent::DirectiveLine`'s own `bar_number` text, then padded by `BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING` (6pt) on both sides — the digits' own glyph width alone (a few points for a single digit) is too thin a strip to reliably hover/click.
- `PostArcGridContent`: `GridContent` minus the four span-marking variants (`TieOrSlur`, `TieOrSlurTail`, `TieOrSlurHead`, `TupletBracket`); these are resolved before `grid_to_absolute` (in `resolve_span_marking`) and must not appear in the coordinate-resolver layer.

### Renderer
- Module: `src/renderer/`
- Entry: `renderer::new_renderer::render_new(pages: &[AbsolutePage], config: &RenderConfig) -> Vec<SvgDocument>`
- Key types: `SvgDocument`, `SvgElement`, `SvgKind` (includes `PlaybackCursorRect { width, height }`, rendered `fill="transparent"` by the serializer/preview and toggled at playback time by the frontend — see **Playback cursor**), `SvgVariant` (includes `TupletBracket`, rendered `"tuplet-bracket"`), `TransparentRectRole` (includes `NoteClickTarget`, rendered `"note-click-target-rect"` — see **Note click target**; also `PartLabelClickTarget`, rendered `"part-label-click-target-rect"` — see **Part label click target**; also `LyricClickTarget`, rendered `"lyric-click-target-rect"` — see **Lyric click target**; also `LyricLabelClickTarget`, rendered `"lyric-label-click-target-rect"` — see **Lyric label click target**; also `BarNumberClickTarget`, rendered `"bar-number-click-target-rect"` — a deliberately distinct role from `MeasureClickTarget` purely so the frontend can hover-style it alone, since `MeasureClickTarget`'s own rect already spans every note in the measure — see **Bar number click target**), `Tag` (includes `Note { source_part_index, note_id }`, wrapping a `PlaybackCursorRect` in one `Group` (`render_playback_cursor_target`) and, in a sibling `Group` carrying the same tag, a `TransparentRect { role: NoteClickTarget }` (`render_note_click_target`) — mirroring `Tag::Measure`'s wrap of a `MeasureClickTarget`; also `PartLabel { source_part_index, measure_index_start, measure_index_end }`, wrapping a `TransparentRect { role: PartLabelClickTarget }` (`render_part_label_click_target`) — see **Part label click target**; also `Lyric { source_part_index, note_id }`, wrapping a `TransparentRect { role: LyricClickTarget }` (`render_lyric_click_target`) in its own `Group`, independent of the note's `Tag::Note` group — see **Lyric click target**; also `LyricLabel { source_part_index, measure_index_start, measure_index_end }`, wrapping a `TransparentRect { role: LyricLabelClickTarget }` (`render_lyric_label_click_target`) — see **Lyric label click target**; also `BarNumber { index, end }`, rendered `data-tag="bar-number"` (not `"measure"`, unlike `Tag::Measure`, to keep `[data-tag="measure"]`'s one-group-per-measure DOM shape unchanged), wrapping a `TransparentRect { role: BarNumberClickTarget }` (`render_bar_number_click_target`) — see **Bar number click target**)
- `renderer::new_renderer::glyph_renderers::render_guitar_diagram` (`glyph_renderers_guitar_diagram.rs`) draws one chord box from `AbsoluteContent::GuitarDiagram`, sized by `font_metrics::guitar_diagram`: grid `Line`s, a thick nut line (or a sans-serif `5fr` base-fret label on the right), `SvgKind::RoundedBar` barres, two crossed `Line`s for a muted string, `SvgKind::RingCircle` for an open string, and `Circle` dots. Every stroke has `SvgVariant::GuitarDiagram`, and the diagram is grouped under `Tag::GuitarDiagram { frets, base_fret }` (`data-tag="guitar-diagram"`, `data-guitar-frets="x 3 2 0 1 0"`), which tests read instead of geometry.
- `renderer::new_renderer::click_targets::wrap_click_target(elem, rect_kind, tag) -> Vec<SvgElement>` is the shared single-child `SvgKind::Group` wrapping behind every `render_*_click_target`/`render_playback_cursor_target` function — each caller just builds its own rect `SvgKind` and `Tag`.
- `renderer::new_renderer::glyph_renderers::render_tuplet_bracket` (dispatched alongside `render_tie_or_slur` in `new_renderer.rs`) draws a flat bracket — two short downward `SvgKind::Line` ticks joined by a horizontal `SvgKind::Line` spanning `width`, with an `SvgKind::Text` label (the tuplet digit) centered above the midpoint — visually distinct from `render_tie_or_slur`'s curved `SvgKind::Path` arc, so a tuplet bracket and a slur/tie arc stacked in the same measure (bracket row above arc row — see **Grid Layout**) don't look identical.
- `SvgElement.variant` is `Option<SvgVariant>`: `None` for group wrappers and highlight rects; `Some(...)` for musical/export drawable primitives
- `SvgKind::TransparentRect` carries a `role: TransparentRectRole` for CSS hover targets (`data-variant` in serializer/preview); roles are `MeasureClickTarget`, `SectionLabelBackground`, `SectionLabelClickTarget`, `NoteClickTarget` (see **Note click target**), `PartLabelClickTarget` (see **Part label click target**), `LyricClickTarget` (see **Lyric click target**), and `LyricLabelClickTarget` (see **Lyric label click target**)
- `config.directive_row_offset` (from `Metadata::directive_row_offset`, `metadata` field `directive row offset`, an `Offset { x, y }` in points, default `(0, 0)`) is applied unconditionally by `render_directive_line` (`src/renderer/new_renderer/directive_line.rs`) to every directive row — the bar-number/section-label/key/bpm/time-signature line above the staff — shifting all of that row's elements (bar-number text, key/bpm/time-signature text, optional label background rect, optional independent label text) together, after `coordinate_resolver` has already resolved their absolute position. `AbsoluteContent::DirectiveLine` renders the bar number, section label, and the key/bpm/time-signature run as three independently-positioned elements rather than one shared text run, each with its x position precomputed in `coordinate_resolver::content_conversion` from real glyph-advance measurements (`src/font_metrics/`, pinned against the same font used for PDF export — embedded via `include_bytes!` on the host CLI build, or supplied at runtime via `set_layout_font_bytes(FontBytesByFamily)` on the `wasm32` build, since embedding the fonts in the wasm binary would otherwise add ~16MB to it) instead of a character-bucket heuristic: `bar_number: Option<TextSpan>` is pinned to the line's start (offset 0); `label_x_offset: f32` positions the section label past the bar number (plus a fixed gap) when one precedes it, zero otherwise; `spans_x_offset: f32` positions the remaining spans past whichever of `bar_number`/`label` is present, so none of the three overlap regardless of their measured widths. When a label is present, its `<g data-tag="section-label">` group also carries an invisible `SectionLabelClickTarget` rect spanning the whole directive line (bar number through trailing spans) as its first child — without it, the group's actual clickable area would be only the union of its children's painted pixels, which leaves an unpainted gap (and therefore a dead click zone, despite `cursor: pointer` covering it) whenever a directive line has a label but no trailing spans (e.g. a bare `label="B"` line). The `sequence` summary header line renders through the same function but is explicitly excluded (`AbsoluteContent::DirectiveLine.apply_row_offset = false`). It never reaches `grid_layout` or `coordinate_resolver` — those layers are unaware of it.

### Serializer
- Module: `src/serializer/`
- Entry: `serializer::serialize(docs: &[SvgDocument], source: Option<&str>) -> Vec<String>`
- When `source` is `Some`, each emitted `<svg>` gets a base64-encoded, non-rendering `<metadata id="poentaro-source">` child holding the original `.poentaro` text, so a lost source file can be recovered from a previously exported SVG. `source_embed::extract_embedded_source(svg: &str) -> Option<String>` (`src/source_embed.rs`) reverses this. Only call sites with source in scope (e.g. `render_svgs_from_source_filtered`, `render_svgs_with_highlight_range`) pass `Some`; the `Score`-only `render_svgs` path passes `None` and emits no metadata tag.
- PDF export (`pdf::write_pdf(svgs: &[String], fonts: &fonts::FontBytesByFamily, source: Option<&str>) -> Result<Vec<u8>, IrrecoverableError>`, `src/pdf.rs`) can't reuse this `<metadata>` embedding — `usvg` (which parses each page's SVG string before `svg2pdf` conversion) strips `<metadata>` tags. Instead, when `source` is `Some`, it base64-encodes the source into a custom `/PoentaroSource` key in the PDF's `/Info` dictionary via `pdf_writer`'s `DocumentInfo`. `source_embed::extract_embedded_source_from_pdf(pdf: &[u8]) -> Option<String>` reverses this by string-searching the (uncompressed) `/Info` dictionary bytes. `write_pdf_from_source_with_visibility` (behind the CLI's non-split `generate_pdf`) passes `Some`; `write_split_pdfs_from_source` (split-track export) passes `None`, matching split-track SVG export's `None` for the same reason.

### MEI export (experimental, staff notation)
- Module: `src/mei_export/`
- Entry: `mei_export::score_to_mei(score: &Score, display_names: &HashMap<String, String>, clef_choice: ClefChoice) -> MeiDocument` (`MeiDocument { mei, measures: Vec<StaffMeasure> }`); CLI: `poentaro generate mei <file>` (always `ClefChoice::Automatic`). From source with the preview's part visibility applied (hidden parts get no staff, chord line or verse): `render_mei_from_source_with_visibility(source, filename, visibility: &ResolvedPartVisibility, instruments, clef_choice: ClefChoice) -> Result<MeiDocument, IrrecoverableError>` (`src/document_render.rs`), exported to the web app as the WASM `render_mei`.
- Web staff views: the preview's Numbered/Violin/Cello/ABC/Guitar/Harmonica toggle (`web/src/components/NotationToggle.tsx`, held by `web/src/hooks/useStaffNotation.ts`: the in-force preset's `notation` line, else the one picked this session, numbered by default; under a preset in a file you can edit, the toggle rewrites that line through `update-preset-field`) has two staff views, **Violin** and **Cello**; `staffClefFor(notation)` (`web/src/worker/staffClef.ts`) maps each to its WIT `staff-clef`, and the hook sends the worker `renderStaff { source, id, visibility, clef }`; `web/src/worker/staffMessageHandlers.ts` calls `render_mei`, then lazily `import()`s [Verovio](https://www.verovio.org) (`verovio@6.3.0`, LGPL-3.0, loaded unmodified as its own ~8 MB chunk only on the first staff render) and replies `staffRendered { id, result: { status: 'ok', score: StaffScore } | { status: 'err', message } }`, where `StaffScore { pages, measures }` holds one SVG string per page plus `render_mei`'s staff-measure list; `PreviewPages.tsx` shows the pages in place of the numbered documents.
- Staff PDF export: while the preview shows staff notation, Export > PDF and PDF (ZIP) send `notation: 'violin' | 'cello'` and `web/src/worker/staffExportHandlers.ts` engraves, in that view's clef, with the same Verovio toolkit, then hands the SVG pages back to Rust: `staff_export::write_staff_pdf(pages, fonts, source)` (A4-sized like the numbered PDF, source embedded for re-import; WASM `generate_staff_pdf`) and, per part, `staff_export::render_split_meis_from_source(source, filename, base_name, selection, instruments, clef_choice) -> Vec<SplitMeiEntry>` (WASM `render_split_meis`) → `staff_export::zip_staff_pdfs(&[StaffPdfPart], fonts)` (WASM `generate_split_staff_pdfs`).
- A side branch off the grouped `Score`, parallel to the compiler → renderer path: it emits MEI for [Verovio](https://www.verovio.org) to engrave as western staff notation. Nothing in the numbered grid pipeline is reused past the grouper.
- One `<staff>` per `notes` part, its clef set by `ClefChoice` (`staves.rs`): `Violin` puts every staff in treble clef and `Cello` in bass clef, with pitches unchanged; `Automatic` (CLI only) picks it from the part's average sounding pitch (bass below MIDI 53, octave-down treble when `octave_offset < 0`, else treble); a `chords` part becomes `<harm>` letter-name symbols (degrees spelled in the key, `6m` in F → `Dm`) over the top staff that has notes in that measure; each `lyrics [X]` part becomes a `<verse>` on X's notes, paired via `lyric_slots::measure_lyric_slots`.
- Pitches are spelled by letter from the key's tonic (`pitch::spell`), so `#4` in C is F♯, never G♭; `pitch::AccidentalTracker` writes `@accid` only against the key signature or an earlier accidental in the measure. Durations that one MEI note can't write are split into tied pieces (`duration::split_duration`); eighths and shorter are beamed per `beat_group_size`; slurs come from `( )` group depth and are encoded in the measure of their start note.
- Each `<measure>` gets an `xml:id` (format private to `mei_export`) and `MeiDocument::measures` lists every `StaffMeasure { element_id, measure_index }` (WIT `staff-measure`), so the web staff view finds a measure's SVG group by the id Rust hands it — never building or parsing ids in TS. With it the staff view gives each measure a hover target (`applyStaffMeasureHitTargets`: the numbered view's hover tint and pointer), highlights the editor's selected measures (`useStaffMeasureHighlights` / `previewStaffMeasures.ts`), scrolls to them (`findRevealTarget`), and turns click-and-click on measures into an editor selection (`previewStaffClickHandler.ts`, with a `mousemove` hover preview between the clicks), like the numbered preview's measure gesture. The geometry decisions are Rust's (`crates/poentaro-wasm/src/staff_measures`): TS measures each Verovio measure's group box and staff-line boxes into a `StaffMeasureGeometry { measure_index, measure_box, staff_line_boxes }` (WIT `staff-measure-geometry`, boxes as `BoundingBox { left, right, top, bottom }`) and paints what comes back. A measure's area runs barline to barline (its staff lines' extent) and down its whole group box, so a slur, tie or lyric crossing a barline never claims the neighbour's staff; where areas of neighbouring systems overlap, the measure whose staff lines are vertically nearest wins.
- Each note's `xml:id` is `n{start}-{end}`, its `event_span`, and Verovio keeps it in the SVG, which is how a staff view can map clicks/playback back to source.
- The `scoreDef` carries `optimize="true"` so Verovio's `condense: "encoded"` hides empty staves per system (`condense: "auto"` does not).
- Key types: `StaffPart`/`Clef`/`ClefChoice` (`staves.rs`), `SpelledPitch`/`Letter` (`pitch.rs`), `SimpleDuration` (`duration.rs`), `LayerContext`/`StaffState`/`Slur` (`layer.rs`), `Verse`/`MeasureSyllables` (`lyrics.rs`), `HarmonyPlacement` (`harmony.rs`).

## Glossary

Each entry says what a term means, in at most two sentences. How it works lives in doc comments next to the code: `trail grep "<type>"` finds them.

### Score structure

| Term | Definition |
|------|-----------|
| **Measure** | One bar of music. The score is a flat sequence of `MultiPartMeasure`s. |
| **Part** | A single instrument or voice track, declared in `parts`. |
| **Part Slice** | One part's content for a single measure (`PartSlice`). |
| **System** | A horizontal row of measures on one line of a page. Its rows are the union of its measures' rows, and it wraps early when the next measure wouldn't fit. |
| **Anyfix document** | A Poentaro document written in anyfix expressions (`syntax.md`). Metadata, parts, sequence and presets are anyfix data (`parser::anyfix::read`); the `score` body stays one raw string that the score parser reads in place. |
| **Column** | A logical horizontal slot in the compiler's grid. Each beat occupies one or more columns. |
| **RowId** | A unique string identifying a compiler row, used to correlate rows across layout stages. |
| **Quarter-beat** | The smallest time unit used for duration arithmetic. A quarter note is 4 quarter-beats. |
| **Timed unit** | The `TimedUnit` trait: anything occupying duration in the grid, meaning a note, chord or percussion hit. |
| **Ditto** | A measure whose input lines are all `"`, repeating the previous measure. It renders blank, but audio uses the resolved content. |
| **Decoration** | Measure-level metadata on a `MeasureBlock`: BPM, time signature, section label, bar number. |
| **Multi-measure rest** | A run of 2 or more measures, all rest on every visible part, collapsed into one `MeasureBlock` drawn as a wide bar with a count. A label or a change of merge or hide settings starts a new run. |
| **Omitted-part rest** | A rest filling in for a part not written in a measure, as opposed to a written `0` (`implicit_fill`). Drawn as vector strokes rather than a font glyph, since a Unicode rest depends on the viewer's fonts. |
| **Tuplet** | An `N:{notes}` or `N:M:{notes}` bracket: `N` notes in the time of `M`. Each measure is rescaled to a finer integer grid by its `resolution_multiplier`, so the engine never needs fractional durations. |
| **Relative key change** | A `key=+N` or `key=-N` directive, resolved in the parser into an absolute key. Later stages only ever see absolute keys. |

### Lyrics

| Term | Definition |
|------|-----------|
| **Lyric part** | A `parts` entry `"Name" [abbr]: lyrics [X]`, a full part with its own track and row that sings along to an earlier part `X`. It never sounds (`PartInfo.sounds` is false). |
| **Verse** | Informal name for a lyric part. The code has no separate verse concept. |
| **Lyrics line** | One plain-text line per measure per lyric part, written on that part's `[Key]` line and split into syllables. |
| **Lyric slots** | The notes of a lyric part's target that take a syllable: every note that isn't a tie continuation, with tie state carried across barlines. |
| **Bracketed lyric item** | Under a `chords` target, an item written `( … )` or `（ … ）` holding one syllable, bound to a chord attack. See "Lyrics under a chord part" in `syntax.md`. |

### Part streams

| Term | Definition |
|------|-----------|
| **Part stream** | One part's measures flattened into text with no key prefixes, barlines or directives, one **Measure chunk** per line. It is re-barred when applied back to the source. |
| **Measure chunk** | The slice of a part stream that fills one measure. An unchanged chunk leaves its source line untouched. |
| **Follow placeholder** | The `@` token in a `follow [X]` part's stream: one whole measure the part doesn't write, so it copies `X`. |
| **Part focus editor** | The web editor mode that edits one part stream in place of the source, re-barred on every keystroke. |
| **Ghost line** | A dimmed, read-only line in the part focus editor, showing another part's source line for that measure. |

### Parts, presets and visibility

| Term | Definition |
|------|-----------|
| **Part toggle** | The shown, hidden and soloed state of a part (`PartToggleState`), the same for every kind of part. `resolve_part_visibility` turns it into `ResolvedPartVisibility`, which every render, span listing and timing path uses. |
| **Preset** | A `preset "Name"` section: a saved way to view and export the song, with its visible and soloed parts, notation and metadata overrides. **Default** means no preset. |
| **Sound label** | The quoted `"N: Name"` text that picks a part's sound. Formatted and parsed only in `src/sound_label.rs`. |
| **GM percussion key** | A General MIDI drum-kit key number (0–127) identifying one drum sample, used in place of a program number on percussion parts. |
| **Range instrument** | An instrument named on a `notes` or `follow [X]` line (`notes violin`), giving a pitch range to check against and a default sound (`RangeInstrument`). |
| **Part setting limits** | The allowed volume and octave offset of a part (`PART_SETTING_LIMITS`). The parser clamps out-of-range values and reports them, so later stages only see valid ones. |

### Layout and engraving

| Term | Definition |
|------|-----------|
| **Rod and spring** | The spacing model: every measure and column first gets a hard-minimum width (rod), then leftover space is shared by weight (spring). A system whose rods exceed the page overflows with a `MeasureOverflow` warning rather than squeezing glyphs together. |
| **Accidental lead** | Room a column reserves ahead of every glyph for a sharp or flat, since accidentals draw before their digit. It keeps every part's digits vertically aligned. |
| **Underline** | A line below note heads marking duration subdivision. `level=0` is a half-beat, `level=1` a quarter-beat. |
| **Octave Dot** | A dot above or below a note head that shifts its octave. |
| **Note Dash** | A visual `-` after a note head for each extra beat of duration. |
| **Arc Span** | The full extent of one slur or tie arc, possibly crossing measures or systems (`SlurSpan`). |
| **ArcKind** | Whether a `SlurSpan` is a `Slur` (from `(…)`) or a `Tie` (from `~`). |
| **Final bar line** | The thin-then-thick double bar closing the score (`BarLineKind::Final`). |
| **Running title** | The small title at the top of every page after the first. |
| **Row Label** | The part name in a fixed-width column at the left of a system row, shared by the whole score. A lone all-rest row drops it, because a label only exists to tell rows apart. |
| **Page margin** | The blank space on every side of a page (`Metadata::page_margin_pt`). A wider margin can change where measures and pages break. |
| **TextStyle** | The style shared by every rendered text kind: size, padding, bold, italic, underline and font family. Written as a `style` record in `metadata`; see `syntax.md`. |
| **GlyphFontFamilies** | The fonts of `notes`, `chords` and `note_dash`, the three text kinds whose font also feeds glyph-width measurement. |
| **Metadata keyword enum** | An enum declared with `keyword_enum!`, so each `metadata` or directive keyword is spelled exactly once. Every layer dispatches on these rather than matching strings. |
| **Highlight token** | One editor keyword from `highlight::highlight_tokens`. Only words the parser recognizes are reported, so editor colors can't drift from the grammar. |

### Notes, playback and selection

| Term | Definition |
|------|-----------|
| **Note identity** | `(source_part_index, note_id)`: the key shared by a written note's rendered element and its audio timing. A tie continuation reuses the id of the note it continues. |
| **Playback cursor** | The red highlight that jumps note to note over whichever note is sounding. |
| **Measure Start Time** | The elapsed-seconds offset of a measure boundary. Used only for playback-order arithmetic; the UI highlights through the **Playback cursor**. |
| **Navigation expansion** | Rebuilding a score's measures into `sequence` playback order for audio (`midi::expand_navigation`). SVG and PDF always render in written order. |
| **Sequence resolution** | Turning each `sequence` label into a measure range and its part filter (`resolve_sequence`, `SequenceSpan`). |
| **Note click target** | A transparent hit rect over a note or rest that stops above its lyric row, so a syllable stays independently selectable. |
| **Lyric click target** | A transparent hit rect over one syllable, widened to its note's written span, and selected independently of the note. |
| **Part label click target** | A hit rect over a part's row label that selects everything the part sounds across its system. |
| **Lyric label click target** | The lyric-side mirror of **Part label click target**: it selects every syllable of that lyric part across its system. |
| **Bar number click target** | A hit rect over a measure's printed bar number (`BarNumberClickTarget`), which selects the measure. It has its own tag so its hover highlight doesn't light up on every note hover. |
| **Clickable element ID** | The tagged union of what a click hit (`ClickableElementId`), the input to `resolve_selection_range`. The web hit-tests and paints; all range logic is in Rust. |
| **Selection time window** | The stretch of score a cross-row range covers, from the earlier endpoint's start to the later endpoint's end. A note overlapping it is selected, so a note held across the range is never a hole. |

### Notation views

| Term | Definition |
|------|-----------|
| **Numbered notation** | The default notation, formerly called jianpu: scale degrees `1`–`7` with octave dots and duration underlines. |
| **Violin / Cello view** | Staff notation engraved by Verovio from MEI, every staff in treble clef (Violin) or bass clef (Cello). |
| **Letter-name view** | The numbered layout with degrees and chords replaced by letter names in the key at that measure (`NoteLabels::LetterNames`). |
| **Harmonica-tab view** | The numbered layout with degrees replaced by chromatic harmonica tabs in C (`NoteLabels::HarmonicaTabs`): `4` blow, `(4)` draw, `←` slide button in. Out-of-range pitches read `?` in red. |
| **Guitar chord view** | The numbered layout with letter names after each degree and a chord diagram above each chord (`NoteLabels::GuitarChords`). |
| **Guitar voicing** | One guitar chord shape from the vendored chords-db. A build step generates the lookup table, so the JSON never ships in the wasm binary. |
| **Voicing path** | The voicings the guitar chord view picks for one chord part: the least-movement sequence overall, found by dynamic programming rather than greedily. |
| **Guitar-diagram band** | The extra sub-row a system reserves at the top of a part when any of its chords has a guitar voicing. |

### Pitch and audio

| Term | Definition |
|------|-----------|
| **Pitch description** | What one selected note or chord means in letter names, in the key at its measure. |
| **Pitch drawer** | The preview's bottom drawer showing the pitch description when the selection covers exactly one note or chord. |
| **Chord tone** | One pitch a chord symbol stacks on its root, matching what playback sounds. A slash chord's bass is kept separate. |
| **Voice leading** | How playback picks each chord's inversion: the one whose upper voices move least from the part's previous chord, kept within a fixed range so a progression can't drift. |
| **Loudness balancing** | Evening out the bundled soundfont's uneven program loudness in WAV and MP3 with a CC11 per program. MIDI export stays unbalanced, since other synths use other soundfonts. |

### Files and migration

| Term | Definition |
|------|-----------|
| **Path** | A validated slash-separated name for a file or folder (`file_tree::TreePath`). See "File tree". |
| **Folder** | A shared path prefix, not a stored object. The tree is derived from file paths plus a list of empty folders. |
| **Placeholder file** | The hidden empty cloud file `<folder>/.keep` that persists an empty folder, since the worker only stores files. Users can't create that name, and it never appears in a `FileStoreState`. |
| **Origin migration** | The one-time handoff of a browser's saved data from the old web origin to the current one. See "Origin migration". |
| **Legacy name** | A name saved data used before the rename to Poentaro, still read and renamed where possible. See "Rename to Poentaro". |

### Editor tooling

| Term | Definition |
|------|-----------|
| **Rename-symbol support** | Renaming a part abbreviation or section label everywhere it's used (`src/symbols.rs`). Reference spans stay separate from error-reporting spans, which need their own shape. |

## Web integration

The React app (`web/`) runs the compiler in a dedicated worker (`web/src/worker/poentaro.worker.ts`) backed by the `poentaro-wasm` crate. The main thread sends source text and asset bytes; the worker calls WASM exports and posts structured results back.

### Storage abstraction

- Module: `web/src/storage/`
- Key types: `StorageBackend` (`web/src/storage/types.ts`) — the interface both the browser-`localStorage` backend and the D1-backed cloud backend implement; `FileStoreState` (`web/src/fileStore.ts`) remains the canonical in-memory shape used by every backend. `StorageBackend.load`/`createFile`/`duplicateFile`/`renameFile`/`deleteFile`/`restoreFile`/`saveContent` are async (to accommodate network-backed backends); `updateActiveContent` is sync with no persistence side effect. `saveContent` is the explicit call that persists the active file's content — debounce ownership lives in the hook layer, not the backend.
- `web/src/storage/localBackend.ts` — thin async adapter over `fileStore.ts`'s pure, synchronous functions (`createFile`, `duplicateFile`, `renameFile`, `deleteFile`, `restoreFile`, `updateActiveContent`). Its `saveContent` is a no-op since `useLocalStorage` already persists on every state change.
- `web/src/storage/cloudBackend.ts` — `createCloudBackend(config: CloudBackendConfig) -> CloudBackend` (`CloudBackend` extends `StorageBackend`), backed by `crates/live-share-worker`'s `/files/*` HTTP routes (D1-backed — see **Synced Share worker**'s "Storage" bullet's `files` table), called through the generated worker client (`web/src/syncedShare/workerClient.ts`, see **Synced Share worker**), the replacement for the deleted GitHub Contents API backend. `CloudBackendConfig` is just `{ token, account, workerHost, onSessionEnded? }`: `token` is the account's session token from `accountAuth.ts` below, `account` its email (which scopes the on-device outbox) (sent as `identityToken` on every request body, never as a header/query param, matching this worker's `FileShareRequest` convention), `workerHost` resolves to an origin via `syncedShareWorkerOrigin`, and `onSessionEnded` is called when an outbox lane newly halts at `NeedsSignIn` (a queued change refused as `unauthorized`), so `useStorageBackend` can clear the dead session — there is no repo/owner concept to configure, unlike the deleted GitHub backend. Contrasted with that backend: no sha-refetch-before-write — `revisionByFileId` tracks each file's last-known `revision` (populated by `load()`, advanced after every successful save) and is sent as `expectedRevision` on the next content save, with the server's atomic `UPDATE ... WHERE revision = ?` as the real CAS guard (`crate::files::classify_content_write`); rename/delete/restore are each a single atomic server-side `UPDATE`, not two sequential calls, so there's no "left at both paths" window to recover from; and a file's id is the real, durable D1 `files.id` straight from the server, not a client-side name→UUID shim. Exposes a non-interface `lastError(): CloudBackendError | null` (`{ kind: 'conflict'; currentRevision } | { kind: 'auth' } | { kind: 'network' } | { kind: 'unknown'; message }` — `'auth'` is the worker's `unauthorized` `ApiError`, meaning the session was signed out or expired; the worker rate-limits only sign-in and sign-up, so there is no `'rate-limited'` kind) and a non-interface `forceOverwrite(state)` ("Overwrite mine": realigns `revisionByFileId` to the server's last-reported `currentRevision`, then retries the save so the CAS write that previously lost the race now succeeds).
- `web/src/storage/accountAuth.ts` — the browser's account session, shared by **both** cloud file storage and Synced Share ownership (see **Accounts (email + password)** below). `useAccountAuth()` is the reactive hook (`useLocalStorage` under the hood, key `poentaro:account-auth:v2`) exposing the stored `StoredAccountAuth { token, email }`: `token` is the opaque session token the worker answered `POST /auth/sign-up`/`/auth/sign-in` with, `email` is only for display. Loading the module removes GitHub sign-in's old `poentaro:synced-share-github-auth:v1` key, whose token the worker no longer accepts.
- `web/src/storage/accountRequests.ts` — `signUp`/`signIn`/`changePassword`/`signOutSession`, the account routes through the generated worker client. Each failure becomes an `AccountFailure` (`email-taken`, `locked { retryAfterSeconds }`, `incorrect-credentials`, `session-ended`, or a worded `message`) for the dialogs to show; the rules themselves are the worker's. `web/src/components/AccountAuthModal.tsx` (sign in / create account) and `ChangePasswordModal.tsx` render them, opened through `useSyncedShareOwner`'s `accountDialog` state from the header's account chip and the "Sign in" shortcuts in `ShareModal`/`StorageSettingsModal`. Signing in never itself switches the storage backend or starts a share.
- Entry: `useStorageBackend()` in `web/src/hooks/useStorageBackend.ts` — holds the `FileStoreState` in React state and exposes `store`/`setStore`/`backend`/`isLoadingCloud`/`saveStatus`/`preference`/`switchBackend()`/`forceSave()`/`flushPendingSave()`/`refreshSaveStatus()`. `local`'s state lives in `useLocalStorage` (seeded synchronously through `localBackend`'s local-only read helpers); `cloud`'s state is fetched via `backend.load()` into a plain `useState` whenever the backend identity (kind/token) changes, by `useCloudStoreLoader()` (`web/src/hooks/useCloudStoreLoader.ts`): an `unauthorized` `ApiError` (session rejected: signed out, expired, or ended by a password change elsewhere) clears the stored `accountAuth.ts` session (as does a queued change refused the same way, via `CloudBackendConfig.onSessionEnded`, and any Synced Share request, silent or not), and any other failure is retried with exponential backoff (1s doubling, capped at 30s) and immediately on the browser's `online` event. Callers `await backend.xxxFile(store)` for structural operations (create/duplicate/rename/delete/restore — hit the backend immediately), then `setStore` the result; content edits go through the existing `setStore`/`backend.updateActiveContent` path and are separately debounced (`AUTOSAVE_DEBOUNCE_MS`, ~20s, via `use-debounce`'s `useDebouncedCallback`) into a `backend.saveContent()` call — one shared cadence for both backends, immaterial for `local` (no-op `saveContent`), kept low-frequency for `cloud`. `switchBackend(target)` persists the choice under `poentaro:storage-backend:v1` (now just `{ backend: 'local' | 'cloud' }` — no repo/owner concept to carry alongside it, since a `cloud` backend is entirely reconstructed from the one shared `accountAuth.ts` token), force-flushing (and awaiting) any pending cloud save first, and always lands on the demo file — there is no per-backend "last active file" memory. `local` vs. `cloud` is an explicit, independent choice from sign-in state: signing in via `accountAuth.ts` never itself switches `preference` — it only makes `'cloud'` selectable; losing either one (switching back to `local`, or the session being cleared by a sign-out anywhere in the app) falls back to `localBackend` on the very next render, with no extra plumbing needed. `flushPendingSave()` similarly force-flushes a pending cloud save without cancelling the debounce timer or switching backend; `App.tsx`'s `handleSelect` calls it before changing the active file tab, since `shouldScheduleAutosave` deliberately does not schedule a new save on such switches.
- `web/src/components/StorageSettingsModal.tsx` — the settings UI that makes the cloud backend reachable: a Radix Dialog offering "This browser" vs. "Cloud storage", and, when not signed in (`useAccountAuth()` returns `null`), a "Sign in" button that opens the account dialog (`AccountAuthModal`); once signed in, the account's email. Selecting "Cloud storage" while signed in calls `switchBackend({ kind: 'cloud' })` directly. Signing out lives only in the header's account chip (`ProfileControl.tsx`), which ends the session on the worker, clears it and falls back to `local`, taking down cloud storage and Synced Share ownership together. Also renders `cloudBackend.ts`'s `lastError()` as a status banner (`storageSettingsModalHelpers.ts`'s `errorBannerMessage`), and resolves `409` conflicts via `resolveCloudConflict()` (also from that helpers file), a minimal "Overwrite mine" (`CloudBackend.forceOverwrite`) vs. "Discard mine" (`backend.load()` then `updateActiveContent` with the remote content) choice — no 3-way merge. `FileSwitcher.tsx`'s file tab bar exposes the entry point (a "Storage…" button plus a saving/saved/offline status badge); `App.tsx` owns the modal's open state and threads `backend`/`isLoadingCloud`/`preference`/`switchBackend`/`refreshSaveStatus` from `useStorageBackend()` down to it.
- There is no OAuth anywhere: no proxy, no callback page, no client id. Sign-in is a plain `POST` to the worker's `/auth/*` routes.

### File tree (path rules, source-level)

- Module: `src/file_tree/` (pure Rust; exposed to the web through `crates/poentaro-wasm`, see below).
- Key types: `TreePath` (validated **Path**; `parse`, `child`, `name`, `parent`, `ancestors`), `PathError` (`Empty`, `EmptySegment`, `RelativeSegment`, `TooLong`), `FileTree { root: FolderContents }` with `FolderContents { folders: Vec<FolderNode>, files: Vec<FileNode> }`, `FolderNode`, `FileNode`, `FileTreeError::FileFolderConflict`.
- Entries: `FileTree::build(file_paths: &[TreePath], explicit_folders: &[TreePath]) -> Result<FileTree, FileTreeError>` `unique_path(desired: &TreePath, taken: &HashSet<TreePath>) -> Result<TreePath, PathError>`, and the tree-aware wrapper `FileTree::unique_path(&self, desired: &TreePath, also_taken: &[TreePath])` (also `FileTree::all_paths`).
- Store operations (`file_ops.rs`, `folder_ops.rs`, state in `store_state.rs`): pure functions over `FileStoreState { active, user_files, bin, file_ids, folders: Vec<TreePath> }`, a Rust mirror of `web/src/fileStore.ts`'s `FileStoreState` (string keys are full paths; `bin` is keyed by the full path a file was deleted from, so the persisted shape is unchanged). Each returns a new state or a typed `FileStoreError` (`InvalidPath`, `NotFound`, `Collision`, `AncestorIsFile`, `MoveIntoItself`, `DemoFileReadOnly`); nothing is mutated on error. `FileOperationContext { demo_files: Vec<DemoFile>, new_file_id }` carries the read-only demo files and a caller-generated file id.
- Entries: `create_file(state, folder: Option<&TreePath>, name, content, context)` (`name` is one trimmed segment, `.poentaro` appended when missing; a `Collision` with any sibling file, folder, demo file or bin entry is an error, nothing is auto-numbered), `create_file_error(state, folder: Option<&str>, name, context) -> Option<String>` (the `Display` text of what `create_file` would reject, shown live in the New file dialog), `duplicate_file(state, source: &TreePath, context)` (same folder, folder-scoped unique name), `rename_file(state, from: &str, to_raw: &str, context)` (rename and move are the same operation; keeps content and file id; `.poentaro` appended when missing; destination colliding with any file, folder, demo file or bin key is a `Collision`), `delete_file(state, path, context)`, `restore_file(state, bin_key, context)` (back to the original path, folders recreated implicitly; if the path is taken by a file/folder/demo a unique name in that folder is used; fails with `AncestorIsFile` if a parent is now a file), `move_folder(state, from: &TreePath, to: &TreePath, context) -> Result<FolderMoveOutcome { state, changes: Vec<PathChange { kind: PathKind, old_path, new_path }> }, FileStoreError>` and `delete_folder(state, folder, context)`.
- `move_folder` is all-or-nothing: it fails (changing nothing) if the folder does not exist, `to` is inside `from`, a parent of `to` is a file, any resulting path is invalid or over 1024 bytes, or `to` / any resulting path already exists as a file, folder, demo file or bin key outside the moved subtree (folders never merge; moving into folder B means `to = B/<name>`). File ids and explicit empty subfolders move along; bin entries stay under their old paths. `delete_folder` bins each file individually under its full path and removes the folder with its explicit subfolders.
- Wasm/TS bridge: `crates/poentaro-wasm/src/component/guest_file_store.rs` converts WIT values (`file-store-state`, `file-operation-context`, `file-store-response`, `folder-move-response`, `file-tree-response`; the exports `file-store-create-file`, `-duplicate-file`, `-rename-file`, `-delete-file`, `-restore-file`, `-move-folder`, `-delete-folder`, `-tree`, `-merge-migrated`) to/from these types and nothing else. `web/src/fileStoreWasm.ts` converts `FileStoreState` (maps as ordered key/value lists) and `web/src/fileStore.ts`'s `createFile`/`duplicateFile`/`renameFile`/`deleteFile`/`restoreFile`/`moveFolder`/`deleteFolder`/`fileTreeRows` call straight into Rust. These stay synchronous but need the component initialised (`poentaroWasm()` throws otherwise): `localBackend` and `cloudBackend` `await ensureWasmInit()` before calling them. A rejected single-file operation returns the same state object (callers test `===`); `moveFolder` returns `{ ok, state, changes }` or `{ ok: false, failure }`. `FileTree::rows` (`TreeRow`) is the flattened, sorted display order. `FileStoreState.folders` (explicit empty folders) is optional in `fileStorePersistence.ts`: absent in old stores, only written when non-empty, no version bump. `importSharedFile`'s naming is still the flat TS `uniqueName`.
- File list view (`src/file_tree/view.rs`): `FileTree::view_rows(&TreeViewInput { active, expanded, demo_file_names, shares: &ShareOverview, query: &str }) -> Vec<TreeViewRow { key, name, depth, kind: TreeViewRowKind, expanded, file_count, is_active, share_id, shared_via, check }>` is what the web file list renders (`TreeViewInput` also carries `selected`, the ticked file paths, which fill each row's `check`). A non-blank `query` (the file browser's search box) keeps only rows whose name contains it (case-insensitive, `.poentaro` ignored) with their ancestor folders forced open; a folder matching by its own name keeps everything inside it, and demo files are searched too. It hides the contents of collapsed folders, forces the ancestors of the active file open, gives an expanded empty folder an `EmptyFolderHint` row, and appends the read-only `Demo/` row last (`DemoFolder`, collapsed unless its expansion key `Demo/` (`DEMO_EXPANSION_KEY`) is in `expanded` or the active file is a demo file; `DemoFile` rows are keyed by bare demo file name). `active_label(active, demo_file_names) -> ActiveLabel { folder_names, file_name }` splits the active file for the toolbar button's muted folder prefix. Wasm exports `file-store-tree-view` / `file-store-active-label`; TS wrappers `fileTreeView` / `activeFileLabel` in `fileStore.ts`, rendered inside the full-screen `FileBrowserModal.tsx` (search box, New / New folder / Import buttons; opened by `FileSwitcherTrigger.tsx`) by `web/src/components/FileSwitcher.tsx` via `hooks/useFileTreeView.ts` (waits for the main-thread wasm). The set of expanded folder keys is a per-browser UI preference (`hooks/useExpandedFolders.ts`, localStorage `poentaro:file-tree-expanded:v1`), not part of `FileStoreState` and never synced.
- Multi-select (`src/file_tree/selection.rs`): the file browser's Select mode keeps only the raw ticked **file paths** (`web/src/hooks/useFileSelection.ts`). Rust decides everything else: a row's `CheckState` (`Unchecked`/`Checked`/`Mixed`; a folder is checked when all its files are, and an empty folder never is), `FileTree::toggle_selection` / `select_all`, and `selection_units(state, selected) -> Vec<SelectionUnit { path, is_folder, file_count }>` (every fully-ticked folder whose parent is not also fully ticked, then the remaining ticked files). `selection_move_plan(state, selected, folder, context) -> SelectionMovePlan { moves: Vec<SelectionMove { from, to, is_folder }>, error }` lays the units out in the target folder keeping their names and applies them in order to a scratch store, so a clash between two ticked items is reported before anything moves; `selection_delete_summary(state, selected, shares) -> SelectionDeleteSummary { units, lines, message, share_warning }` is the confirmation text. Wasm exports `file-store-selection-all` / `-toggle` / `-units` / `-move-plan` / `-delete-summary` (`guest_file_selection.rs`), TS wrappers in `fileStoreQueries.ts`. `FileSelectionBar.tsx` (shown inside `FileBrowserModal` in select mode) wires Rename (one unit only, reusing `RenameFileDialog`), Move to… (`MoveToDialog` with `hasCurrentFolder={false}`) and Delete (`DeleteSelectionDialog`); `useFileOperations`' `handleMoveSelection` / `handleDeleteSelection` apply a plan one backend call after the other (`renameFile`/`moveFolder`, `deleteFile`/`deleteFolder`), each from the store the previous one returned, so the cloud backend queues them in the outbox exactly like single operations.
- File actions (`file_ops.rs`/`folder_ops.rs`/`move_targets.rs`): `create_folder(state, parent: Option<&TreePath>, name, context)` adds an explicit empty folder `name` (one trimmed segment; a `Collision` with any sibling file, folder or bin entry is an error, nothing is auto-numbered) and leaves the active file alone; the web app asks for the name in `NewEntryDialog` (also used for New file) first. `move_targets(state, excluded_folder: Option<&TreePath>) -> Vec<MoveTarget { path, name, depth }>` lists the "Move to…" picker (the top level with an empty `path`, then every folder depth-first); passing a folder to `excluded_folder` drops it and its descendants so a folder cannot be moved into itself (the folder-move UI reuses this). `move_destination(source, folder)` is where a path lands in a chosen folder, and `rename_error(state, from, to_raw, context) -> Option<String>` is the `Display` text of the error `rename_file` would return, used to show clashes and invalid paths inline (collision, empty segment, `..`, too long) before anything changes. `ActiveLabel.folder_path` is the folder next to which "New"/"New folder" create (none for the top level and for demo files). Wasm exports `file-store-create-folder`, `-create-file-error`, `-move-targets`, `-move-destination`, `-rename-error`; TS wrappers `createFolder` in `fileStore.ts` and the read-only `fileMoveTargets`/`moveDestination`/`renameError` (with `fileTreeRows`/`fileTreeView`/`activeFileLabel`) in `web/src/fileStoreQueries.ts`; UI in `FileSwitcher.tsx` (`···` menu), `MoveToDialog.tsx`, `RenameFileDialog.tsx` and the inline rename in `FileTabName.tsx`. `StorageBackend` gained `createFolder(state, parent?)` and `createFile(state, folder?)`; the cloud backend stores empty folders as placeholder files (see below).
- Folder actions and the bin (`folder_queries.rs`, `bin_entries.rs`): `create_folder_error(state, parent: Option<&str>, name, context) -> Option<String>` is the `Display` text of what `create_folder` would reject, shown live in the New folder dialog; `folder_move_error(state, from, to_raw, context) -> Option<String>` is the `Display` text of what `move_folder` would reject (trimmed `to_raw`), used by the folder Rename and Move to dialogs; `folder_delete_summary(state, folder, shares) -> Option<FolderDeleteSummary { file_count, message, share_warning }>` is the Delete folder confirmation text (`None` for an unknown folder; `share_warning` is the extra paragraph when the folder, a folder inside it, an enclosing folder or a file inside has a live link); `folders_to_reveal(path) -> Vec<String>` is the folder and its ancestors, outermost first (what the list expands after a create/move); `bin_entries(state) -> Vec<BinEntry { key, name, location }>` splits each bin key into the file's own name and the folder it was deleted from (sorted by name), and `key` is what `restore_file` takes. Wasm exports `file-store-folder-move-error`, `-folder-delete-summary`, `-folders-to-reveal`, `-bin-entries`; TS wrappers `folderMoveError`/`folderDeleteSummary`/`foldersToReveal`/`binEntries` (and `parentFolder`, which asks `activeFileLabel` for any path's folder) in `fileStoreQueries.ts`. UI: folder rows (not the read-only `Demo/` row) carry a `···` menu (`FolderRowMenu.tsx`: New file here, New folder here, Import here, Rename, Move to…, Delete folder; hover on desktop, always visible on touch and phone widths, nested drawer on phones; Share folder sits between Move to… and Delete folder; a shared folder instead leads with Copy share link and Sharing settings, a folder inside a shared one with Shared via <name>), whose Rename is `RenameFileDialog` with `kind="folder"`, Move to is `MoveToDialog` fed `fileMoveTargets(store, folder)` (the folder and its descendants are not offered), and Delete is `DeleteFolderDialog`; `BinModal` shows each entry's name with its location muted. `StorageBackend` gained `moveFolder(state, from, to)` and `deleteFolder(state, folder)`; the local backend calls Rust, while the cloud backend renames/trashes each file inside on its own (one outbox message per file, no batch) and renames/trashes the placeholder file of every explicit folder inside. `useFileImport`'s `handleImportFile(file, folder?)` imports into a folder (the path comes from `moveDestination`).
- Folder sharing, owner side (`folder_share.rs`, `folder_snapshot.rs`, `crates/poentaro-wasm/src/folder_share_payload.rs`; the worker half is under "Folder shares" below). `ShareOverview { files: Vec<FileShareLink { share_id, file_id, name }>, folders: Vec<FolderShareLink { share_id, folder }> }` mirrors `POST /shares/overview`; the web keeps the last reply (`hooks/useFolderShares.ts`, refreshed on load, after start/stop, when the file paths change and again when the cloud save settles because the worker re-points a renamed folder's link when the outbox batch is delivered) and hands it to Rust unchanged. `folder_share_info(state, overview, folder) -> Option<FolderShareInfo { name, file_count, folder_count, share_id, shared_via: Option<SharedVia { folder, name, share_id }>, own_link_warning }>` drives the folder Share dialog, `folder_delete_share_warning` the delete warning, and `view_rows` the marker and "via X" tag. Static folder link: `folder_snapshot(state, folder) -> Option<FolderSnapshot { folder, files: Vec<SnapshotFile { path, content }>, empty_folders }>` (paths relative to the folder); `FolderSnapshot::from_json` validates the folder name (one segment) and every full path with `TreePath` (1024-byte rule) and rejects file/folder conflicts; `poentaro-wasm` brotli-compresses and base64url-encodes it (`encode_folder_snapshot`, `None` when the link would exceed `MAX_STATIC_FOLDER_LINK_CHARS` = 8000 characters or the JSON exceeds `MAX_UNCOMPRESSED_SNAPSHOT_BYTES`) into the `#share-folder=<payload>` hash; `decode_folder_snapshot` is the inverse and is what the viewer task reads. The old `#share=` single-file payload is untouched. Wasm exports `file-store-folder-share-info`, `file-store-folder-static-share` (`folder-static-share { file-count, folder-count, suffix, too-long }`) and `decode-folder-share-suffix`; `file-store-tree-view` and `file-store-folder-delete-summary` take the overview as an argument. TS: `ShareModal.tsx` is generic over `kind: 'file' | 'folder'` (static tab with the size-guard banner and a "Use synced link" button, synced tab with the own-links warning and an inline Stop Sync confirmation), opened for files from the actions menu and for folders from `FolderRowMenu`; `hooks/useStaticShareLink.ts` fetches the static link from wasm. Local-backend folders show the same cloud-only note as files.
- Folder sharing, viewer side (`shared_folder.rs`): a viewer of a folder link (`?s=` synced, or `#share-folder=` static) sees the **shared folder viewer**: the shared folder as the root of a read-only tree. `shared_folder_view(snapshot: &FolderSnapshot, active, expanded) -> Result<Vec<TreeViewRow>, FileStoreError>` builds a `FileStoreState` keyed by the snapshot's relative paths (placeholder files skipped, empty folders kept) and renders it through `view_rows` with no Demo files and no share overview; `shared_folder_first_file(snapshot) -> Option<String>` is the file opened first (the first file row of the fully expanded tree: folders before files, depth-first, alphabetical); `import_shared_folder(state, snapshot, file_ids, context) -> Result<FileStoreState, FileStoreError>` copies the snapshot into the viewer's own files under `unique_folder_path(snapshot.folder, ...)` (`Hymns`, then `Hymns 2`, `Hymns 3`, ...) with the caller's file ids, empty folders kept as explicit folders, the first file active. An invalid snapshot is `FileStoreError::InvalidSharedFolder` (WIT kind `invalid-path`). Wasm exports `shared-folder-view`, `shared-folder-first-file`, `file-store-import-shared-folder`; TS wrappers in `web/src/sharedFolder.ts`. Web: `hooks/useSyncedShareViewer.ts` fetches `GET /shares/{id}` and exposes the folder (or a status: `loading`, `synced`, `ended`, `unreachable`, `too-large` for the 413 `shared_folder_too_large`, `none`); `hooks/useSharedFolderViewer.ts` holds only the raw viewer state (selected path, expanded folder keys) for either link kind and returns a `SharedFolderViewer`; `SharedFolderSwitcher.tsx` renders the rooted list through `FileTreeList` with `readOnly`; `SyncedShareBanner` / `SharedPreviewBanner` carry "Import this folder" and Discard (which leaves a link in any state: loading, shown, ended or failing); `SharedViewerNotice.tsx` replaces the score when a link has ended, cannot be reached or is too large. `StorageBackend.importFolder(state, snapshot)` runs `import_shared_folder` (local) and additionally queues one `create-file` per file plus a placeholder per empty folder (cloud); afterwards the viewer leaves viewer mode (clears `?s=` / the hash). Changes by the owner appear when the viewer reloads; there is no polling.
- Placeholder files (`placeholder.rs`, `cloudBackendPlaceholders.ts`): the cloud (worker + D1) knows only files, so an explicit empty folder `A/B` is persisted as the file `A/B/.keep` with empty content; the worker and schema are unaware of folders. Rust owns the convention: `PLACEHOLDER_LEAF`, `is_placeholder_path(path)`, `placeholder_path(folder) -> Result<TreePath, PathError>`, and `classify_cloud_listing(&[ListedEntry { id, name, trashed }]) -> ListingClassification { folders, trashed_folders, hidden_file_ids }` (`FolderPlaceholder { folder, file_id }`), exposed as WIT `file-store-classify-cloud-listing`, `-placeholder-path`, `-is-placeholder-path`. Users cannot create the reserved leaf: `rename_file`, `rename_error` and `move_folder` fail with `FileStoreError::ReservedName` (WIT kind `reserved-name`); the placeholder logic itself does not go through those. Where the placeholder's file id lives: not in `FileStoreState` (local mode never has placeholders, and `file_ids`/`user_files`/`bin` stay placeholder-free). The cloud backend's `load()` runs the raw listing (after `applyPendingStructure`, so a still-queued create-folder shows at once, also after a reload) through `classify_cloud_listing`, drops every placeholder row (active or trashed) from the file list, sets `FileStoreState.folders` from the active ones, and keeps a `folder -> { fileId, trashed }` map inside the backend (`createPlaceholders`). `createFolder` queues `CreateFile` `<folder>/.keep`; folder rename/move queues one `RenameFiles` batch covering the moved files and the explicit-folder placeholder moves (`moveFolder` in `cloudBackendFolders.ts`; `rename()` in `cloudBackendPlaceholders.ts` returns the placeholder's `RenameMove`); `deleteFolder` queues `TrashFile` for every explicit folder removed. Because names are unique across trashed rows too, re-creating a deleted folder restores its trashed placeholder, and renaming a folder onto a deleted folder's name trashes the old placeholder and restores that one. A placeholder stays when real files are later added to the folder (harmless; the folder is explicit from then on). A `name_taken` answer for a placeholder create/rename (another device made the same folder) counts as delivered instead of becoming a visible `.keep 2.poentaro`.
- Sorting compares names case-insensitively (lowercased), falling back to a plain comparison for a deterministic total order; this approximates the TS `localeCompare` it replaces.
- `unique_path` mirrors the old `fileStore.ts` `uniqueName`: only siblings of `desired` (same parent folder) count as taken; a collision becomes `stem 2.ext`, `stem 3.ext`, ...; a name with no extension (or only a leading dot) gets `.poentaro`.
- `path_after_collision(rejected: &TreePath) -> Result<TreePath, PathError>` (also in `unique_name.rs`) is the cloud outbox's retry name when the worker answers `name_taken` and only the rejected path is known: it steps over a trailing ` N` on the stem (`a.poentaro` -> `a 2.poentaro`, `a 2.poentaro` -> `a 3.poentaro`), keeps the folder prefix, and goes through `unique_path`. Exposed as the WIT export `file-store-name-after-collision` (`guest_file_store.rs`); `web/src/storage/outbox/sendMessage.ts` calls it for a `create-file`/`rename-file`/`restore-file` `name_taken` answer. The old TS `suggestNameAfterCollision` is deleted. A folder move is one `rename-files` batch (see Cloud outbox): when the worker answers `names_taken`, `sendMessage.ts` maps each listed name to a `NameAdjustment { rejected, replacement }` through the same export (`replacement` is none for a `.keep` placeholder path, whose move is dropped), and the outbox rewrites the matching moves and resends the whole batch.

### Source editing

- Module: `src/source_edit/`
- Entries: `source_edit::update_part_declaration(source, abbreviation, settings: &PartSettings, instruments: &[InstrumentInfo]) -> Option<String>`; `source_edit::shift_part_octave(source, abbreviation, delta: i8) -> String`; `source_edit::shift_range_octave(source, ranges: &[ByteRange], delta: i8) -> RangeEditResult`; `source_edit::toggle_range_slur(source, ranges: &[ByteRange]) -> RangeEditResult`; `source_edit::toggle_range_tie(source, ranges: &[ByteRange]) -> RangeEditResult`; `source_edit::parse_metadata_fields(source, preset_name: Option<&str>) -> MetadataFields`; `source_edit::update_metadata_field(source, preset_name: Option<&str>, edit: MetadataEdit) -> String`; `source_edit::list_presets(source) -> Vec<PresetSummary>`; `source_edit::update_preset_field(source, preset_name, edit: &PresetEdit) -> String`; `source_edit::add_preset_section(source, preset: NewPreset) -> String`; `source_edit::follow_preset_rename(previous_source, source, preset_name) -> Option<String>`; `source_edit::set_preset_part_shown(source, preset_name, all_parts, part: &PartShown) -> String`; `source_edit::set_preset_part_soloed(source, preset_name, all_parts, part: &PartSoloed) -> String`; `source_edit::new_preset_name_error(source, name) -> Option<String>`
- `parse_metadata_fields`/`update_metadata_field` (`src/source_edit/metadata_edit.rs`) back the Edit Metadata modal instead of any TypeScript parser. Both edit the section of the preset called `preset_name` (the one chosen in the preset menu), or `metadata` when `None`. `MetadataFields { metadata: ParsedMetadata, directive_row_offset: Option<String>, inherited: ParsedMetadata }` is that section as read by `read_metadata_field` (all-unset when there's no such section), with `directive_row_offset` also kept verbatim so a half-typed value survives the form's re-parse, and `inherited` the song's `metadata` when editing a preset (what its unset fields fall back to; empty otherwise). `MetadataEdit` is one typed field edit (`Text`/`Number`/`Flag`/`DirectiveRowOffset`/`Style { kind, value: TextStyleComponentValue }`/`ResetStyle(kind)`; `None` removes the field, so under a preset it falls back to `metadata`). `update_metadata_field` applies it and re-emits the whole section in canonical order (a preset's `visible_parts`/`soloed_parts`/`notation` lines first, then each text kind's content then its style, the numeric keys, the remaining styles, the flags, `directive_row_offset`, then one blank line), leaving other sections untouched; no such section → `source` unchanged.
- The preset menu's source edits live in `src/source_edit/preset_edit.rs`. `list_presets` reads each `preset` section's name and `notation` as a `PresetSummary { name, notation: Option<Notation> }`, from the sections alone (not a full parse), so a half-typed score doesn't empty the menu; like the parser it skips `"Default"` and repeated names. `update_preset_field` rewrites one preset's line in place for a `PresetEdit` (`VisibleParts(Vec<String>)`, comma-separated, empty hides every part; `SoloedParts(Vec<String>)`, empty removes every such line; or `Notation(Notation)`): the last such line, which is the one the parser keeps, else a new line under the header. Unlike `update_metadata_field` it doesn't re-emit the section, so the user's formatting stays. `add_preset_section` writes a `NewPreset { name, visible_parts, soloed_parts, notation }` (no `soloed_parts` line when empty) after the last `preset`, else before the first `score`/`sequence`, else at the end. `follow_preset_rename` keeps the chosen preset across an edit: the same name while the source still has it, else the preset now at its position when the edit renamed it (same number of presets, a new name), else `None`. `set_preset_part_shown` is a part pill under a preset: it rewrites `visible_parts` with `PartShown { abbreviation, shown }` applied, in `all_parts` order (a preset without `visible_parts` starts from every part). `set_preset_part_soloed` is a solo button under a preset: it rewrites `soloed_parts` with `PartSoloed { abbreviation, soloed }` applied, in `all_parts` order, and removes the line once nothing is soloed. `new_preset_name_error` is the New preset dialog's check: empty, quotes or line breaks, `Default`, or a name already taken. An unknown preset name leaves `source` unchanged / returns `None`.
- `update_part_declaration` rewrites a single `parts` declaration line in place from a `PartSettings { mode: PartMode, range_instrument: Option<RangeInstrument>, program: Option<u8>, volume: u8, octave_offset: i8 }` (`PartMode` is `Chords`/`Notes`/`Percussion`/`Follow { target }`). The **Range instrument** is written right after the kind, before the sound. `program` is written as its quoted **Sound label** (see `src/sound_label.rs`), named from `instruments` or, on a percussion part, the GM percussion map. The volume and octave offset are first clamped into `PART_SETTING_LIMITS` (see **Part setting limits** in the glossary), so the modal can never write an out-of-range suffix. A volume of `DEFAULT_PART_VOLUME` (100) or an octave offset of `DEFAULT_PART_OCTAVE_OFFSET` (0) is written as no suffix, the same way `list_part_declarations_from_source` reads a missing suffix, so a declaration's `settings` round-trip unchanged. Used by the Edit Parts modal instead of any TypeScript parser.
- `shift_part_octave` bulk-rewrites the `'`/`,` octave marker on every `ScoreEvent::Note` belonging to one part in `score` (the "notation octave" control in the web editor — distinct from `update_part_declaration`'s MIDI-only `octave_offset`, which never touches rendered notation). Parses via `parser::parse` to read each note's resolved `octave: i8` and full-token `Span` (`ParsedTimedTrack::measure_slots`), then splices new marker text into the raw source back-to-front by span. Infallible / best-effort, mirroring `update_part_declaration`'s "not found → unchanged" convention: an unknown abbreviation, a `follow [X]` part (no notes of its own), `delta == 0`, or a parse failure all return `source` unchanged. No clamping on the resulting octave.
- `shift_range_octave` is `shift_part_octave`'s counterpart for the editor toolbar's "Octave up"/"Octave down" buttons: instead of one named part, it shifts every note across every part whose `Span` overlaps any of the (generally disjoint — e.g. a clicked part label's multicursor selection) `ranges` — the web editor's current text selection, so a selection crossing a measure or part boundary shifts notes in every measure/part it touches. Shares `shift_part_octave`'s edit-collection/splicing helpers, `follow [X]`-safe (no events to match), and unchanged-on-`delta == 0`/parse-failure conventions. Unlike `shift_part_octave`, it returns a `RangeEditResult { source: String, ranges: Vec<ByteRange> }` (`src/source_edit/range_edit.rs`, shared by every selection-scoped edit along with `ByteRange` and the `SourceEdit { span, replacement }` edit type): `ranges` is the caller's own input `ranges`, remapped forward through the edits to their new `[start, end)` byte spans *in the new source* (`remap_ranges`/`remap_offset`, biasing a range's start to an overlapping edit's new start and its end to that edit's new end), preserving the input ranges' count and shape — a contiguous multi-note range selection stays one contiguous range, a disjoint multicursor selection stays that many disjoint ranges — rather than exploding into one range per shifted note. This lets the editor restore its selection in the same synchronous call that applies the new source (`web/src/types.ts`'s `EditorHandle.replaceContentWithSelections`) — see `HANDOFF-octave-toolbar-part-label-selection-bug.md`.
- `toggle_range_slur` backs the editor toolbar's single "Slur/Unslur" toggle button, over the same `ranges` and returning the same `RangeEditResult`. If any closed `(…)` group (`ParsedTimedTrack::slur_groups`) in any part overlaps the selection, it deletes the `(`/`)` of every such group (unslur); otherwise, for each part with at least two selected `Note`/`Chord` events, it inserts `(` before the first and `)` after the last (slur) — so a part-label click's per-measure disjoint ranges become one cross-measure group. Parts with fewer than two selected notes are skipped (a one-note group is a `group_too_few_notes` warning). Identical edits surfaced by more than one track (desugaring can share source spans) are applied once.
- `toggle_range_tie` backs the editor toolbar's single "Tie/Untie" toggle button, with the same `ranges`/`RangeEditResult` shape and dedup as `toggle_range_slur`. If any selected `Note`/`Chord`/`PercussionHit` carries a `~` (`tie_to_next_span`), it deletes every such `~` (untie) — replacing it with a space instead when the next character is a digit or repeat atom (`_`/`=`/`r`), so e.g. `5~_` becomes `5 _`, not the eighth note `5_`. Otherwise, per part, it inserts `~` at the end of each selected event whose immediately following event (rests interrupt) has the same pitch (note degree+accidental+octave, identical chord symbol, or any percussion hit) and is also selected — or, when the part has exactly one selected event, into whatever follows it — since a tie between different pitches is an error.

### Split-track export

- Module: `src/split_track.rs`
- Entries: `write_split_pdfs_from_source(source, filename, base_name, selection: SplitSelection { tracks, preset_name }, fonts, note_labels: NoteLabels) -> Result<Vec<SplitPdfEntry>, IrrecoverableError>` (`pdf` feature); `write_split_midis_from_source(source, filename, base_name, selection) -> Result<Vec<SplitFileEntry>, IrrecoverableError>` (`midi` feature); `write_split_wavs_from_source(source, filename, base_name, selection, sf2_bytes) -> Result<Vec<SplitFileEntry>, IrrecoverableError>` (`wav` feature); `write_split_mp3s_from_source(source, filename, base_name, selection, sf2_bytes) -> Result<Vec<SplitFileEntry>, IrrecoverableError>` (`mp3` feature)
- Each parses/compiles the source once (with the preset's metadata), then renders/synthesizes one output per track. `split_track_names(source, filename, score, selection)` picks the tracks: `selection.tracks` when given, else every score track narrowed to the preset's `visible_parts`. Under a preset, `SplitSelection::file_base_name` labels the files `<base_name> (<preset>) - <part>`, the way the web labels a single export `<base_name> (<preset>)`.
- Key types: `SplitPdfEntry` (`track_name`, `filename`, `pdf`), `SplitFileEntry` (`track_name`, `filename`, `bytes` — shared by the MIDI, WAV, and MP3 variants)
- `zip_split_pdfs(entries: &[SplitPdfEntry])` / `zip_split_entries(entries: &[SplitFileEntry])` archive the entries into a single ZIP (`Vec<u8>`), one file per track named via `split_track_filename`.
- `src/lib.rs`'s `write_midi_from_source_filtered(source, filename, enabled_tracks, instruments) -> Result<Vec<u8>, IrrecoverableError>` (`midi` feature) is the whole-score counterpart used by `generate_midi`, mirroring `write_pdf_from_source_with_visibility` and `write_wav_for_measure_range_from_source`.
- `midi::write_midi_with_metronome(score: &Score, metronome: bool) -> Result<Vec<u8>, IrrecoverableError>` (and `write_midi_for_measure_range_with_metronome`) is `write_midi` plus, when `metronome` is `true`, a click on GM percussion channel 9 at every beat (`midi::metronome::metronome_events`): key 34 (Metronome Bell) on the first beat of each measure, key 33 (Metronome Click) on the others. The beat length comes from the time signature carried forward from the last measure that set one (4/4 when none did; compound 6/8, 9/8, 12/8 beat on the dotted quarter), and `build_measure_range_score` carries the time signature in from before the range just as it does BPM and key. `write_midi` is the same call with `metronome: false`. `audio_source::write_wav_from_source_filtered` takes a trailing `metronome: bool`, and `MeasureRangeAudioOptions::metronome` does the same for `write_wav_for_measure_range_from_source` (the MP3 entry points always pass `false`); the WASM `generate-wav`/`generate-wav-for-measure-range` exports take a `metronome` parameter after `enabled-tracks`. The web app's "Tick" toggle (`components/MetronomeToggleButton.tsx`, state in `useMeasureAudioPlayback`) only passes the on/off flag with each `generateMeasureRangeAudio` request; whole-score preview audio (`generateAudio`) never has the click.

### Part declarations (source-level)

- Module: `src/part_info.rs` (re-exported from `src/lib.rs`)
- Entry: `list_part_declarations_from_source(source, filename, instruments) -> Result<Vec<SourcePartDeclaration>, IrrecoverableError>`
- Backed by `parser::anyfix::read::source_part_declarations()` — returns **raw** fields from each declaration (before `follow [X]` inheritance) as `SourcePartDeclaration { abbreviation, display_name, line_number, settings: source_edit::PartSettings, sound_label: Option<String> }`, with a missing volume/octave suffix read as its default, an out-of-range one already clamped into `PART_SETTING_LIMITS`, and `sound_label` the display-only **Sound label** for `settings.program`. The Edit Parts modal displays these values.

### Rename-symbol support (source-level)

- Module: `src/symbols.rs` (re-exported from `src/lib.rs`)
- Entries: `collect_symbols(document: &ParsedDocument) -> Vec<Symbol>`; `rename_edits(document: &ParsedDocument, kind: SymbolKind, old_name: &str, new_name: &str) -> Vec<TextEdit>`
- Key types: `SymbolKind` (`Abbreviation` — parts and groups share one namespace, so both kinds of declaration collect under it — or `SectionLabel`), `Symbol { name, kind, occurrences: Vec<SymbolOccurrence> }`, `SymbolOccurrence { span, role: OccurrenceRole }` (`Declaration` or `Reference`), `TextEdit { span, replacement }`
- Built entirely on spans `ParsedDocument` already tracks (see **Rename-symbol support** in the glossary for which fields) rather than a separate parse pass; `rename_edits` is `collect_symbols` plus a lookup and a flat map from occurrence to replacement text, so it never mutates source itself — callers apply the edits.

### Keyword highlighting (source-level)

- Module: `src/highlight.rs`
- Entry: `highlight::highlight_tokens(source: &str) -> Vec<HighlightToken>`; `highlight::section_headers(source: &str) -> Vec<SectionHeader>`
- Key types: `HighlightKind` (`SectionHeader`/`MetadataKey`/`PartKind`/`DirectiveKey`), `HighlightToken { kind, span }` (see **Highlight token** in the glossary), `SectionHeader { kind: SectionKind, span }` (each section the reader finds, with the same span as its `SectionHeader` highlight token)
- Every token comes from the parser's own reading, not a separate grammar: `src/highlight/anyfix_highlight.rs` walks the anyfix tree for section names, metadata keys and part kinds, leaving a field the reader reported as unknown uncolored, and reads the score string's directive lines with `interleaved_parser::directive_keyword_spans` (tokens classified by `score::directive_keyword::classify_directive_token`, the same classifier `parse_directive_line` dispatches on).
- Web: `registerPoentaroSemanticTokensProvider` (`web/src/monacoSemanticTokensProvider.ts`) feeds these to Monaco as semantic tokens (`semanticHighlighting.enabled`), encoded by `encodeSemanticTokens` (`web/src/semanticTokens.ts`); the legend and colors are `editorTheme.ts`'s `highlightKindStyles: Record<HighlightKind, …>`. The Monarch tokenizer (`monacoPoentaroLanguage.ts`) keeps only vocabulary-free rules (strings, comments, `[Abbrev]` tags, operators).
- Web: the Edit Parts / Edit Metadata CodeLens (`registerSectionCodeLensProvider`, `web/src/monacoSectionCodeLensProvider.ts`) places one lens per `section_headers` entry whose `SectionKind` has a command, so lenses sit exactly on the sections the parser reads.

### Score formatting (source-level)

- Module: `src/format_source.rs` (re-exported from `src/lib.rs`)
- Entry: `format_score(source: &str) -> String`
- A normal-editor toolbar action: per `score` measure group, drops `[Key]` data lines that are entirely redundant with implicit-fill (an all-rest line for a `Notes`/`Chord`-role occurrence, an all-`_` line for a `Lyrics`-role occurrence), sorts the surviving `[Key]` lines into `parts` declaration order, and collapses whitespace to single spaces on every surviving directive/data line. A key's droppable lines must be a *trailing* run (dropping an earlier lyric part out from under a later one with real content would shift it into the wrong slot) and dropping is blocked outright if it would leave the group with zero data lines (a parse error). `follow [X]` parts are excluded from redundant-line dropping (an explicit rest there is real content, not filler) but still take part in sorting.
- Sorting groups a key's surviving lines into one block that moves together, regardless of how interleaved with other keys' lines it was in the source; a lyric part's `[Key]` line (see **Lyric part** in the glossary) sorts right after its target's notes line, in declaration order. A block for an unrecognised key, or a bare unprefixed line, keeps its original relative position, ordered after every recognised part's block.
- Classification/removability is hand-rolled directly from each key's raw per-group line count (mirrors `desugar::roles_for_group`'s static-kind branch without invoking it), and the output is spliced together directly from the filtered/reordered/normalized raw lines. `desugar::desugar_groups` is still run once over the filtered groups as a best-effort validity check (falling back to `source` unchanged on `Err`), but its materialized output is discarded. Real implicit-fill for the dropped lines happens later, ordinarily, whenever the resulting `.poentaro` source is actually parsed — exactly as it already does for any measure group that omits a part's line.
- Infallible / best-effort, mirroring `source_edit::update_part_declaration`'s "not found → unchanged" convention: a missing/empty `parts` or `score` section, or an internal parse failure, returns `source` unchanged.
- The per-measure work is `format_source::format_measures(score_content: &str, score_offset: usize, declarations: &[PartDecl]) -> Option<String>` (`None` when the result would no longer desugar), which `format_anyfix` also runs.
- Anyfix documents (see **Anyfix document** in the glossary): `format_anyfix::format_anyfix(source: &str) -> String` (`src/format_anyfix.rs`) runs anyfix-expr's canonical `anyfix_expr::format` over the whole document, then `format_measures` over the score string (with the part declarations read from the document), then `anyfix_expr::format` again to indent the rewritten string. A score string holding a `//` comment is skipped. `document::formatted` (the toolbar action, and the CLI's `poentaro format <files>` via `cli::format_file`) picks `format_anyfix` or `format_score` by content.

### Part stream editing (source-level)

- Module: `src/part_stream/` (`pub mod part_stream`): `mod.rs` (public API, `join_chunks`), `document.rs` (`ScoreDocument` / `ScoreMeasure` (`start`..`end`: the group's lines) / `ScoreLine` / `GroupLine`), `editable.rs` (eligibility, `StreamGrammar`), `extract.rs`, `split.rs`, `lyrics.rs`, `chord_lyrics.rs`, `apply.rs`, `format.rs` (live re-barring), `focus.rs` (ghost lines), `selection.rs` (selections between the stream and the draft source), `timing.rs` (`TimedLineKind`, `TimedCluster` / `timed_layout`, `rest_fill`), `diagnostics.rs` (stream-only errors: `|` unless `AllowedMarkers` allows it, misplaced `@`, inline directives); tests in sibling `*_tests.rs` files
- Entries: `list_streamable_parts(source: &str) -> Vec<StreamablePart>`, `extract_part_stream(source: &str, abbreviation: &str) -> Result<PartStream, PartStreamError>`, `apply_part_stream(source: &str, abbreviation: &str, stream: &str) -> PartStreamApplyResult`, `format_part_stream(source: &str, abbreviation: &str, stream: &str, cursors: &[StreamCursor]) -> FormattedPartStream`, `part_focus_measures(source: &str, abbreviation: &str, toggles: &PartToggleState) -> Result<Vec<FocusMeasure>, PartStreamError>`, `part_stream_ranges_to_source(source: &str, abbreviation: &str, stream: &str, ranges: &[Span]) -> Vec<Span>`, `source_ranges_to_part_stream(source: &str, abbreviation: &str, stream: &str, ranges: &[Span]) -> Vec<Span>`
- Key types: `StreamablePart` / `StreamablePartKind` (part picker rows, with `ineligible_reason`), `PartStream` (`text` + `measure_starts`), `PartStreamError` (`SourceDoesNotParse` / `UnknownPart` / `Ineligible`, `describe()` for the UI), `StreamDiagnostic` / `StreamDiagnosticSeverity` (spanned in stream coordinates), `PartStreamApplyResult` (`source: Option<String>`, `None` while errors exist; `measure_starts` of the edited stream; `changed_measure_count`), `FormattedPartStream` (`edits: Vec<StreamTextEdit>`, empty when already formatted or the stream has errors; `cursors: Vec<StreamCursor>` mapped into the formatted text), `StreamTextEdit` (`span` + `replacement`, whitespace only), `StreamCursor` (byte `offset`), `FocusMeasure` (`section_label`, `lines_above` / `lines_below`: the measure's **Ghost lines**)
- Pure: `apply_part_stream` is meant to be called on every keystroke for diagnostics and measure markers; its `source` is only used on Apply. `ScoreDocument::load` refuses (every part ineligible) when the sections don't load, no part is declared, or desugaring reports any measure group error.
- Edits one `notes`/`chords`/`percussion`/`follow [X]`/lyric part as a **Part stream**. `document.rs` locates every measure group's `[Key]` lines in the original source (comment-stripped offsets) and each group's beat capacity (`interleaved_parser::measure_capacities`, tracking `time=` the way the parser does). `extract.rs` produces one **Measure chunk** per measure; `split.rs` parses the edited stream once, groups timed events with their `-` extensions into clusters and re-bars them by capacity (a cluster or tuplet crossing a barline is an error; a cut moves back over a preceding `(` / `N:{`; a measure mixing a tuplet with a `-` extension is an error — and at extraction, makes the part ineligible, as does implicit padding of a tuplet measure — because the parser counts `-` in a tuplet measure on the rescaled grid; a stream ending partway through a measure has its last measure completed with `rest_fill`); `apply.rs` rewrites only measures whose chunk changed — replacing the content after `[Key]`, inserting a line in declaration order, removing lines past the stream's end (rest-filling a group's only data line), and appending `[Key] chunk` groups — so every other byte (other parts, directives, comments) is untouched.
- `editable.rs` decides eligibility and each part's stream grammar (`StreamGrammar::Timed { TimedGrammar { kind, is_follow } }`, `StreamGrammar::Lyrics { slot_counts }` or `StreamGrammar::ChordLyrics { attacks }`; `StreamGrammar::join` turns chunks into the stream text). A `follow [X]` part uses its target's line grammar plus the **Follow placeholder** `@`: `split.rs` pre-scans `@` offsets and lays them out alongside the timed clusters, each one filling a whole measure (`@` partway through a measure, sharing a measure, or past the score's end is an error); `apply.rs` removes the part's line for an `@` (rest-filling a group's only data line), and writes explicit rests past the end of a shorter follow stream, since an unwritten follow measure would copy its target. `lyrics.rs` handles lyric parts whose target is a `notes` part (or a follow of one): `lyric_slot_counts` counts each measure's **Lyric slots** from the target's desugared line (`document.rs` keeps every part's desugared content per measure), padded like the parser pads it (`interleaved_parser::padded_measure_events`); the stream is tokenized with `utils::tokenize_lyrics` and cut by those counts (`_` takes a whole measure, a zero-slot measure gets an empty chunk, a short last measure is only a warning; leftover syllables are appended as new measures, one per line with `_` on its own, unless they are all `_` placeholders, which are dropped). Lyric parts with a syllable/slot tally mismatch in any measure other than a `_` one are ineligible. `chord_lyrics.rs` handles lyric parts whose target is a `chords` part: their unbracketed syllables flow freely, so there is nothing to cut by and the stream ends each measure with `|` (`join_chord_lyric_chunks`; a trailing `|` opens no measure, an unwritten measure is `_`, an empty measure is an error). `chord_attacks_per_measure` parses the target's desugared line into `MeasureChordAttacks` (attack count, first attack on beat 1), and `split_chord_lyric_stream` checks each measure with `parser::lyrics::bracketed::tokenize_bracketed_lyrics` plus `chord_lyric_tally_error`, so bracket mistakes are spanned in the stream; measures past the score's end are checked as having no chord attacks and appended like a longer timed stream's, and fewer measures is only a warning. A lyric part under a chords part with a bracket error in its source is ineligible (its own stream would not re-split cleanly).
- A part is eligible only if its own extracted stream splits back into exactly the same chunks with no errors, so applying an untouched stream never changes the source. The exception is a misbarred measure (more beats than its capacity, or fewer that can't be implicitly padded): `extract.rs` keeps it as written and reports the first one (`ExtractedChunks::first_misbarred_measure`), and the self-check only covers the measures (and the stream errors) before it — applying the untouched stream re-bars it and every later measure, and errors from there on surface in the stream editor instead.
- Wasm exports (`crates/poentaro-wasm/wit/world.wit`, helpers in `component/guest_metadata_and_misc.rs`, selection mapping in `component/guest_part_stream_selection.rs`): `list-streamable-parts(source) -> list<streamable-part>`, `extract-part-stream(source, abbreviation) -> extract-part-stream-response` (`ok(part-stream)` / `err(part-stream-failure { reason: part-stream-error, message })`), `apply-part-stream(source, abbreviation, edited-stream) -> part-stream-apply-result` (`StreamDiagnostic`s mapped onto the shared `diagnostic` record). Every offset (`measure-starts`, diagnostic spans) is a UTF-8 byte offset into the stream text. `format-part-stream(source, abbreviation, edited-stream, cursors: list<u32>) -> formatted-part-stream`, `part-focus-measures(source, abbreviation, toggles: part-toggle-state) -> part-focus-measures-response` (`ok(list<focus-measure>)` / `err(part-stream-failure)`), `part-stream-ranges-to-source(source, abbreviation, edited-stream, ranges: list<span>) -> list<span>` and `source-ranges-to-part-stream(...)` (byte ranges in the stream ↔ in the draft source, empty while the stream has errors). TS: `poentaroWasm().listStreamableParts` / `extractPartStream` / `applyPartStream` / `formatPartStream` / `partFocusMeasures` / `partStreamRangesToSource` / `sourceRangesToPartStream`.
- `format.rs` re-bars an edited stream by changing whitespace only: from `split`'s deduplicated `measure_starts`, the whitespace before each measure start becomes `\n` (inserted when there is none), any other whitespace run holding `\n` becomes a space, and leading whitespace is dropped; trailing whitespace is kept. A stream with errors gets no edits. Cursors map by non-whitespace count. Idempotent. `focus.rs` returns, per `ScoreDocument` measure, the group's other physical lines (trimmed, verbatim; `ScoreMeasure::lines` keeps every line with its part index, or none for directives and comments), split by declaration order around the focused part, keeping a part line only if `filters::resolve_part_visibility` renders it, plus the `label=`.
- Web (`web/src/components/partFocus/`, called on the main thread, no worker): the toolbar's "Edit part as stream" `PartFocusMenu` (a `ResponsiveMenu` of `listStreamableParts`, ineligible items disabled with Rust's reason) sets `focusedPart` in `AppWorkspace.tsx`, which overlays `PartFocusEditor` on `.editor-main` while the main `<Editor>` stays mounted (`inert`) beneath, keeping its model and undo history. `PartFocusEditor` is a Monaco editor on the stream: each change queues `formatPartStream` (skipped during IME composition, undo/redo, and its own edit) and applies the edits with `executeEdits` and no undo stop, so a keystroke and its re-barring undo together; `applyPartStream` supplies markers, the status and `measureStarts`; `partFocusViewZones.ts` renders each measure's header row (`createMeasureViewZoneDomNode`) and **Ghost lines** from `partFocusMeasures` as view zones, and a `before` decoration draws the `[Abbrev]` prefix. Apply (enabled only when `source` is present and something changed) writes `source` through `EditorHandle.replaceContent` (`editorImperativeHandle.ts`), an edit operation, so the main editor's undo reverts it; Cancel or Escape discards. Selection syncs with the preview as in the main editor: `selection.rs` aligns each measure's chunk with the focused part's line in the draft by counting non-whitespace characters, so a stream range maps to one draft range per measure (padding maps to nothing, a caret in a measure the part doesn't write lands on its group), and a draft range maps back onto its chunk, or — on another part's line — to a caret at that measure's chunk. `usePartFocusSelection.ts` reports `PartFocusEditor`'s mapped selections through the main editor's selection callbacks and fills `PartFocusSelectionHandle` (`types.ts`), which `usePartFocusSelectionRouting` (called by `useAppController`) lays over the main `EditorHandle` for the selection hooks while focused; both map through the last stream that applied without errors. The TS layer does no splitting, re-barring, filtering or validation.

### Pitch description (source-level)

- Module: `src/pitch_description/`
- Entry: `pitch_description::describe_selection(source: &str, ranges: &[source_edit::ByteRange]) -> Option<PitchDescription>`
- Key types: `PitchDescription` (`Note(NoteDescription { letter_name })` / `Chord(ChordDescription { chord_name, tone_names, bass_note, guitar_diagram_svgs })`), `GuitarVoicing` (see **Pitch description**, **Chord tone**, **Guitar voicing** in the glossary)
- Compiles the source and collects every note/chord/rest/percussion event whose span overlaps any of `ranges`, merging a tied run into one. Returns `Some` only when exactly one note or chord remains; `None` for a caret-only selection, several notes, a rest, a percussion hit, or a source that fails to compile. So the web drawer never decides for itself whether to open.
- Spelling (`spelling::SpelledPitch`) is by letter, from the measure's key tonic (carried forward from the last measure whose `key` is set, same as MIDI's active key): `7` in G → `F#`, `4` in F → `Bb`. Octave is ignored.
- `guitar_diagram_svgs` holds one chord box per chords-db voicing, in chords-db's order (most common first: usually the open shape, then shapes further up the neck). Each is rendered in Rust (`guitar_diagram_svg::render_guitar_diagram_svg`), using `currentColor` so the page theme colors it. The root `<svg>` has `data-guitar-frets` (absolute frets, e.g. `"x 3 1 3 4 x"`) and `data-guitar-base-fret` for tests. It is empty when chords-db has no voicing for the chord kind (`7sus2`, `maj7sus2`, `maj7sus4`, `dim(maj7)`). A slash chord with no slash-specific voicing uses its plain chord's shapes.
- `piano_diagram_svg` is rendered in Rust (`piano_diagram_svg::render_piano_diagram_svg`), always present. It spans only the chord tones plus two white keys of padding each side (edges always white keys). Highlighted keys are labelled with their interval from the chord root (`1 3 5` major, `1 b3 5` minor, `b7`, `#5`), never the key-relative degree, so a quality always shows the same pattern. Root position, except a slash bass that is a triad tone is lowest. The root `<svg>` has `data-piano-tones` (e.g. `"1:0 b3:3 5:7"`) for tests.
- Web: `useSelectionPitchDescription` (`web/src/hooks/`) sends the editor's committed selection to the worker's `describeSelection` request, holding the last answer while a click-and-click gesture waits on its second click. `PitchDrawer` (`web/src/components/`) only presents the generated `PitchDescription` (`web/src/poentaroWasm.ts`) that comes back (see **Pitch drawer** in the glossary).

### WASM boundary mechanism (`crates/poentaro-wasm`)

The Rust↔TS boundary is `wit-bindgen`/`cargo-component`/`jco` — not
`wasm-bindgen`/`tsify` (fully replaced; see
`PLAN-wit-bindgen-migration.md` for the migration's rationale and
history). One `.wit` schema (`crates/poentaro-wasm/wit/world.wit`, flat
world-level `record`/`variant`/`enum`/`export` declarations, kebab-case
names) is the single source of truth for every exported function's
*full* signature — inputs and outputs alike — generated into real Rust
types via `wit_bindgen::generate!` and real TS types via `jco
transpile`, so a shape mismatch on either side is a compile error, not
a runtime `JsValue`/`any` surprise.

- `crates/poentaro-wasm/src/component.rs` — the crate's one and only
  `Guest` impl (`struct Component`), implementing every exported
  function against the same internal `crate::responses::*`/
  `crate::part_declarations::*`/`crate::symbols::*`/
  `crate::selection_range::*` functions the rest of the crate (and its
  own unit tests) call directly. Converts WIT-generated
  records/variants to/from this crate's own internal `*Out`/`*In` types
  at each function's boundary (e.g. `part_to_wit`, `render_response`'s
  `documents`/`diagnostics` conversion) — the only place in the crate
  that names both type families.
- Build: `cargo component build --features wav,mp3,pdf,midi --target
  wasm32-unknown-unknown` (workspace's `release-wasm` Cargo profile for
  the real asset build — `debug = false, strip = true`, keeping the
  native `poentaro` CLI's own `[profile.release]` untouched) produces a
  component `.wasm`; `jco transpile <that>.wasm --instantiation async
  -o crates/poentaro-wasm/pkg-component` generates the JS glue
  (`poentaro_wasm.js`/`poentaro_wasm.d.ts`/`poentaro_wasm.core.wasm`,
  gitignored, like the crate's own `target/`). `web/package.json`'s
  `build:wasm-component` script runs both steps in sequence and is
  wired into `predev`/`prebuild`/`build` — the only wasm build path;
  there is no wasm-bindgen/wasm-pack fallback left anywhere in the repo.
- `web/src/wasmInit.ts`'s `ensureWasmModule`/`ensureWasmInit` fetch and
  compile the component's core `.wasm` once (shared in-flight promise
  across every caller, including a `postMessage` hand-off of the
  compiled `WebAssembly.Module` to the render worker) and instantiate
  it via the jco-generated `instantiate((_) => module, {})` (no host
  imports needed on `wasm32-unknown-unknown`). `web/src/worker/poentaro.worker.ts`
  does the same on its own thread once it receives the module.
- `web/src/poentaroWasm.ts` is the single web import surface for the
  component, with no reshaping layer: it re-exports every jco-generated
  type unchanged (camelCase fields, `{ tag, val }` variant envelopes,
  kebab-case enum/case names, and `SvgDocument`'s flat pre-order element
  arena — `rootElementIndices`/`SvgGroupKind.childIndices` index into
  `elements`) and hands out the generated `Root` via `poentaroWasm()`
  (`setWasmRoot`, called once by `wasmInit.ts`/`poentaro.worker.ts` after
  instantiation). A field or case renamed in `world.wit` is therefore a
  type error at every consumer. WIT record names must not equal
  `<variant>-<case>` of any variant: jco names each case's interface that
  way too, and TypeScript would merge the two declarations into one
  unconstructible type (`poentaroWasmDts.test.ts` guards this). The
  exports table below names each function by its WIT (snake_case) name;
  `Root`'s method for it is the camelCase form.
- The React preview (`PreviewSvgRenderer.tsx`) writes `data-tag`/`data-variant`
  as the generated `Tag['tag']`/`SvgVariant`/`TransparentRectRole` strings
  verbatim (`DataVariant` in `web/src/dataAttributes.ts`), so its
  click-target `data-variant`s drop the Rust serializer's `-rect` suffix.
  CSS selectors on those attributes are built in
  `web/src/injectPreviewInteractionStyles.ts` from the typed selector
  helpers, never hand-typed in a `.css` file (`dataAttributes.test.ts`
  guards this).
- Each `Tag` field's `data-*` attribute name lives only in
  `TAG_FIELD_ATTRIBUTES` (`web/src/dataAttributes.ts`, checked against the
  generated `Tag` field set). The writer (`groupAttributesForTag`), the
  reader (`tagFromElement`) and the selector builders (`tagSelector`,
  `variantSelector`, `rectVariantSelector`, `tagFieldAttribute`) all derive
  from it. The Playwright step files use the same builders, so they never
  hand-type a `[data-tag=…]` selector either.

### WASM exports (`crates/poentaro-wasm`)

| Export | Purpose |
|--------|---------|
| `render_mei(source, visibility, raw_instruments, clef)` | The score as MEI for the staff-notation view, every staff in the `staff-clef` (`violin` → treble, `cello` → bass) (`render_mei_from_source_with_visibility`, see **MEI export**); `ok { mei, measures: list<staff-measure> }` / `err(diagnostics-error)`. Takes the same `resolved-part-visibility` as `render_svg` |
| `render_split_meis(source, base_name, preset_name, raw_instruments, clef)` | One MEI per sounding part (with its lyric parts) for the staff PDF (ZIP) export, in the given `staff-clef`, each with the PDF file name it becomes |
| `generate_staff_pdf(pages, source, fonts)` | Verovio's staff SVG pages → PDF bytes, source embedded; returns `generate-pdf-response` |
| `generate_split_staff_pdfs(parts, fonts)` | Each part's Verovio pages → one PDF each, zipped; returns `generate-split-pdfs-response` |
| `list_parts(source, raw_instruments)` | Part summaries for preview toggles **and** `declarations: list<part-declaration>` (each carrying `settings: part-settings`) for the Edit Parts modal |
| `list_part_declarations(source, raw_instruments)` | Declarations only (re-list after a write) |
| `update_part_declaration(source, abbreviation, settings, raw_instruments)` | Returns updated source (unchanged if the part isn't declared). `settings` is the WIT `part-settings` record (`mode: part-mode` variant whose `follow` case carries its target, `program: option<u8>`, `volume: u8`, `octave-offset: s8`), the same record `part-declaration.settings` reports, so the Edit Parts modal edits one field and sends the rest back as-is; Rust writes `program` as its **Sound label** |
| `list_sound_choices(raw_instruments)` | `sound-choices { instruments, percussion: list<sound-choice { program, label }> }` — every sound the Edit Parts sound picker offers, labelled by `sound_label::list_sound_choices`, so TS never builds a **Sound label** |
| `get_part_setting_limits()` | `part-setting-limits { volume: volume-range, octave-offset: octave-offset-range }` (each `{ min, max }`, inclusive) — `PART_SETTING_LIMITS`, from which the Edit Parts modal builds its volume slider bounds and octave select options (`web/src/hooks/usePartSettingOptions.ts`), so TS never hard-codes a **Part setting limit** |
| `list_range_instruments()` | `list<range-instrument-choice { name, label }>` — every `RangeInstrument`'s keyword and its picker label (e.g. `guzheng 古箏`), in declaration order; the Edit Parts modal's Instrument column, fetched alongside the limits by `usePartSettingOptions` |
| `shift_part_octave(source, abbreviation, delta)` | Rewrites `'`/`,` octave markers on every note in the named part by `delta` octaves (see **Source editing**); returns the updated source directly, no response envelope (same infallible shape as `update_part_declaration`) |
| `shift_range_octave(source, ranges, delta)` | Rewrites `'`/`,` octave markers on every note (across every part) whose span overlaps any of the disjoint `ranges` by `delta` octaves — the editor toolbar's "Octave up"/"Octave down" buttons, scoped to the current (possibly multicursor) selection (see **Source editing**); returns `RangeEditResponse { source, ranges }`, where `ranges` is the caller's own input `ranges` remapped onto their new byte spans in `source` (preserving count/shape, not one range per shifted note), so the editor can restore its selection synchronously alongside the new source |
| `toggle_range_slur(source, ranges)` | The editor toolbar's "Slur/Unslur" button: removes every `(…)` group the selection overlaps, or — if none — wraps each part's selected notes in a new group (see **Source editing**); returns the same `RangeEditResponse { source, ranges }` as `shift_range_octave` |
| `toggle_range_tie(source, ranges)` | The editor toolbar's "Tie/Untie" button: removes the `~` of every selected tied note, or — if none — ties each part's selected same-pitch neighbours (see **Source editing**); returns the same `RangeEditResponse { source, ranges }` as `shift_range_octave` |
| `compress_share_payload(payload) -> Vec<u8>` | Brotli-compresses a share-link JSON payload (quality 11); caller base64url-encodes the result |
| `decompress_share_payload(bytes) -> Option<String>` | Inverse of the above; `None` if `bytes` isn't valid brotli or decodes to invalid UTF-8 |
| `format_score(source)` | "Format" toolbar action: drops redundant implicit-fill-equivalent `[Key]` lines, sorts surviving lines into `parts` declaration order, and normalizes whitespace in `score` (see **Score formatting (source-level)**); returns the updated source directly, no response envelope (same infallible shape as `update_part_declaration`) |
| `describe_selection(source, ranges) -> Option<PitchDescription>` | The pitch drawer: letter names (plus chord name, bass note and guitar-diagram SVG for a chord) of the one note/chord `ranges` covers, or `none` for any other selection (see **Pitch description (source-level)**) |
| `extract_source_from_svg(svg_bytes) -> Option<String>` | Recovers the `.poentaro` source embedded in a previously exported SVG (`source_embed::extract_embedded_source`); `None` if absent/invalid |
| `extract_source_from_pdf(pdf_bytes) -> Option<String>` | Recovers the `.poentaro` source embedded in a previously exported PDF (`source_embed::extract_embedded_source_from_pdf`); `None` if absent/invalid |
| `generate_midi(source, enabled_tracks)` | Generates MIDI (SMF) bytes for the whole score. `midi` feature only. |
| `generate_split_midis(source, base_name, preset_name)` | One MIDI file per part, zipped. `midi` feature only. |
| `generate_split_wavs(source, base_name, preset_name, soundfont)` | One WAV file per part, zipped; `soundfont` is raw SF2 bytes supplied by the caller. `wav` feature only. |
| `generate_mp3(source, enabled_tracks, soundfont)` | Generates 128kbps CBR MP3 bytes for the whole score (`rusty_mp3`, pure Rust — cross-compiles to `wasm32-unknown-unknown`, unlike LAME-backed encoders); same shape as `generate_wav`. `mp3` feature only (implies `wav`). |
| `generate_mp3_for_measure_range(source, start_index, end_index, extend_to_last_occurrence, respect_sequence, sequence_entry_start_index, sequence_entry_end_index, enabled_tracks, trim_start_s, trim_end_s, trim_next_note_start_s, soundfont)` | MP3 counterpart of `generate_wav_for_measure_range` — same parameters and semantics, MP3-encoded output. `mp3` feature only. |
| `generate_split_mp3s(source, base_name, preset_name, soundfont)` | One MP3 file per part, zipped; MP3 counterpart of `generate_split_wavs`. `mp3` feature only. |
| `list_note_timings(source, visible_tracks, enabled_tracks)` | Elapsed-seconds start/end of every sounding note/rest, keyed by `(source_part_index, note_id)` — see **Note identity**. Drives the SVG preview's per-part, per-note playback cursor. `wav` feature only. `visible_tracks` must be the part-visibility toggle's own state (the same set physically removed before the rendered SVG's own render call), so block/note-id structure (including a `MultiMeasureRest` run only created once a hidden sibling part's notes are removed) always agrees with the rendered SVG; `enabled_tracks` separately mutes a further, possibly narrower subset of those visible parts for this one call only (e.g. "play selection"), without affecting `source_part_index` or block structure. |
| `group_note_selection(note_spans, selected_cells)` | Groups a selected set of `(sourcePartIndex, noteId)` cells (already-fetched from `list_note_spans`) into contiguous per-`(part, measure)` byte runs (see **Note click target**), ready to become a Monaco multicursor selection. Pure grouping — does not re-parse `source` — so it's called directly on the main thread rather than routed through the render worker, keeping it responsive on every selection-change tick. |
| `list_lyric_spans(source)` | Per-syllable `(sourcePartIndex, noteId, measureIndex, start, end, timing)` byte spans for every rendered lyric syllable, mirroring `list_note_spans` (see **Lyric click target**). Drives the SVG preview's lyric-syllable selection independently of note selection. |
| `group_lyric_selection(lyric_spans, selected_cells)` | Groups a selected set of `(sourcePartIndex, noteId)` cells (already-fetched from `list_lyric_spans`) into contiguous per-`(part, measure)` byte runs (see **Lyric click target**), ready to become a Monaco multicursor selection. Pure grouping, called directly on the main thread like `group_note_selection`. |
| `select_notes_in_byte_ranges(note_spans, ranges)` / `select_lyrics_in_byte_ranges(lyric_spans, ranges)` | The notes (or lyric syllables) whose source token overlaps any of `ranges`, plus their runs: `span_selection::select_notes_in_byte_ranges(ranges: &[ByteRange], note_spans: &[NoteSourceSpan]) -> NoteSelection` (`NoteSelection { cells: Vec<NoteCell>, runs: Vec<NoteSelectionRun> }`) and its lyric twin returning `LyricSelection { cells: Vec<LyricCell>, runs: Vec<LyricSelectionRun> }` (`src/span_selection.rs`). An empty range (a bare caret) selects nothing. Used for a Monaco selection change, and to re-derive a selection after a part is hidden or shown: the selected source bytes stay put while the spans' `source_part_index`es are renumbered. Pure grouping, called directly on the main thread like `group_note_selection`. |
| `resolve_selection_range(note_spans, lyric_spans, anchor, current)` | Resolves the click-and-click range between two `ClickableElementId`s (see **Clickable element ID**) into the note/lyric cells it covers — `ok { note_cells, lyric_cells }`, or `err` when an endpoint isn't in the spans. Pure grouping over already-fetched `note_spans`/`lyric_spans` (`crates/poentaro-wasm/src/selection_range/mod.rs`), called directly on the main thread like `group_note_selection`. Every anchored mode in `previewSelectionResolveModes.ts`/`previewSelectionResolveLabelModes.ts` resolves through it; there is no TS fallback. |
| `resolve_system_selection_range(note_spans, lyric_spans, systems, anchor, current)` | Every part of every system from the anchor's through the current endpoint's (`selection_range::resolve_system_selection_range_response`); `systems` are the rendered systems' inclusive measure ranges. Backs a bar number's click-and-click and a Cmd/Ctrl-click on a part label. |
| `fully_selected_labels(note_spans, lyric_spans, labels, note_cells, lyric_cells) -> list<bool>` | Whether each part/lyric label reads as selected: once every note (part label) or syllable (lyric label) it covers across its system is in the given cells (`selection_range::fully_selected_labels`). Drives the label fill in `previewLabelRangeHighlights.ts`. |
| `staff_measure_at_point(measures, x, y) -> option<u32>` | The staff-view measure a click/hover lands on, from each Verovio measure's `staff-measure-geometry` (`staff_measures::staff_measure_at_point`; see **MEI export**). |
| `staff_measure_area(measure) -> bounding-box` | The area a staff measure's hover target and highlight rect cover: its staff lines' extent across, its group box down. |
| `highlighted_staff_measures(measure_indices, ranges) -> list<u32>` | The staff measures inside any of the editor selection's measure ranges. |
| `pending_staff_measures(measure_indices, anchor, current) -> list<u32>` | The staff measures a click-and-click gesture spans between its anchor and current end, in either order (the hover preview). |
| `note_selection_trim_window(cells, note_timings) -> option<trim-window>` | The `{ start_s, end_s, next_note_start_s }` stretch of a measure-range clip that exactly covers the selected notes, given that clip's `list_note_timings_for_range` output (`note_selection_trim`); `none` plays the clip in full. Feeds `generate_*_for_measure_range`'s `trim_*` arguments when playing a note selection. |
| `list_note_timings_for_range(source, start_index, end_index, extend_to_last_occurrence, respect_sequence, sequence_entry_start_index, sequence_entry_end_index, visible_tracks, enabled_tracks)` | Same as above, scoped to a measure range and relative to the start of that range. Syncs the playback cursor against the audio from `generate_wav_for_measure_range`. `wav` feature only. `respect_sequence: false` (used by "play current measure") ignores `sequence`/D.C./D.S. navigation and any part omissions it applies, playing exactly what is written; `true` (used by "play from current measure") follows them. See `list_note_timings` for how `visible_tracks` differs from `enabled_tracks`. |
| `parse_metadata_fields(source, preset_name)` | Returns the WIT `metadata-fields` record from `source_edit::parse_metadata_fields`: optional title/subtitle/author text, numeric and flag fields, the verbatim `directive-row-offset`, `styles: list<text-style-entry>` (one `{ kind: text-style-kind, fields: text-style-fields, defaults: text-style-defaults }` per kind, in `TextStyleKind::ALL` order), and the scalar `defaults: metadata-defaults`. Under a preset (`preset_name` set) the fields are that preset's overrides, and `defaults` (which then also carry title/subtitle/author) are the song's `metadata` values they fall back to. Both `defaults` come from `grouper::metadata_defaults` on the parsed metadata, so each placeholder is what that field would resolve to if left unset in *this* document (e.g. `title`'s font size tracks an explicit `row_height`, `notes`' tracks an explicit `lyrics` font size); the web app never recomputes a default itself. The web app calls it on the main thread via `useMetadataFields` (only while the Edit Metadata modal is open). |
| `update_metadata_field(source, preset_name, edit)` | Applies one `metadata-edit` variant (`text`/`number`/`flag`/`directive-row-offset`/`style`/`reset-style`, keyed by the WIT `metadata-text-key`/`metadata-number-key`/`metadata-flag-key`/`text-style-kind` enums and the `text-style-component-value` variant) to the preset's section, or `metadata` when `preset_name` is unset, via `source_edit::update_metadata_field`; returns the updated source directly. |
| `list_presets(source)` / `update_preset_field(source, preset_name, edit)` / `set_preset_part_shown(source, preset_name, all_parts, part)` / `set_preset_part_soloed(source, preset_name, all_parts, part)` / `new_preset_name_error(source, name)` / `add_preset_section(source, preset)` / `follow_preset_rename(previous_source, source, preset_name)` | The preset menu's calls into `source_edit::preset_edit`, with the WIT `notation` enum, `preset-summary`, `preset-edit` (`visible-parts`/`soloed-parts`/`notation`), `part-shown`, `part-soloed` and `new-preset` types (`crates/poentaro-wasm/src/component/guest_presets.rs`). The edits return the updated source directly; `follow-preset-rename` returns the chosen preset's name after an edit, or `none`. |
| `highlight_tokens(source)` | Returns `list<highlight-token>` (`{ kind: highlight-kind, span }`, sorted, each within one line) from `highlight::highlight_tokens`. Called on the main thread by the editor's semantic-tokens provider on every document change. |
| `section_headers(source)` | Returns `list<section-header>` (`{ kind: section-kind, span }`, sorted) from `highlight::section_headers`. Called on the main thread by the editor's section CodeLens provider. |
| `set_layout_fonts(fonts: font-bytes-by-family)` | Supplies each `font-family` role's font bytes (`serif`/`sans-serif`/`monospace`, the same roles as the keys of `fonts/fonts.json`) used for real glyph-advance measurement during layout (see `src/font_metrics/`). Keyed by role only: which role measures which text (directive line → `SansSerif`, lyrics → `Serif`, noteheads → `Monospace` or their `font_family` override) is decided in `src/font_metrics/`, never by the caller. The same `font-bytes-by-family` record is taken by `generate-pdf`/`generate-split-pdfs`. Unconditional (not feature-gated) and needed by every build — the wasm binary doesn't embed these three fonts at compile time (unlike the host CLI build), since doing so would add tens of MB to it; the caller supplies the same bytes it already fetches for PDF export. Forwards to the core crate's `set_layout_font_bytes(fonts::FontBytesByFamily)` (`src/lib.rs`), which is a no-op on non-`wasm32` builds. |

`generate_pdf`/`generate_split_pdfs` (`pdf` feature) follow the same pattern as the MIDI/WAV exports above: structured `{ status, ... }` envelope, `Vec<u8>` font/soundfont parameters supplied by the caller rather than embedded in the WASM binary.

`web/src/shareUrl.ts` calls `compress_share_payload`/`decompress_share_payload` directly from the main thread (a separate WASM component instance from the render worker's), lazily calling `ensureWasmInit()` on first use, to build/parse `#share=<base64url>` links. `decodeShareHashSuffix` falls back to the legacy `lz-string`-encoded format, then plain-JSON, for links created before this switch. `web/src/hooks/useNoteSelection.ts` calls `group_note_selection` the same way, on every note range-select and editor-selection-change tick; it needs no debounce, since it's pure grouping over the already-fetched `note_spans` array rather than a re-parse of `source`. `web/src/hooks/useLyricSelection.ts` is the fully independent lyric-syllable counterpart, calling `group_lyric_selection` over a separately-fetched `lyric_spans` array — it shares no state with `useNoteSelection`, only the same unified click-and-click gesture plumbing in `usePreviewClickSelection.ts` (see **Lyric click target**). Both wrap the same low-level `web/src/hooks/useByteRangeSelectionCore.ts`'s `useByteRangeSelectionCore<Cell, Span, Run>` for their shared plumbing — `selectedCells`/`runs` state, the `suppressNextEditorSelectionSyncRef` echo-guard, and the range-select ⇄ Monaco-selection round-trip (`handleRangeSelect`/`handleEditorSelectionChange`) — each supplying its own grouping and byte-range selection functions, cell/span shape, and (for `useNoteSelection` only) an `onNoMountedEditor` fallback for Live/shared views; `useNoteSelection` layers `selectedNoteRangePlaybackInfo` on top of the core's `runs` via its own `useMemo`. The core also keeps the selection's source byte ranges, and when the set of visible parts in `spans` changes it re-derives `selectedCells`/`runs` from them via `select_notes_in_byte_ranges`/`select_lyrics_in_byte_ranges`, so the highlight and "play selection" follow parts hidden or shown after selecting. The core is deliberately not itself a merge point between the two hooks — each still calls it independently, so a lyric selection can never leak into note-highlight state or vice versa.

The worker (`web/src/worker/poentaro.worker.ts`) gates its `render`/`renderWithHighlightRange` handlers on `ensureCoreFontsApplied()`, which awaits both wasm init and the role-keyed font bytes (`FontBytesByFamily`) (resolved from the `loadPdfFonts` message the app already sends at boot — see `web/src/hooks/useFontsLoader.ts`) before calling `set_layout_fonts`; other handlers that don't reach the layout/render chain (`listParts`, `updatePartDeclaration`, `listMeasureSpans`, export handlers) aren't gated on it.

Worker messages: `listParts` → `{ parts, declarations }`; `updatePartDeclaration` → `{ source, declarations }` (hook updates `partDeclarations` immediately, without waiting for the debounced re-render); `importFromFile { bytes, kind: 'svg' | 'pdf' }` → `importOk { source }` / `importErr` (`web/src/worker/importMessageHandlers.ts` calls `extract_source_from_svg`/`extract_source_from_pdf`; `usePoentaroWorkerImport.ts`'s `importFromFile(file: File) -> Promise<string>` is the promise-based wrapper `App.tsx`'s Import button uses, resolved/rejected from `usePoentaroWorkerMessageHandler.ts`'s `pendingImportsRef` map keyed by request id, mirroring `updatePartDeclaration`'s pending-map pattern).

## Synced Share worker (`crates/live-share-worker`)

A separate Cloudflare Worker, not part of the `web/` build or the
`poentaro-wasm` component boundary above — it's a small standalone HTTP
backend the web app's Synced Share feature calls cross-origin. The old
TypeScript, KV-backed worker (formerly the top-level `live-share-worker/`
directory, `src/{resolveRole,doc,index,protocol}.ts`, deployed as
`jianpu-live`) and its anonymous device-secret ownership scheme
(`getOrCreateDeviceSecret`/`deriveSyncedShareIdentity`) have been retired
(`TODO-synced-share-rust-d1-migration.md` task 11) — that source is deleted,
not kept as reference. The real D1 database (`jianpu-live-share`, region
APAC) exists and is migrated as of task 12 (`crates/live-share-worker/cloudflare.config.ts`'s
`database_id`), and a deploy pipeline now exists to ship it
(`.github/workflows/live-share-worker.yml` — builds with `worker-build`,
applies D1 migrations, then `cf deploy`, on every push to `master`
touching `crates/live-share-worker/**`/`live-share-worker/migrations/**`).
It needs no secrets: accounts are email + password, kept in D1. The
The
top-level `live-share-worker/` directory now holds only the shared D1 schema
migrations (see "Storage" below), nothing else.

- Crate: `crates/live-share-worker` (target `wasm32-unknown-unknown`,
  built as a `worker`-crate Cloudflare Worker). Entry point: `#[event(fetch)]
  async fn fetch` in `src/lib.rs` — applies CORS (`Access-Control-Allow-*`,
  via the `worker` crate's `Cors` builder) and the `OPTIONS` preflight
  response uniformly around the `worker::Router` that
  `handlers::routes()` builds (routing `GET /shares/{share_id}`, the
  owner-only `POST /files/{id}/share` (start), `POST /files/{id}/share/stop`
  and `POST /files/{id}/share/status`, the folder-share routes
  `POST /folders/share`, `/folders/share/stop`, `/folders/share/status`
  (body `{identityToken, folder}`) and `POST /shares/overview`, the rest of the `/files/*` cloud
  storage routes, and the account routes `POST /auth/sign-up`,
  `/auth/sign-in`, `/auth/sign-out` and `/auth/change-password`, see
  "Accounts (email + password)" below, and the anonymous origin-migration
  handoff `POST /origin-migrations` / `POST /origin-migrations/claim`, see
  "Origin migration" below). Plus the test-only `POST /e2e/reset`
  (`handlers::e2e_reset`, config in `src/e2e_reset.rs`): registered via
  `Routes::post_if` only when the `SYNCED_SHARE_E2E_RESET_EMAILS`
  var is set — never in `cloudflare.config.ts`, only by `web/playwright.config.ts`'s
  local `cf dev` — so otherwise it answers like any unknown path. The
  var's JSON array of emails is also its allowlist; it hard-deletes
  those accounts' password credentials, sessions, sign-in lockouts, `files`
  and `shares` rows (`queries/delete_*_of_users.sql`,
  `delete_sign_in_lockouts_of_emails.sql`), so each run can sign them up
  afresh, and is called by `web/e2e/global-setup.ts` before every local e2e run.
  Second entry point: `#[event(scheduled)] async fn scheduled` in
  `src/lib.rs`, the daily cron that prunes `app_log` — see "App log
  (observability)" below — and then expired `origin_migrations` rows.
- File names are slash paths: `files.name` holds the whole path (`Hymns/Advent/O Come.poentaro`) and `idx_files_owner_name` (unique on `(owner_user_id, name)`) is therefore a uniqueness constraint on the full path; the schema has no length cap or character filter. The worker depends on the root `poentaro` crate (`default-features = false`, same as `crates/poentaro-wasm`), and `files::validate_file_name` runs `file_tree::TreePath::parse` on the name of every `POST /files`, `/files/{id}/rename` and `/files/{id}/restore` before touching D1, so the web app and the worker share one rule set (no empty segments, no `.`/`..`, at most 1024 UTF-8 bytes for the whole path). A violation answers `ApiError::InvalidName` (400). `POST /files/rename-batch` (`{identityToken, moves:[{id,name}]}`, `RenameBatchRequest`) validates every name, then `files::plan_batch_rename` over the current rows decides `Apply`, `AlreadyApplied` (an ack-lost retry succeeds), `NotFound`, `Invalid` or `NamesTaken`; the move itself is one `UPDATE ... json_each(?1)` statement (`queries/rename_files_batch.sql`), atomic by statement semantics and free of D1's per-invocation query limit. Create, rename, restore and batch rename also enforce that a path is not both a file and a folder: the handler loads the owner's live file names (`queries/list_live_file_names.sql`) and `files::find_file_folder_conflict` judges the final state (a batch is evaluated as a whole; `<folder>/.keep` is an ordinary file, so it only conflicts with a file named `<folder>`). A violation answers `ApiError::FileFolderConflict { path }` (409), which the outbox treats as a permanent failure. The check is read-then-write, not atomic, so two concurrent writes can still race past it. `share::to_public_doc` returns only the last path segment as `filename`, so a viewer of one shared file never sees the owner's folder names.
- Folder shares (`src/folder_share.rs`, pure; handlers in `handlers/shares.rs`; migration `0005_folder_shares.sql`): a `shares` row is either a file share (`file_id`) or a folder share (`folder_prefix`, the folder path without trailing slash), exactly one (CHECK), and now carries its own `owner_user_id` (file shares used to derive it from the file). A folder has at most one row per owner (unique `(owner_user_id, folder_prefix)`), so stop-then-start reuses the `share_id`. The worker still knows nothing about folders as entities: a folder share covers every active, non-binned file whose name starts with `<prefix>/` at read time (matched with `substr`, whole segments, so `Hymns` never covers `Hymns 2/...`; `%`/`_` are literal), so files added, edited, moved in or out and deleted show on the viewer's next load (no push, no polling). Shared folders are read-only for viewers. Routes: `POST /folders/share` (`FolderShareRequest {identityToken, folder}` -> `{shareId}`; `folder` validated with `TreePath` rules plus "no `.keep` segment" -> `invalid_name`; `not_found` when the folder has no active file, a `.keep` placeholder counts; `shared_folder_too_large` (413) over the limits), `/folders/share/stop` (204, or `not_found` if never shared; works on a folder that has since emptied), `/folders/share/status` (`{share: {shareId, ended} | null}`, same shape as the file status). `POST /shares/overview` (`{identityToken}` -> `{files: [{shareId, fileId, name}], folders: [{shareId, folder}]}`, live links only, binned files excluded) is the one read the share UI derives everything from: own-link markers, "shared via <ancestor>" (a `folders` entry that is a path-segment prefix of the file/folder) and the "this folder contains files with their own link" warning. Viewer: `GET /shares/{share_id}` falls through from the file-share view to `get_folder_share_header`; for a folder share `SyncedDoc.folder` (`SharedFolder {name, files: [{path (relative to the shared folder), content, revision}], emptyFolders}`) holds the tree, `filename` repeats the folder name, `content` is `""` and `revision` 0; `.keep` rows are never files, and `emptyFolders` are placeholder folders with no file under them. `folder` is absent (skipped, not `null`) for file shares, ended shares and unknown ids, so those responses are unchanged. A stopped share answers `ended: true` with nothing else; a live folder with no files is an empty live tree (trashing every file does not end the share). Size limit: `folder_share::MAX_SHARED_FOLDER_FILES` (300 files, placeholders included) and `MAX_SHARED_FOLDER_BYTES` (10 MiB of content), checked by `get_folder_files_stats.sql` both when starting and when a viewer loads; over it answers `ApiError::SharedFolderTooLarge`, never a truncated tree. Renames: `POST /files/rename-batch` reads the owner's folder-share prefixes (live or stopped), and `folder_share::plan_prefix_rewrites` treats a prefix as moved when the batch moves every active file under it to one common new folder keeping the rest of each path (a folder rename/move); nested shared folders get their own rewrite, partial moves leave the share alone (it follows the prefix, not file ids). The file renames and `rewrite_folder_share_prefix.sql` updates go out in one D1 `batch` (one transaction), so the link keeps its `share_id`; a rewrite onto a folder that already has its own share hits the unique index and answers `name_taken`.
- Route list and generated client: `handlers::routes()` is the one list of
  routes. Each `.get(path, handler)`/`.post(path, handler)` call on
  `handlers::routes::Routes` registers the route into both the
  `worker::Router` and an OpenAPI spec, deriving every part of the spec
  from the handler's signature — path params from an `IntoParams` struct
  (`protocol::FileIdPath`/`ShareIdPath`, checked against the path
  template's `{name}` segments when the list is built), the request body
  from a `ToSchema` type, the success response from its `Reply` type
  (`Json<T>` → `200`, `NoContent` → `204`), and `api_error::ApiError` as
  every route's `default` response. `openapi_json()` (`src/lib.rs`) serializes
  that spec; `tests/export_openapi.rs` writes it to
  `web/src/generated/live-share-worker/openapi.json`, and `web`'s
  `build:worker-types` script turns it into `schema.ts` with
  `openapi-typescript` (pinned to TypeScript 5 by `web/.pnpmfile.cjs`, since
  it needs the TS 5 compiler API). The same test also writes
  `shareIdFormat.json` there (`share_id::share_id_format()`: a `share_id`'s
  length and anchored regex), a runtime value rather than a type, which
  `web/src/syncedShareUrl.ts` and the Pages Function `web/functions/index.ts`
  import to parse share links — so neither restates the id shape — and
  `e2eWorkerConfig.json` (`e2e_worker_config::e2e_worker_config()`: the `--var`
  name `web/playwright.config.ts` sets to enable `POST /e2e/reset`) and
  `deployConfig.json` (the public host out of `deploy.json`, which
  `web/vite.config.ts` and `web/functions/index.ts` read). All five
  files are gitignored and regenerated by `build:worker-types` (run by
  `predev`/`prebuild` and the `web-typecheck` pre-commit job), so they
  can't go stale against the Rust source. `web/src/syncedShare/workerClient.ts`'s
  `createWorkerClient(host)` is an `openapi-fetch` client over those types,
  and `callWorker` turns each call's result into its success body or a
  `NetworkFailure`/`UnreadableResponse`/`WorkerRequestError` (the latter
  carrying the parsed `ApiError`) — every web call site and e2e helper
  (`web/e2e/cloudFileHelpers.ts`) goes through it, and the Pages Function
  `web/functions/index.ts` builds its own `openapi-fetch` client over the same
  `paths` type (keeping SPA code out of its edge bundle), so no path, method,
  body shape or error literal is restated by hand in TS.
- Storage: D1 (SQLite), not the old KV namespace. Schema in
  `live-share-worker/migrations/0001_init.sql` (shared, as plain `.sql`,
  between real D1 migrations and a local shadow SQLite database
  `crates/live-share-worker/build.rs` builds at build time for query
  checking — see below). Tables: `shares` (one row per shared file:
  `share_id`, `file_id` — unique, `REFERENCES files(id)` — `ended_at`,
  `created_at`; see **share** in the glossary; since `0005_folder_shares.sql` also `owner_user_id` and a nullable `folder_prefix`, see "Folder shares"), `users` (`id`, `created_at`),
  `user_identities` (`provider`, `provider_user_id`, `user_id`, `login`,
  `linked_at` — the GitHub sign-in links, read by nothing since
  `0007_password_auth.sql`, kept as a record of which GitHub account a user
  was), `password_credentials` (`user_id`, unique `email`,
  a self-describing `password_hash` `pbkdf2-sha256$<iterations>$<salt>$<hash>`,
  timestamps), `sessions`
  (`token_hash`, `user_id`, `created_at`, `expires_at`) and
  `sign_in_lockouts` (`email`, `failed_count`, `locked_until`,
  `lockout_level`) — all three since `0007_password_auth.sql`, which also
  dropped GitHub's `oauth_sessions` verification cache; see "Accounts
  (email + password)" below — `app_log` (since `0006_app_log.sql`; see "App log
  (observability)" below), `file_versions` (since `0008_file_versions.sql`;
  see "Version history" below) and `origin_migrations` (since
  `0010_origin_migrations.sql`; see "Origin migration" below). `live-share-worker/migrations/0004_shares_reference_files.sql`
  replaced the old `docs` table (a share's own copy of
  `filename`/`content`/`revision`, pushed by the owner's browser separately
  from autosave — a stale-render race in that push could write one file's
  content to another file's link) with `shares`, carrying over only the
  `docs` rows whose `external_file_id` was a `files.id` owned by the same
  user.
- Key types: `share::ShareView` (what `queries/get_share_view.sql` returns
  for a share id — the share's `ended_at` plus the pointed-at file's
  `trashed_at`/`filename`/`content`/`revision`; nothing about the owner,
  never the file id or `owner_user_id`) /
  `share::to_public_doc` (builds the public response from it, D1-free and
  unit-testable; an ended share — stopped, or its file in the bin — keeps
  only `ended`, with empty `filename`/`content`) /
  `share::empty_doc` (the response for an unknown share id);
  `api_error::ApiError` (every route's failure body: a `code`-tagged union
  — `bad_request`, `invalid_name { message }` (400: a create/rename/restore name that fails `files::validate_file_name`), `not_found`, `shared_folder_too_large { message }` (413: a folder share over `folder_share`'s limits), `origin_migration_too_large { message }` (413: an origin-migration upload over `MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES`), `name_taken`, `names_taken { names }` (409: a `POST /files/rename-batch` collision, nothing applied), `revision_conflict {
  currentRevision }`, `unauthorized` (a flattened `VerificationFailure`),
  the account failures `email_taken` (409), `invalid_email` and
  `weak_password { message }` (400), `incorrect_credentials` (401: unknown
  email and wrong password alike), `sign_in_locked { retryAfterSeconds }`
  and `rate_limited` (429), and `internal { message }` — whose HTTP
  status comes from `ApiError::status_code`, never chosen at a call site) /
  `api_error::ApiErrorBody` (what a failure is actually answered with: the
  flattened `ApiError` plus its request's `traceId`, also sent as the
  `X-Trace-Id` header);
  `handlers::routes::Routes` / `Reply` (see "Route list and generated
  client" above);
  `protocol::SyncedDoc` (the `GET /shares/{share_id}` response shape:
  `ended`/`filename`/`content`/`revision`, plus `folder` for a folder
  share; nothing identifies the owner, and no token, hash, or other
  internal id is ever present);
  `protocol::FileShareRequest` (the owner routes' body, just
  `identityToken`) / `protocol::FileShareResponse` (`shareId`) /
  `protocol::ShareStatusResponse` (`share: ShareStatus { shareId, ended } |
  null`), camelCase on the wire; `identity::resolve_verified_user_id`
  (the session lookup every owner route calls, see "Accounts" below);
  `auth::session::VerificationFailure` (why a token was refused, the body
  of `unauthorized`); `auth::{email, password, lockout, session}` (the
  D1-free account rules) / `crypto` (WebCrypto PBKDF2 and random bytes) /
  `handlers::auth` (the four account routes); `app_log::LogEvent` /
  `app_log::AuthEvent` / `app_log::Outcome` / `app_log::LogRow` (the typed
  `app_log` rows, see "App log (observability)" below);
  `file_history::VersionRow` (see "Version history" below); `db` (crate-private raw D1 query
  functions, one `.sql` file per query under `queries/`, loaded via
  `include_str!`).
- Ownership model: a share's `owner_user_id` column (backfilled from the
  pointed-at file by `0005_folder_shares.sql`; a file share's value always equals the file's `files.owner_user_id`, set from the same resolved caller at insert).
  `POST /files/{id}/share` answers `ApiError::NotFound` unless the resolved caller owns
  that file and it isn't in the bin; `stop`/`status` are owner-scoped in
  SQL the same way (`queries/end_share.sql`,
  `queries/get_share_status_by_file.sql`). Starting is idempotent:
  `queries/upsert_share.sql` inserts on first share and otherwise clears
  `ended_at` on the existing row (`ON CONFLICT(file_id)`), returning the
  file's one `share_id` — so re-sharing, from any device, reproduces the
  same link. There is no share-side content write at all: the owner's
  ordinary autosave (`POST /files/{id}/content`) is what a viewer sees.
- `share_id` generation moved server-side (`share_id::generate_unique_share_id`):
  no more client-derived id. Every id is exactly `SHARE_ID_LENGTH` (11)
  base64url characters; on the astronomically unlikely collision against an
  existing row it re-rolls a fresh candidate of the same length. That shape
  is defined only in `share_id.rs`: `share_id::share_id_format()`
  (`ShareIdFormat { length, pattern }`) feeds both the `share_id` path
  param's schema in the OpenAPI spec and the generated
  `shareIdFormat.json` the web client parses share links with — see "Route
  list and generated client".

### Accounts (email + password)

One account, an email and a password, proves identity for **both** cloud
file storage (`/files/*`) and Synced Share ownership. Sign-up is open, with
no email verification and no email sending: a forgotten password is reset
by the admin with `queries/admin/link_credentials_to_user.sql`, the same
query that hands a former GitHub user's `users` row to a new email.

- Routes (`src/handlers/auth.rs`; the rules in `src/auth/`, D1-free and
  tested in `tests/auth_*.rs`):
  - `POST /auth/sign-up` (`CredentialsRequest { email, password }` →
    `SessionResponse { sessionToken, email }`): `auth::email::normalize_email`
    trims and lowercases the email (`invalid_email` if it isn't one),
    `auth::password::password_weakness` wants at least
    `MIN_PASSWORD_LENGTH` (12) characters (`weak_password`), and a taken
    email answers `email_taken`. Creates the `users` and
    `password_credentials` rows and a first session.
  - `POST /auth/sign-in` (same body and answer): `incorrect_credentials`
    for an unknown email and a wrong password alike — an unknown email
    costs the same hashing and counts toward the lockout the same, so
    neither the answer, its timing nor the lockout tells them apart.
  - `POST /auth/sign-out` (`SignOutRequest { identityToken }`, 204): deletes
    that one session, whether or not it still existed.
  - `POST /auth/change-password` (`ChangePasswordRequest { identityToken,
    currentPassword, newPassword }`, 204): checks the current password under
    the same lockout as sign-in, stores the new hash and deletes the
    account's **other** sessions; the caller's survives.
- Passwords: PBKDF2-HMAC-SHA256 through WebCrypto (`crypto::pbkdf2_sha256`,
  `web-sys`'s `SubtleCrypto`), `PBKDF2_ITERATIONS` (100 000, the Workers
  cap) with a 16-byte random salt, stored self-describing as
  `pbkdf2-sha256$<iterations>$<salt>$<hash>` (`auth::password::PasswordHash`),
  so the iteration count can change later without breaking old hashes.
  Compared in constant time. A password only ever reaches
  `crypto::pbkdf2_sha256`.
- Sessions (`auth::session`, `sessions` table): a random 32-byte token,
  base64url (43 chars), handed to the browser once; only its SHA-256 hash
  is stored. A session lasts `SESSION_LIFETIME_MILLIS` (60 days) from its
  last use: `identity::resolve_verified_user_id` (called by every
  authenticated route, through `handlers::resolve_files_caller`) looks up
  an unexpired row by hash and slides `expires_at` forward only once less
  than 59 days is left, so most requests don't write. No live row answers
  `ApiError::Unauthorized` (a `VerificationFailure`) and logs
  `session_rejected`; a D1 failure is a retryable `internal` and keeps the
  token.
- Lockout (`auth::lockout`, `sign_in_lockouts`, per email whether or not it
  has an account): the `MAX_FAILED_SIGN_INS`th (5) wrong password in a row
  locks the email for `BASE_LOCKOUT_MILLIS` (15 min) × 2^level, the level
  rising each lockout up to `MAX_LOCKOUT_LEVEL`; while locked the password
  isn't checked and the answer is `sign_in_locked { retryAfterSeconds }`.
  A successful sign-in deletes the row.
- Rate limits: the Workers Rate Limiting bindings `SIGN_IN_RATE_LIMITER`
  (20 per 60 s) and `SIGN_UP_RATE_LIMITER` (3 per 60 s) in
  `cloudflare.config.ts`, keyed on `CF-Connecting-IP`, answer
  `rate_limited` (429). `RATE_LIMIT_PER_MINUTE_OVERRIDE` raises both for
  the local e2e worker.
- Web: the frontend holds no rule. `accountRequests.ts` calls the routes and
  maps each `ApiError` to an `AccountFailure`; `AccountAuthModal.tsx`
  (Sign in / Create account tabs) and `ChangePasswordModal.tsx` show the
  worker's answer, counting a lockout's `retryAfterSeconds` down
  (`accountDialogParts.tsx`). `useSyncedShareOwner.ts` owns which dialog is
  open (`accountDialog`, rendered by `AccountDialogs.tsx`) and `signOut`,
  which fires `POST /auth/sign-out` without waiting and clears the stored
  session. The header chip (`ProfileControl.tsx`) shows the email and is
  the only place to change the password or sign out.
- Owner UI (`useSyncedShareOwner(filename, cloudFileId)`, `ShareModal.tsx`):
  `cloudFileId` is the active file's `files.id`, `null` for a local or demo
  file (then `canSync` is false). Clicking "Start Sync" while signed out
  opens the account dialog instead. The hook holds no share state beyond the
  last `POST /files/{id}/share/status` answer, so every device sees the same
  live/stopped state and link; `startSync`/`stopSync` send only
  `identityToken`, never content.

### Version history

`files.content` stays the head. Each accepted content save that changes the
text also inserts a `file_versions` row (`file_id` — `REFERENCES files(id)
ON DELETE CASCADE` — `revision`, `newer_revision`, `reverse_patch`,
`saved_at`) holding the **reverse patch** from the new text back to the
previous one (`live-share-worker/migrations/0008_file_versions.sql`).

- Pure logic lives in `src/file_history.rs` (D1-free, property-tested in
  `tests/file_history.rs`, golden fixtures in `tests/file_history_golden.rs`):
  `reverse_patch`, `apply_patch` (diffy unified diffs),
  `reachable_versions(head_revision, rows)` (follows `newer_revision`
  links from the head and stops at the first gap),
  and `rebuild(head_content, head_revision, rows, revision)`.
- Writes: `handlers::files::update_file_content` reads the previous head
  before the compare-and-swap. Once the CAS applies, it records the version
  as a best-effort step (any error is logged and does not fail the save). A
  save that leaves the text unchanged moves the newest row's
  `newer_revision` forward (`queries/advance_file_version_link.sql`)
  instead of storing an empty patch. Each insert is batched with
  `queries/prune_file_versions.sql`, which keeps the newest
  `file_history::MAX_VERSIONS_PER_FILE` (100) rows.
- Reads: `POST /files/{id}/versions` lists the reachable versions
  (`protocol::ListFileVersionsResponse`). `POST /files/{id}/versions/get`
  rebuilds one revision and returns its `content`
  (`protocol::FileVersionResponse`). Both routes are owner-scoped
  in SQL (`queries/list_file_versions.sql`).
- Web: `web/src/components/fileHistory/FileHistoryModal.tsx`, opened from
  the file actions menu's "History" item (cloud backend only). It diffs the
  editor's current text against the chosen version with
  `web/src/components/PoentaroDiffEditor.tsx`, the Monaco diff the part stream
  editor also uses (inline or side by side, `hooks/useDiffLayout.ts`). Restore puts
  the version's text in the editor, and the normal save path stores it as a
  new head, so a restore adds a version and never rewrites history.

### Origin migration

The app moved from `jianpu-generator.pages.dev` to `poentaro.pages.dev`.
Browsers keep `localStorage` per origin, so a plain redirect would strand
every local file and sign the user out. The old origin instead serves a
bridge page that hands the browser's saved data to the new origin through a
one-time **origin migration** handoff in the worker.

- Worker (`src/origin_migration.rs`, pure and tested in
  `tests/origin_migration.rs`; handlers in `src/handlers/origin_migrations.rs`;
  table `origin_migrations` (`id`, `payload`, `created_at`) since
  `0010_origin_migrations.sql`):
  - `POST /origin-migrations` (`OriginMigrationUploadRequest { payload }` →
    `OriginMigrationUploadResponse { id }`): anonymous, rate limited per IP by
    `ORIGIN_MIGRATION_RATE_LIMITER` (5 per 60 s), refused with
    `origin_migration_too_large` (413) over
    `MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES` (1 900 000, under D1's row limit).
    The payload is opaque text the worker never parses. The id is
    `ORIGIN_MIGRATION_ID_BYTES` (24) CSPRNG bytes, base64url: the only
    credential.
  - `POST /origin-migrations/claim` (`OriginMigrationClaimRequest { id }` →
    `OriginMigrationClaimResponse { payload }`): deletes the row as it reads
    it (`DELETE ... RETURNING`), so a handoff is read at most once; unknown,
    already claimed and expired (older than `ORIGIN_MIGRATION_TTL_MILLIS`,
    1 h) ids all answer `not_found`. The daily cron prunes expired rows.
- Payload (`web/src/originMigration/originMigrationPayload.ts`,
  `OriginMigrationPayload`, version 1): every `localStorage` entry whose key
  starts with `jianpu:` and every cloud outbox IndexedDB database
  (`jianpu-outbox:<account>`) with its records. These are the legacy names
  (`LEGACY_LOCAL_STORAGE_PREFIX`, `LEGACY_OUTBOX_DATABASE_PREFIX` in
  `web/src/legacyStorage/legacyStorageNames.ts`): the old origin only ever
  ran the app before the rename to Poentaro (see "Rename to Poentaro").
- Bridge page (`web/bridge/index.html`, `web/src/bridge/`, built by
  `pnpm build:bridge` with `web/vite.bridge.config.ts` into
  `web/dist-bridge/`): served by the old origins for every path. If the old
  origin is already marked migrated (`jianpu:origin-migrated:v1`, a legacy
  name kept on purpose since old origins already hold it) or holds
  nothing, it redirects at once; otherwise it uploads the payload, sets the
  mark and redirects with `?migrate=<id>`. `bridgeRedirectUrl` keeps the
  path (minus the old base path), query and fragment, which share links
  carry their payload in. A failed upload shows "Try again" / "Continue
  without my files" and leaves the mark unset. The old origin's data is
  never deleted. The old Cloudflare Pages project is deployed with
  `web/functions/` too, so link-preview crawlers fetching an old `?s=` link
  still get the share's title (the bridge carries the same `og:` tags).
- Deploy (`.github/workflows/pages.yml`): the app goes to the `poentaro`
  Cloudflare Pages project; once that succeeds, the bridge goes to the old
  `jianpu-generator` project.
- GitHub Pages: the old `jianpu-generator.github.io/jianpu-generator/` is
  served by the separate repo `jianpu-generator/jianpu-generator.github.io`
  (the old org's site, kept after this repo moved to `poentaro/poentaro`):
  a static page at every path that only redirects, keeping the path minus
  `/jianpu-generator/`, the query and the fragment. Saved files are not
  carried over from GitHub Pages. Nothing in this repo deploys it.
- Import on arrival (`web/src/originMigration/importOriginMigration.ts`,
  run by `prepareStorage` (`web/src/legacyStorage/prepareStorage.ts`), which
  `web/src/main.tsx` loads and awaits before the app renders, only when the
  URL carries `?migrate=` or legacy keys are saved): reads `?migrate=`,
  strips it from the URL, claims the handoff and saves each carried
  `localStorage` entry, under its current name (`currentLocalStorageKey`),
  only where the new origin has none, so a session or
  preference already set here wins. `poentaro:files:v1` is merged instead by
  `merge_migrated_file_store(current, carried, context)`
  (`src/file_tree/origin_migration.rs`, WIT export
  `file-store-merge-migrated`, TS `mergeMigratedFileStore` in `fileStore.ts`):
  an empty current store takes the carried one whole; otherwise every carried
  file, bin entry and explicit folder goes under the top-level folder
  `MIGRATED_FOLDER_NAME` ("From jianpu-generator.pages.dev", numbered when
  taken), keeping its file id unless that id is already used here, and the
  current active file stays active. A store the merge rejects is kept under
  `poentaro:files:v1:unmerged-origin-migration` rather than lost. A carried
  outbox database is restored only when this origin's outbox for that
  account is empty. Covered by `web/e2e/features/origin-migration.feature`,
  where the e2e config serves the bridge on port 5184 as the old origin.

### Rename to Poentaro

The app was first called jianpu-generator; it is now Poentaro. Every name
that meant the app was renamed: the crates (`poentaro`, `poentaro-wasm`, the
`poentaro` CLI), the WIT package (`poentaro:wasm`), TS modules and hooks,
the file extension (`.jianpu` → `.poentaro`), the `localStorage` prefix
(`jianpu:` → `poentaro:`), the outbox databases (`jianpu-outbox:<account>` →
`poentaro-outbox:<account>`), the asset cache, the Monaco language id and
the source tag embedded in exported files. The notation, once called
jianpu, is now numbered notation: the numbered view, `Notation::Numbered`
(`notation: numbered`), `NumberedPitch`.

What still says `jianpu`, on purpose:

- **Cloud resources**, which can't be renamed without recreating them: the
  worker `jianpu-live-share-worker-rs` (its workers.dev host is what every
  deployed client calls), the D1 databases `jianpu-live-share` and
  `jianpu-live-share-staging`, the B2 bucket `jianpu-live-share-backups`,
  the Keychain service `jianpu-backup-reader`, and the old origins
  (`jianpu-generator.pages.dev`, the old GitHub org). New resources get
  `poentaro` names (see "Naming" in `AGENTS.md`).
- **Legacy names read for old data**, each next to the current one:
  - The extension: `file-extension.json` (`legacyExtension`) is read by Rust
    (`build.rs` → `file_tree::LEGACY_FILE_EXTENSION`) and TS
    (`web/src/fileExtension.ts`). A legacy name still opens, shows without
    its extension (`strip_file_extension`, `stripFileExtension`) and can be
    imported.
  - Browser storage: `web/src/legacyStorage/`. `hasLegacyLocalStorage()`
    makes `main.tsx` await `prepareStorage()` before rendering, which runs
    `renameLegacyStorage()` (copies each legacy outbox database to its
    current name and deletes it, unless the current one already queued
    edits; moves the asset cache; moves every `jianpu:` key to `poentaro:`
    last, so an interrupted rename finishes next visit. A current key wins,
    except that a differing legacy file store is kept under
    `poentaro:files:v1:legacy-backup`), then the origin-migration import,
    then `renameLegacyFiles()`.
  - Local file names: `file_tree::rename_legacy_extensions`
    (`src/file_tree/extension.rs`, WIT `file-store-rename-legacy-extensions`,
    TS `renameLegacyExtensions`) renames every active and binned `.jianpu`
    file to `.poentaro`, keeping content and id, unless the new path is
    taken (then it keeps its legacy name). `merge_migrated_file_store` runs
    it on the carried store first.
  - Cloud file names: D1 migration `0011_poentaro_file_extension.sql`, which
    the worker deploy applies, renames `files.name` the same way (a taken
    name keeps its legacy one; `updated_at` untouched).
    `queries/admin/preview_poentaro_file_extension.sql` lists what it renames,
    or what it left.
  - Embedded source: exports carry `<metadata id="poentaro-source">` (SVG) and
    `/PoentaroSource` (PDF); `source_embed` also reads the legacy
    `jianpu-source` and `/JianpuSource`, so old exports still import.
  - Preset notation: `Notation::from_source_value` (`src/ast/parsed/preset.rs`)
    reads a preset's legacy `notation: jianpu` as `Notation::Numbered`.
  - The bridge collects the legacy names (see "Origin migration").

### App log (observability)

An append-only `app_log` D1 table (`live-share-worker/migrations/0006_app_log.sql`)
that records what the worker did so production behaviour — first of all why
someone got signed out or can't sign in — can be answered from data. It
is queried with SQL, by agents and the owner, through the saved queries in
`crates/live-share-worker/queries/observability/` (see "Debugging production"
in `AGENTS.md` for the run command).

- Columns: `at` (ms since epoch), `flow` (`auth` or `request`), `kind` (the event),
  `outcome` (`ok`/`rejected`/`unavailable`/`error`), `user_id` (when known),
  `subject` (`email:<email>` or `token:<token tag>`), `worker_version` (the git SHA `build.rs`
  embeds as `WORKER_GIT_SHA`), `origin`/`user_agent` (request headers, on every
  request-driven row), `detail` (the event payload as JSON), `trace_id` (the
  request's **trace ID**, migration `0009_app_log_trace_id.sql`). Indexed on `at`,
  `(flow, kind, at)`, `subject` and `trace_id`.
- Key types (`src/app_log.rs`, pub and D1-free, tested in `tests/app_log.rs`):
  `app_log::LogEvent` (one row's event, adjacently tagged
  `{"flow", "event"}`; `LogEvent::Auth(AuthEvent)` or
  `LogEvent::Request(RequestEvent)`) /
  `app_log::RequestEvent` (`RequestFailed { route, status, code, message }`:
  one per non-2xx response) /
  `app_log::AuthEvent` (the account and session life cycle, adjacently
  tagged `{"kind", "detail"}`: `SignedUp`, `SignedIn`, `SignInFailed`
  (`reason`: `wrong_password`/`unknown_email`/`locked`), `LockedOut`,
  `SignedOut`, `PasswordChanged`, `SessionRejected`, each wrapping a payload
  struct of the same name; GitHub sign-in's `TokenIssued`,
  `TokenExchangeFailed`, `TokenVerified`, `TokenRejected`,
  `VerificationUnavailable`, `TokenRevoked` and `TokenRevokeFailed` stay
  only so their rows keep decoding until they age out) / `app_log::Outcome` (derived from the variant, never chosen at
  a call site) / `app_log::LogRow` (`LogEvent::to_row`'s split into the
  `flow`/`kind`/`outcome`/`subject`/`detail` columns; `LogEvent::try_from(&LogRow)`
  reassembles it) / `app_log::RequestMeta` (`Origin`/`User-Agent`, the
  request's `trace_id`, and `resolved_user_id`, which the session lookup
  fills in so a later `request_failed` row names the caller).
  Schema-evolution rule: rows must keep decoding for their 30 days, so only
  add fields, each with `#[serde(default)]`; a changed shape is a new variant.
- Writes: `db::record_app_log` (`queries/insert_app_log.sql`) is
  best-effort — a failed insert is reported with `console_error!` and never
  fails the request. No password, raw token or full hash is ever stored,
  only a session's **token tag**. Where events are logged:
  `handlers::auth` (`signed_up`, `signed_in`, `sign_in_failed`,
  `locked_out`, `signed_out`, `password_changed`) and
  `identity::resolve_verified_user_id` (`session_rejected`; a valid
  session logs nothing, so not once per request). Every row carries its request's `RequestMeta`: `Routes::post`
  hands each handler one, read before the body is consumed, and the
  authenticated `/files/*`, `/folders/*` and `/shares/overview` routes pass
  it to the session lookup through `resolve_files_caller`. Only `post_if`'s
  `/e2e/reset`, which logs nothing, gets none.
- Failed requests: every request gets a trace ID (`crypto::new_trace_id`, 16
  hex chars) when it arrives. Every non-2xx answer goes through
  `handlers::fail`, which writes one `request_failed` row under that ID and
  answers an `ApiErrorBody` with the `X-Trace-Id` header (exposed by CORS);
  unmatched routes and router errors in `src/lib.rs` take the same path. The
  web shows the ID with the error (`components/TraceId.tsx`, copyable), read
  by `WorkerRequestError.traceId`; a request that never reached the worker
  has none, and its error says it is a network error instead
  (`NetworkFailure`).
  `queries/observability/by_trace_id.sql` looks up every row of one reported
  ID; `request_failures.sql` lists recent failures.
- Retention: the `#[event(scheduled)] async fn scheduled` entry point in
  `src/lib.rs`, fired by the daily cron in `cloudflare.config.ts`
  (`triggers.scheduled`), deletes rows older than
  `app_log::APP_LOG_RETENTION_MILLIS` (30 days) via `db::prune_app_log`
  (`queries/prune_app_log.sql`), `APP_LOG_PRUNE_BATCH_SIZE` (5000) rows per
  statement and at most `APP_LOG_PRUNE_MAX_BATCHES` (10) statements per run.
  Locally, `cf dev` fires it on `GET /cdn-cgi/local/scheduled`.

### Backups

- `.github/workflows/d1-backup.yml` runs `wrangler d1 export` against
  production daily (04:41 UTC) and uploads the gzipped SQL dump to the
  private Backblaze B2 bucket named by `backupBucket` in `deploy.json`, as
  `daily/<YYYY-MM-DD>.sql.gz`. The 1st of each month's dump is also stored as
  `monthly/<YYYY-MM-DD>.sql.gz`. B2 rather than R2 because R2 needs a payment
  method on the Cloudflare account, and B2 keeps the backups outside the
  account they protect. Other options were ruled out: GitHub Actions
  artifacts can be downloaded by any signed-in GitHub user (the repo is
  public), and Workers KV or a second D1 database would sit in the same
  Cloudflare account.
- Retention: the bucket's lifecycle rules (`backup-lifecycle.json`) delete
  `daily/` files after 30 days and `monthly/` ones after 365.
  `just setup-backup-bucket` creates the bucket and applies the rules, and is
  safe to rerun after editing them. It needs a key with `writeBuckets`.
- Keys:
  - The workflow's `B2_KEY_ID` / `B2_APPLICATION_KEY` secrets are a key
    scoped to the bucket with only `listBuckets,writeFiles`. A leaked secret
    can add files but can't read or delete them.
  - The read-only key `just restore-backup` uses lives in each laptop's
    macOS Keychain (service `jianpu-backup-reader`). See "Handling secrets" in
    `AGENTS.md` for setting up another laptop.
  - Only the Backblaze master key can delete backups or change the rules.
    It's kept in the owner's password manager, never in a secret or a file.
  - The export uses the deploy workflow's `CLOUDFLARE_API_TOKEN`.
  - The worker has no access to the bucket.
- Silent failure: a failed run only emails the owner, and GitHub turns off
  scheduled workflows after 60 days without commits. To check a backup ran,
  look at the latest run in the Actions tab, or list the bucket with the
  reader key (`uvx b2==5.0.0 ls b2://jianpu-live-share-backups/daily/`).
- Within the last 7 days, D1 Time Travel (`wrangler d1 time-travel restore`)
  restores production to any minute without these dumps. The dumps cover
  older points and the database itself being deleted.
- Restore: `just restore-backup <date>` loads a dump into the local D1
  database. To restore production, download it with
  `b2 file download b2://<bucket>/<key> <file>`, gunzip, and run it with
  `wrangler d1 execute <database> --remote --file` into a new, empty
  database (the dump contains `CREATE TABLE` statements), then point
  `d1DatabaseId` at it.

### D1 query checking

`sqlx`'s "sqlite" feature can't be a normal dependency of this crate — its
`libsqlite3-sys` C build doesn't target `wasm32-unknown-unknown` (see the
long comment on the `[build-dependencies] sqlx` entry in
`crates/live-share-worker/Cargo.toml`). So `sqlx` appears twice, scoped
away from the wasm build both times:

- As a `[build-dependencies]` entry, used only by `build.rs` to apply
  `live-share-worker/migrations/*.sql` to a local shadow SQLite database
  (`crates/live-share-worker/shadow.sqlite`, gitignored) at build time.
- As a `[dev-dependencies]` entry, used only by
  `crates/live-share-worker/tests/query_syntax.rs` (a native-only test,
  not run for `wasm32-unknown-unknown`) to *prepare* (not execute) each
  `.sql` file under `queries/` against that shadow database — the same
  `include_str!`-loaded query text `src/db/`'s submodules use for real
  `D1Database::prepare()` calls at runtime. This is the pragmatic stand-in
  for `sqlx::query_file!`'s compile-time type-checking, which this crate's
  wasm-targeted compile can't use: it catches a broken/typo'd query
  (unknown table/column, invalid SQL) but not bound-parameter-type or
  result-column-name mismatches, and not D1-vs-SQLite dialect drift.

### Glossary additions

| Term | Definition |
|------|-----------|
| **Shared folder viewer** | What someone opening a folder link sees: the shared folder as the root of a read-only tree (`shared_folder_view`, relative paths, empty folders kept, no Demo files, no actions), the same toolbar as for a single shared file, and an "Import this folder" button that copies it into their own files under a unique name (`Hymns 2` when `Hymns` is taken). Works for both link kinds (synced `?s=` and static `#share-folder=`); a stopped or unknown synced link shows the "no longer shared" notice instead. |
| **Synced Share worker** | The standalone Cloudflare Worker (`crates/live-share-worker`) backing the Synced Share feature's anonymous `GET /shares/{share_id}` and owner-only `POST /files/{id}/share[/stop|/status]` routes, as well as the cloud storage backend's `/files/*` API, described above. |
| **resolve_verified_user_id** | `identity::resolve_verified_user_id`: the session lookup every authenticated route calls instead of ever trusting a client-asserted identity — hashes the token, finds an unexpired `sessions` row and slides its expiry. See "Accounts (email + password)". |
| **session token** | The random 32-byte (43 base64url chars) token `POST /auth/sign-up`/`/auth/sign-in` hands the browser, sent as `identityToken`. Only its SHA-256 hash is stored (`sessions.token_hash`); it lasts 60 days from its last use, and signing out or changing the password elsewhere ends it. |
| **sign-in lockout** | A `sign_in_lockouts` row: 5 wrong passwords in a row for one email (account or not) lock it for 15 min × 2^level, answered `sign_in_locked { retryAfterSeconds }`; a successful sign-in clears it. Separate from the per-IP rate limits. |
| **app log** | The `app_log` D1 table: an append-only, 30-day record of what the worker did (today: the account and session life cycle), written best-effort through `app_log::LogEvent` and read with the saved SQL in `queries/observability/`. Query it before guessing at production behaviour — see "App log (observability)". |
| **trace ID** | The 16-hex-char ID the worker gives each request (`X-Trace-Id` header, `traceId` in every error body, `app_log.trace_id`). The web shows it with every failed request so a reported failure can be looked up with `queries/observability/by_trace_id.sql`. |
| **flow** | `app_log.flow`: which part of the system a row is about (`auth` for the account and session life cycle, `request` for failed requests), the outer tag of `app_log::LogEvent`. New flows are new `LogEvent` variants in the same table, not new tables. |
| **subject** | `app_log.subject`: what a row is about as a typed tag (`token:<token tag>`), derived from the event, `email:<email>` for account events, `token:<token tag>` for session events (signed out, rejected), so rows of one account or session join across kinds. |
| **token tag** | `app_log::token_tag`: the first 12 hex chars of a session token's SHA-256 hash (`auth::session::hash_token`). Enough to tell tokens apart in `app_log`, never the token or full hash. |
| **VerificationFailure** | `auth::session::VerificationFailure { reason, failed_at, attempts }`: why a request's token was refused (no live session), returned as `ApiError::Unauthorized` (a `401` body with `code: "unauthorized"` beside these fields). `attempts` is always 1, kept from the GitHub era so the web reads it unchanged. Never carries the token or its hash. |
| **ApiError** | `api_error::ApiError`: the `code`-tagged union every Synced Share worker route answers a failure with; its HTTP status is derived from the variant. `web/` switches on the generated `code` literals (`WorkerRequestError.apiError`), never on raw statuses. |
| **Worker client** | The typed `openapi-fetch` client for the Synced Share worker (`web/src/syncedShare/workerClient.ts`), generated from the OpenAPI spec that `handlers::routes()` builds from the Rust handlers' own signatures — see "Route list and generated client". |
| **owner_user_id** | A file's owner (`files.owner_user_id`, an internal `users.id`), and so also the owner of that file's share — `shares` has no owner column of its own. Replaced the old KV model's bearer `ownerToken`, which any first writer could claim. |
| **Account** | An email + password (`password_credentials`) on a `users` row, signing in to **both** cloud file storage and Synced Share ownership. Held in the browser as `accountAuth.ts`'s `{ token, email }` under `poentaro:account-auth:v2`. See "Accounts (email + password)". |
| **static folder link** | A `#share-folder=<payload>` link carrying a compressed `FolderSnapshot` of a folder at one moment; works for local and cloud folders, never updates, and is not offered above `MAX_STATIC_FOLDER_LINK_CHARS` (the dialog suggests a synced link instead). |
| **share overview** | `file_tree::ShareOverview`, the owner's live file and folder links as `POST /shares/overview` reports them; everything the share UI shows (markers, "via X" tags, warnings) is derived from it in Rust. |
| **selection unit** | `file_tree::SelectionUnit`: one thing a multi-select action applies to — a file, or a folder whose files are all ticked (moved and deleted as a folder, so its structure survives). Folders inside a ticked folder are not listed again. |
| **shared via** | `file_tree::SharedVia`: the innermost shared folder strictly above a file or folder, shown as a "via <name>" tag and "Shared via <name>" menu item. |
| **folder share** | A `shares` row with `folder_prefix` set instead of `file_id`: a synced, read-only link to a whole folder (see "Folder shares" in the worker section). It follows the folder path, not file ids, so files moved in or out change what it shows and renaming/moving the folder re-points the prefix in the same transaction. |
| **share** | A `shares` row: a pointer from a `share_id` (the `?s=` link) to one cloud `files` row (or, for a **folder share**, to a folder path), not a copy of its content. A viewer reads the file's saved content directly, so the owner's autosave is the only write path and a link can only ever show its own file. At most one per file (`shares.file_id` is unique), so re-sharing reproduces the same link; stopping sets `ended_at` rather than deleting, and a binned file reads as ended until restored. Only cloud files can be shared this way. |

## Cloud outbox (`crates/cloud-outbox`)

The outbox is the pure, I/O-free state machine that decides what a signed-in
user's cloud file edits do next: which message to send, when to retry, when to
stop and ask the user. It is compiled to wasm and exposed through the
`outbox-*` functions of `crates/poentaro-wasm/wit/world.wit` (guest side:
`crates/poentaro-wasm/src/component/guest_outbox.rs`, conversions in
`outbox_conversion.rs`). Every function takes a `Queue` and returns a new one;
nothing is mutated in place and the crate never reads a clock, a random number
or the network (`now_ms` and `jitter_unit` are passed in).

### Modules and entry functions

| Module | Entry |
|--------|-------|
| `enqueue` | `enqueue(queue: Queue, file_id: &str, message: Message, message_id: &str, now_ms: u64) -> Queue` |
| `scheduling` | `begin_send(queue: Queue, now_ms: u64) -> BeginSend` (`BeginSend { queue, request: Option<SendRequest> }`); `next_wake_ms(queue: &Queue) -> Option<u64>` |
| `transitions` | `on_result(queue: Queue, file_id: &str, message_id: &str, result: SendResult, now_ms: u64, jitter_unit: f64) -> Queue`; `resolve(queue: Queue, file_id: &str, resolution: Resolution) -> Queue`; `resolve_signed_in(queue: Queue) -> Queue`; `record_revision(queue: Queue, file_id: &str, revision: i64) -> Queue` |
| `backoff` | `backoff_ms(consecutive_failures: u32, jitter_unit: f64) -> u64` |
| `codec` | `encode_queue(queue: &Queue) -> Vec<StoredRecord>`; `decode_queue(records: &[StoredRecord]) -> Result<Queue, DecodeError>` |
| `summary` | `summarize(queue: &Queue, now_ms: u64) -> QueueSummary` |
| `history` | `record_sent(queue_before: &Queue, existing: &[StoredRecord], file_id: &str, message_id: &str, revision: i64, now_ms: u64) -> HistoryWrite`; `rows(queue: &Queue, sent_records: &[StoredRecord], file_names: &[FileName], filter: &HistoryFilter, now_ms: u64) -> Vec<HistoryRow>`; `expired_keys(records: &[StoredRecord], now_ms: u64) -> Vec<String>` |
| `merge` | `merge_three_way(base: &str, mine: &str, theirs: &str, accepts: impl Fn(&str) -> bool) -> MergeOutcome` |

The WIT exports are `outbox-enqueue`, `outbox-begin-send`,
`outbox-next-wake-ms`, `outbox-on-result` (its result parameter is named
`send-result`), `outbox-resolve`, `outbox-resolve-signed-in`,
`outbox-record-revision`, `outbox-encode-queue`, `outbox-decode-queue`,
`outbox-summarize`, `outbox-record-sent` and `outbox-history-rows`
(conversions in `history_conversion.rs`). `merge_three_way` has no `outbox-*`
WIT export.

### Key types (`src/types.rs`)

All derive `Debug, Clone, PartialEq, Serialize, Deserialize`; structs use
camelCase, enums are tagged with `kind`.

- `Queue { lanes: Vec<Lane>, revisions: Vec<FileRevision> }`, with
  `FileRevision { file_id, revision }` recording the last server revision known
  per file.
- `Lane { file_id, status: LaneStatus, in_flight: Option<String>, messages: Vec<QueuedMessage> }`.
  The head message is index 0.
- `QueuedMessage { message_id, created_at_ms, message: Message, attempts: Vec<Attempt> }`.
- `Message`: `CreateFile { name, content }`, `SaveContent { content }`,
  `RenameFile { to }`, `TrashFile`, `RestoreFile`, `RenameFiles { moves: Vec<RenameMove { file_id, to }> }`. The file id lives on the lane, except for `RenameFiles` (a folder move): it sits alone on a synthetic lane `batch:<message_id>` (`BATCH_LANE_PREFIX`) so a half-delivered move cannot exist; no revision is recorded for that lane and it is removed once the batch is delivered.
  Ordering (`scheduling::is_blocked`): a batch is not sent before a pending `CreateFile` (placeholder creates included) of any member file, nor before an earlier rename/trash/restore of a member file or an earlier batch sharing a file; a later per-file rename/trash/restore of a member waits for the batch. `SaveContent` is never blocked. `applyPendingStructure` (`cloudBackendOutbox.ts`) replays pending messages in `createdAtMs` order, including batch moves.
- `Attempt { at_ms, outcome: AttemptOutcome }`; `AttemptOutcome`: `Succeeded`,
  `Transient`, `Unauthorized`, `RevisionConflict`, `NameTaken`, `NamesTaken { adjustments }`, `Permanent`.
  A delivered message's log (kept in its `sent:` record) ends in `Succeeded`;
  that is what makes a history row "Sent".
- `LaneStatus`: `Idle`, `Draining`, `Waiting { retry_at_ms }`, `NeedsSignIn`,
  `NeedsMerge { current_revision }`, `Failed { reason }`.
- `SendRequest { file_id, message_id, message, expected_revision }`
  (`expected_revision` is 0 when no revision is recorded).
- `SendResult`: `Ok { revision }`, `Transient`, `Unauthorized`,
  `Conflict { current_revision }`, `NameTaken`, `NamesTaken { adjustments: Vec<NameAdjustment { rejected, replacement: Option<String> }> }`, `Permanent`. `NamesTaken` is the batch collision answer: `transitions::apply_names_taken` rewrites the matching moves (dropping those with no replacement) and resends the whole batch; no match fails the lane with `name_taken`, and all moves dropped completes it. This is what the
  shell reports back after performing a `SendRequest`.
- `Resolution`: `Retry`, `Discard`, `MergedAndSave { content, server_revision }`.
- `StoredRecord { key, value }` and `DecodeError` (`UnsupportedVersion`,
  `Corrupt`); the persisted layout is `meta:version`, `meta:revisions` and one
  `lane:<file_id>` record each, versioned by `QUEUE_SCHEMA_VERSION`.
- `MergeOutcome`: `Clean { text }` or `Conflicted { text, has_markers }`.
- History (`src/history.rs`): `SentEntry` (a delivered message, stored under
  `sent:<sent_at_ms>:<message_id>`, outside the queue codec), `HistoryWrite
  { put, deletes }`, `HistoryFilter` (query, `StatusFilter`, `MessageKind`,
  file, max age, min size, `AttemptsFilter`), `FileName`, and `HistoryRow`
  (one queued or delivered message with its `HistoryStatus`, size, attempts,
  pretty-printed payload JSON and `payload_matches_query`). Retention keeps
  sent records for `RETENTION_MS` (7 days), at most `MAX_SENT_ENTRIES` (200)
  and `MAX_SENT_BYTES` (5 MB), always keeping the newest.
- `QueueSummary { lanes, total_pending, lanes_needing_attention, has_unsynced }`
  of `LaneSummary { file_id, reason: LaneReason, pending_messages, retry_in_ms, attempts, failure_reason }`;
  `LaneReason` is `Syncing`, `WaitingToRetry`, `PossiblyStuck`, `NeedsSignIn`,
  `NeedsMerge`, `Failed`. A lane is `PossiblyStuck` after
  `POSSIBLY_STUCK_AFTER` failed attempts.

### TypeScript shell (`web/src/storage/outbox/`)

The shell only does I/O, timers and rendering; every decision is a wasm call.

- Existing: `outboxStore.ts` (persists `StoredRecord`s), `memoryOutboxStore.ts`
  (in-memory store), `outboxLock.ts` (single-tab lock), `outboxScheduler.ts`
  (timer driven by `next_wake_ms`).
- `sendMessage.ts` (performs a `SendRequest` over HTTP and maps the
  response to a `SendResult`), `outboxLooper.ts` (the Looper; runs on the
  leader tab only), `baseSnapshots.ts` (Base snapshot storage under
  `base:<fileId>`), `conflictFlow.ts` (drives `NeedsMerge` through
  `merge_three_way`: a clean merge is saved at once, a conflicting one is kept
  for the merge editor) and `keepBoth.ts` (saves the local text as a
  `(conflicted copy <date>)` file, then discards the original lane).
- The Looper calls `outbox-record-sent` just before a successful result is
  applied, writes the returned record and expired-key deletes to the store,
  and serves `historyRows(fileNames, filter)` from the cached `sent:` records
  (`refreshHistory()` re-reads them, e.g. for a follower tab).
- `web/src/storage/cloudBackend.ts` enqueues every remote update; structural
  operations apply locally at once and queue `create-file`, `rename-file`,
  `trash-file` or `restore-file`.
- UI: `web/src/components/syncPanel/` (reads `QueueSummary`; `SyncPanel` is
  the "Sync status" section listing unsettled files with Retry now, Review
  and merge, Keep both, Discard and Download my copy),
  `web/src/components/outboxHistory/` (the one full-screen dialog opened from
  the save badge: `OutboxHistoryDialog` hosts `SyncPanel` above the history
  and the Copy diagnostics button, reads `historyRows`, `OutboxHistoryView` renders the
  filter controls and table, `historyFilterState.ts` maps the controls to a
  `HistoryFilter`; matching, sorting and retention stay in Rust) and
  `web/src/components/mergeEditor/` (`MergeEditorContainer` offers Save,
  Keep both and Discard).

### Glossary additions (outbox)

| Term | Definition |
|------|-----------|
| **Outbox** | The persisted `Queue` of not-yet-acknowledged cloud edits plus the pure functions in `crates/cloud-outbox` that advance it. |
| **Lane** | The per-file ordered queue of messages (`Lane`) with its own `LaneStatus`. Lanes drain independently, so one halted file never blocks another. |
| **Message** | One queued cloud operation (`Message`) with a `message_id` and its recorded attempts (`QueuedMessage`). A `SaveContent` coalesces into a directly preceding, not in-flight `SaveContent`. |
| **History** | Every message the outbox knows about, newest first: those still queued plus the delivered ones kept under `sent:` keys until retention prunes them. Shown in the outbox history dialog. |
| **Looper** | The TS loop (`outboxLooper.ts`) that repeatedly calls `begin_send`, performs the request, and feeds the `SendResult` to `on_result`, sleeping until `next_wake_ms`. |
| **Halt** | A lane stopping instead of retrying: `NeedsSignIn`, `NeedsMerge` or `Failed`. It stays halted until a `Resolution` (or `resolve_signed_in`) moves it back to `Draining`. |
| **Base snapshot** | The last content known to match the server for a file (kept by `baseSnapshots.ts`); the `base` input of `merge_three_way` when a revision conflict needs merging. |
