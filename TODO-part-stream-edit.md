# TODO: Part Stream Edit

Edit **one part** of the `# score` as a single continuous stream of musical
elements (no measure groups, no barlines, no directives), then **Apply** it
back into the measure-grouped source. The point: make it easy to shift the
notes that come before or after a given measure — insert or delete in measure
5 and everything after it re-flows across the following barlines.

Branch: `part-only-text-view` (worktree `.claude/worktrees/part-only-text-view`).
All phases land as separate commits in **one PR**.

---

## Progress

- [x] Phase 1 — Rust core: extract / split / apply for `notes`, `chords`, `percussion` parts
- [x] Phase 2 — Rust: `follow[X]` parts (`@` placeholder) and notes-bound lyric parts
- [x] Phase 3 — Wasm/WIT exports
- [x] Phase 4 — Web: part picker modal, editable diff view, Apply, markers, e2e
- [x] Phase 5 — Docs (`syntax.md`, `ARCHITECTURE.md`) and PR

Tick each box in the same commit that completes the phase.

---

## History: why this is the third attempt

Two earlier attempts were built and removed. Read them before writing code
(`/usr/bin/git show <sha>`), but **do not resurrect their code**:

- `1714290e` — "Solo Edit": per-part view with live write-back.
- `5b9f7c50`, `9f70b77b`, `468d38d9`, `ee37ad37` — "Unzipped View": every
  part flattened at once, live write-back.
- `2690409b` — removal of both, with the reasons. Its parent has
  `PLAN-unzipped-view.md` (`/usr/bin/git show 2690409b^:PLAN-unzipped-view.md`).

What broke them, and how this design avoids it:

| Failure | Avoided by |
|---|---|
| Live write-back re-extracted the view while the user typed, clobbering edits | No live write-back. The stream is extracted once when the modal opens; the source changes only on **Apply**. |
| A whole-document view had to reconcile every part at once | Only **one** part is edited at a time. Every other part's lines are never touched. |
| Lyric measures were split by heuristics (token counts, LCS diff) | Lyric measures are split by the **target part's lyric slots** — an intrinsic count, no guessing. Chord-bound lyric parts are excluded. |
| Implicit trailing padding (`1 2` silently stretched to fill the bar) was lost when measures were flattened, so later notes shifted | Extraction **materializes** implicit padding as explicit `-` tokens. |
| Tuplet durations ignored when re-barring | Split uses the real (rescaled) tuplet durations. |
| A separate diagnostic span-mapping layer between views | Stream diagnostics are produced directly in stream coordinates by the split step; the main source is only touched on Apply. |
| Unchanged measures were re-serialized, losing formatting/comments | **Only changed measures are rewritten**; every unchanged measure keeps its original line byte-for-byte. |

---

## Confirmed decisions (from the user — do not re-litigate)

1. One part at a time, chosen from a modal listing the eligible parts.
2. Eligible kinds: `notes`, `chords`, `percussion`, `follow[X]` parts, and
   lyric parts whose target is a `notes` part (or a `follow` of one).
   **Lyric parts bound to a `chords` part are not eligible.**
3. Directives (`bpm=`, `key=`, `time=`, `label=`, `break`, …) are hidden in
   the stream and never move. Their time signatures still define measure
   capacity.
4. **Barlines are banned** in the stream: `|` is an error.
5. Measures where the part has no line are shown as rests (non-follow parts).
6. `follow[X]` measures where the part has no line (so it copies X) are shown
   as a dedicated placeholder token: **`@`**, one per measure. Applying `@`
   leaves the measure unwritten, so the round trip is lossless.
7. Write-back happens only on **Apply**. The editor is a live, **editable diff
   view** (like VS Code's diff): original stream on the left (read-only),
   edited stream on the right (editable), so the user sees what changed
   before applying.
8. Stream longer than the score → append new measures (with only this part's
   line). Stream shorter → the unfilled rest of the last measure is filled
   with rests and later measures drop this part's line (see rules below).

---

## The stream format

- One part only, no `[Key]` prefix, no directive lines, no blank-line groups.
- Whitespace (including newlines) is insignificant. Extraction puts a newline
  after every 4 measures purely for readability and diff granularity.
- Content grammar = that part's normal line grammar (notes / chords /
  percussion / lyrics), plus:
  - `@` (follow parts only): one whole measure, unwritten (copies target).
  - `_` (lyric parts only): one whole measure with no lyrics (same meaning as
    a `_` lyric line today).
- Errors specific to the stream (all reported with spans **in stream
  coordinates**):
  - `|` anywhere.
  - Inline directive tokens (`bpm=N`, `1=C4`, `N/N`) — directives are hidden.
  - A timed cluster (head + its `-` extensions) that crosses a barline →
    "crosses the barline of measure N; split it with a tie (`~`)".
  - A tuplet `{…}` that crosses a barline.
  - `@` or `_` placeholder not at a measure boundary.
  - `@` in a non-follow part, `_` in a non-lyric part.
  - Lyric stream with more syllables than the remaining lyric slots in the
    score (lyric streams cannot create measures).
  - Any lex/parse error the normal line parser reports.

## Extraction rules (source → stream)

Per measure group `m`, take this part's **desugared** line (so follow-copies
and implicit fills are resolved) and produce a measure chunk:

| Situation | Chunk |
|---|---|
| Part written in `m` | The written content, whitespace-collapsed, with implicit trailing padding materialized as explicit ` -` tokens (only the padding case `can_implicitly_pad` allows: deficit % 4 == 0) |
| Non-follow notes/chords/percussion part not written | Rests that exactly fill the capacity: `capacity / 4` `0`s, then `0_` / `0=` for any remainder |
| Follow part not written | `@` |
| Lyric part not written | `_` |
| Lyric part, target has zero lyric slots in `m` | empty chunk (nothing emitted) |

The stream is the chunks joined with spaces (newline after every 4th measure).

**Eligibility (a part that fails any of these is listed in the modal but
disabled, with the reason shown):**

- Kind is eligible (see decision 2).
- No measure of this part has a beat overflow, an un-paddable underflow, or
  an implicit pad that lands on a tuplet member.
- No inline directive tokens in any of this part's lines.
- Lyric parts: no measure has a lyric tally mismatch (underflow/overflow
  warning), except `_` measures.
- The source parses (`# parts` and `# score` resolve).

## Split rules (stream → measure chunks)

Capacity of measure `m` = `beats_per_measure(num, den)` from the time
signature in effect at `m` (tracked across directive lines exactly like
`interleaved_parser::process_bar_group`). Measures past the end of the score
use the last capacity.

**Notes / chords / percussion:**

1. Parse the whole stream once with the part's line parser
   (`parse_notes_line` / `parse_chord_line` / `parse_percussion_line`, fresh
   `GroupStack`, `base_offset = 0`) and lex it once (`lex_line`) for
   punctuation token spans.
2. Rescale tuplet durations (`tuplet::resolution_multiplier_of` /
   `apply_resolution_multiplier`), scaling capacities by the same multiplier.
3. Walk timed clusters (head + following `Extension`s), accumulating
   duration. A cluster that starts exactly on a measure's capacity starts the
   next measure. A cluster that would cross a capacity → error.
4. Cut points: the start of the first event of the next measure, moved back
   over any immediately preceding `(` (`LParen`) and tuplet-open (`LBrace`)
   tokens. `)`, `}` and `~` stay with the measure on the left.
5. `@` (follow parts): must start exactly at a measure boundary; it fills
   that whole measure. Pre-scan the stream for `@` before handing the
   segments between them to the line parser.
6. If the stream ends partway through a measure, fill the remainder with
   rests (same rest spelling as extraction) — this is what makes deleting a
   note "just work".

**Lyric parts (target is notes or follow-of-notes):**

1. Per measure, the number of lyric slots = notes of the **target's
   desugared line** that aren't the continuation of a tie, with tie state
   carried across measures (mirror `lyric_slots::measure_lyric_slots`; a tie
   is `tie_to_next_span.is_some()`).
2. Tokenize the stream with `utils::tokenize_lyrics(stream, 0)` (`-` held
   markers are syllables in their own right and count toward slots).
3. Fill measures in order: a measure with 0 slots gets an empty chunk; a
   measure whose next token is `_` takes just that `_`; otherwise it takes
   exactly `slots` syllables, sliced from the stream by span.
4. Running out of syllables partway through a measure: that measure keeps
   what it has (it will show the existing underflow warning after Apply).
   Later measures get no chunk.
5. Syllables left over after the last measure with slots → error on the
   first leftover syllable.

## Apply rules (measure chunks → source)

Re-extract the **original** chunks from the current source (the TS side
holds no state) and compare chunk by chunk, whitespace-normalized:

| Original vs new chunk for measure `m` | Edit to the source |
|---|---|
| Equal | **Nothing.** The original line stays byte-for-byte, comments included. |
| Different, part written in `m` | Replace that line's content after `[Key] ` |
| Different, part not written in `m` | Insert `[Key] <chunk>` at its declaration-order position in the group (before the first data line of a later-declared part, otherwise after the group's last line) |
| New chunk is `@` (follow) / empty (beyond the end of the stream) | Remove this part's line in `m` — **unless** it is the group's only data line, in which case write the rest fill (removing it would leave a `measure_no_data_lines` group) |
| Beyond the end of a follow part's stream | Write explicit rests (removing the line would make it copy its target) |
| `m` beyond the last measure group | Append a new group `\n\n[Key] <chunk>` at the end of the `# score` section |

All edits are byte-range splices on the original source, applied from the
end backwards. Nothing outside this part's lines (plus appended groups) is
touched. Comments live in the source as blanked spans
(`section_splitter::strip_comments` keeps byte offsets), so replacing a line
replaces only its content range; keep a trailing comment when it sits after
the replaced range.

**Apply is refused** (no new source) while any stream **error** exists.
Warnings (e.g. a lyric measure left short) don't block Apply.

---

## Rust API (Phase 1–2)

New module `src/part_stream/` (`mod.rs`, `extract.rs`, `split.rs`,
`apply.rs`, `lyrics.rs`, tests in sibling `*_tests.rs` files — never inline).
Structs only, no tuples in new data structures. Prefer `itertools`/functional
style unless a loop is clearly simpler.

```rust
pub struct StreamablePart {
    pub abbreviation: String,
    pub display_name: String,
    pub kind: StreamablePartKind,          // Notes | Chords | Percussion | Follow | Lyrics
    pub ineligible_reason: Option<String>, // None = selectable
}
pub fn list_streamable_parts(source: &str) -> Vec<StreamablePart>;

pub struct PartStream {
    pub text: String,
    /// Byte offset in `text` where each measure's chunk starts (index = measure index).
    pub measure_starts: Vec<usize>,
}
pub fn extract_part_stream(source: &str, abbreviation: &str) -> Result<PartStream, PartStreamError>;

pub struct StreamDiagnostic { pub span: Span, pub message: String, pub severity: StreamDiagnosticSeverity }
pub struct PartStreamApplyResult {
    /// `None` while any error-severity diagnostic exists.
    pub source: Option<String>,
    pub diagnostics: Vec<StreamDiagnostic>,
    /// Where each measure starts in the *edited* stream (drives the measure markers in the UI).
    pub measure_starts: Vec<usize>,
    pub changed_measure_count: usize,
}
pub fn apply_part_stream(source: &str, abbreviation: &str, stream: &str) -> PartStreamApplyResult;
```

The web calls `apply_part_stream` on every keystroke in the modal (it's
pure; it just doesn't write anything until Apply) to show diagnostics,
measure markers and the changed-measure count, then uses `source` on Apply.

### Existing code to reuse (verified while planning)

- `parser::load_document_sections(source)` → `sections.parts` / `sections.score`
  as `(content, byte_offset)`; comments are blanked with spaces, offsets preserved.
- `parser::parts_parser::parse_parts(&content, offset, &[])` → `Vec<PartDecl>`
  (`abbreviation`, `display_name`, `kind: PartKind`, `follow_target`, …;
  `score_line_role()`).
- `parser::score::measure_group::{collect_groups, directive_line_count, is_directive_line}`.
- `desugar::{desugar_groups, parse_key_prefix, implicit_fill, SourceLine}`
  (`pub(crate)`; `SourceLine.is_implicit_fill` tells written vs filled).
- `parser::score::token_parser::{parse_notes_line, parse_chord_line, parse_percussion_line}`
  → `TimedLineParse { events: Vec<Spanned<ScoreEvent>>, chord_errors, lex_errors }`.
- `parser::score::timed_parser::{lex_line, LexContext, TimedLexToken}` —
  `LParen`, `RParen`, `LBrace`, `RBrace`, `Extension`, `HeadStart`, `Tilde`,
  `Repeat`, `Bpm`, `KeyChange`, `TimeSignature`. `|` already lexes as a
  recoverable "unexpected character" error (`timed_lexer.rs`).
- `ScoreEvent` (`src/ast/parsed/events.rs`): `Note`/`Chord`/`PercussionHit`/`Rest`
  carry `duration` (quarter-beats) and `tuplet`; `Extension { dotted, double_dotted }` = 4/6/7.
- `interleaved_beat_padding.rs`: `beats_per_measure`, `can_implicitly_pad`,
  `validate_and_pad_beats` (private to the parser — widen to `pub(crate)` or
  add a thin `pub(crate)` wrapper rather than copying the logic).
- `interleaved_parser.rs` / `interleaved_directives.rs`: how `time=` is tracked
  across groups (`split_directive`, private — add a `pub(crate)` wrapper).
- `lyric_slots::measure_lyric_slots`, `utils::tokenize_lyrics`.
- `format_source.rs`: example of splicing a rewritten `# score` back into the source.
- `source_edit/`: existing source-rewrite module patterns and test style.

## Wasm (Phase 3)

- WIT: `crates/poentaro-wasm/wit/world.wit` — add records/variants mirroring the
  Rust API and exports `list-streamable-parts`, `extract-part-stream`,
  `apply-part-stream`. Spans as UTF-8 byte offsets, like the existing `span` record.
- Guest impl: `crates/poentaro-wasm/src/component/guest_impl.rs` delegating to a
  helper next to `shift_part_octave` in `guest_metadata_and_misc.rs`.
- Tests next to the existing wasm tests (`crates/poentaro-wasm/src/tests_*.rs`).

## Web (Phase 4)

- Call wasm **on the main thread** (`await ensureWasmInit(); poentaroWasm().…`,
  see `web/src/renameSymbol.ts`) — no worker messages needed; the calls are
  pure string transforms.
- The web stays dumb: no splitting, diffing of measures or validation in TS.
- Entry: a button near the existing Edit Parts / Format actions opens a
  modal listing `list_streamable_parts` (disabled rows show the reason).
- Selecting a part shows the stream editor in the same modal:
  - `DiffEditor` from `@monaco-editor/react` (pattern:
    `web/src/components/mergeEditor/MergeEditor.tsx`), `original` = extracted
    stream (read-only), `modified` = editable copy, word wrap on, numbered
    language for highlighting if it applies cleanly to a bare stream.
  - On every change: `apply_part_stream` → Monaco markers on the modified
    editor from `diagnostics` (byte → UTF-16 offset conversion like the main
    editor does), a ghost marker (injected text, e.g. `𝄀5`) at each
    `measure_starts` entry, and "N measures changed" in the footer.
  - **Apply** (disabled while errors exist or nothing changed) replaces the
    main editor's source with `result.source` **through the editor model**
    (push an edit operation) so Ctrl+Z in the main editor undoes it.
  - Cancel / Esc closes without changes.
- e2e: Gherkin `.feature` + step defs (the user's preferred style). Write the
  failing scenarios first. Cover: shifting a note forward re-bars later
  measures; deleting a note fills the last measure with rests; `|` shows an
  error and disables Apply; a follow part shows `@` and round-trips with no
  changes; a lyric part re-splits by the target's notes; an ineligible
  chord-bound lyric part is disabled with a reason; Apply is undoable.
  Run scoped: `cd web && pnpm test:e2e --grep "..."`. Stop any dev server you start.

## Docs (Phase 5)

- `syntax.md`: a "Part stream editor" section (it's user-facing input): no
  barlines, `@`, `_`, crossing a barline needs a tie, directives hidden.
- `ARCHITECTURE.md`: the new module, its entry functions and key types, the
  wasm exports, and glossary terms **part stream** and **measure chunk**.

---

## Test checklist (Rust, Phases 1–2)

Round-trip property: for every eligible part of every file in `fixtures/`,
`scores/` and the repo-root `.poentaro` files, `apply_part_stream(source, p,
extract_part_stream(source, p).text)` returns `source` **unchanged**,
`changed_measure_count == 0`, and no errors.

Plus focused cases:

- Insert one note in measure 1 of 4/4 → every later note shifts; the stream
  overflows into a new appended measure.
- Delete one note → last measure filled with rests; later measures lose this part's line.
- Implicit padding (`[A] 1 2`) extracts as `1 2 - -`, applies back unchanged.
- Time signature change mid-score (4/4 → 3/4) splits at the right capacities.
- Triplet `3:{1_1_1_}` fills one beat; a tuplet crossing a barline is an error.
- Cluster crossing a barline (`1 2 3 4 - 5 6 7`) → error with the stream span of the cluster.
- Cross-measure slur `(` attaches to the next measure; `)` and `~` stay left.
- `|` → error; inline `bpm=90` → error.
- Unwritten measures round-trip as unwritten; a written rest line stays written.
- Group whose only data line is this part, emptied by a shorter stream → rest fill, not removal.
- Unchanged measures keep trailing `//` comments and original spacing.
- Follow part: `@` round-trips; `@` mid-measure is an error; shorter stream writes explicit rests.
- Lyrics: CJK and Latin; `-` held syllables count; tie continuation across a
  barline gets no syllable; `_` measure; zero-slot measures; leftover
  syllables → error; short last measure → warning only.
- Ineligibility: chord-bound lyric part, beat overflow measure, inline directive, lyric tally mismatch.

## Project rules to follow (from CLAUDE.md / memory)

- Commit titles `<type>(<scope>): <description>`; scope = where the change
  lives (e.g. `part-stream`, `poentaro-wasm`, `web`). Pass a 480000 ms timeout
  to `git commit` (the pre-commit hook runs cargo checks + web typecheck).
  Don't re-run the full test suite right before committing.
- In this worktree, run git as `/usr/bin/git`.
- Never commit to `master`. One PR at the end: `gh pr create`, merged later
  with `gh pr merge --squash`.
- No uncommon abbreviations in names; no tuples in new data structures; tests
  in separate files.
