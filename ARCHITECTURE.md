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

Each layer lists its module and entry function, then what a reader can't see from one file.

### Parser
- `src/parser/`, `parser::parse` → `ParsedDocument`
- Turns source into a raw event stream, without grouping notes into measures. Relative keys and implicit fills are resolved here, so later layers never see them.
- A document is read from `anyfix_expr`'s tree (`parser::anyfix::read`), except the `score` body, which stays one raw string read in place so every span is a span in the file. Text that doesn't fit an **Anyfix document** still opens with errors: `document::anyfix_errors` stops anything that writes a file, but a render keeps going so a typo doesn't blank the preview.

### Grouper
- `src/grouper/`, `grouper::group` → `Score`
- Groups notes into measures, pairs lyrics with **Lyric slots**, resolves `sequence` and warns on notes outside a **Range instrument**.
- `merge_duplicate_measures_across_parts` and `hide_resting_parts` are resolved on every measure rather than only where they change, since later layers need a plain value per measure.

### Compiler
- `src/compiler/`, `compiler::compile` → `CompileResult`
- Assigns each note and rest to a column, computes underlines and records slur and tuplet spans. A post-pass folds **Multi-measure rest**s, so anything walking blocks must advance by `MeasureBlock::represents_measures`, not by 1.
- `compiler::label_note_heads` then relabels note heads for the letter-name, harmonica and guitar views. Layout measures the resulting text, so nothing downstream needs to know which view is active.

### Consolidator
- `src/consolidator/`, `consolidator::consolidate` → `CompileResult`
- Merges rows with identical elements within one measure. It can't tell whether a match holds for the whole system, so it keeps the merged-away row in `MeasureRow::absorbed_rows` and leaves labels for grid layout to resolve.

### Grid Layout
- `src/grid_layout/`, `grid_layout::layout` → `LayoutOutput`
- Packs measures into **System**s and pages, sizes columns by **Rod and spring**, resolves slur and tuplet spans and computes click targets and highlights.
- A padded row has no note identity. A row restored from `absorbed_rows` keeps it, because those are the part's real notes.

### Coordinate Resolver
- `src/coordinate_resolver/`, `coordinate_resolver::resolve` → `Vec<AbsolutePage>`
- Gives every element absolute page coordinates. `PageFrame` owns the margin, so nothing here adds one by hand.
- Click targets resolve last, in a fixed order, because `elementFromPoint` hit-testing picks the topmost: a lyric target wins over the note target covering the same row.

### Renderer
- `src/renderer/`, `renderer::new_renderer::render_new` → `Vec<SvgDocument>`
- Produces SVG primitives for each absolute element, with `Tag` and `data-variant` values the web reads for hit-testing and hover.
- Text widths come from real font metrics. The wasm build receives fonts at runtime (`set_layout_font_bytes`), since embedding them would add about 16 MB.

### Serializer
- `src/serializer/`, `serializer::serialize` → `Vec<String>`
- Embeds the source in each SVG's `<metadata>`, so a lost file can be recovered from an export. PDF stores it in `/Info` instead, because `usvg` strips `<metadata>`.

### MEI export (staff notation)
- `src/mei_export/`, `mei_export::score_to_mei` → `MeiDocument`
- A side branch off the grouped `Score`: nothing in the numbered pipeline is reused past the grouper. The web engraves the MEI with Verovio, loaded lazily as its own chunk of about 8 MB, and hands SVG pages back to Rust for PDF export.
- Rust hands the web each measure's element id, so TS never builds or parses ids. TS measures Verovio's boxes, and Rust decides which measure owns a point (`crates/poentaro-wasm/src/staff_measures`).

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

### Cloud and sharing

| Term | Definition |
|------|-----------|
| **Synced Share worker** | The standalone Cloudflare Worker (`crates/live-share-worker`) behind cloud files, accounts and share links. |
| **Worker client** | The typed `openapi-fetch` client (`web/src/syncedShare/workerClient.ts`), generated from the routes' Rust signatures. |
| **ApiError** | The `code`-tagged union every worker route fails with; the variant decides the HTTP status. The web switches on `code`, never on raw statuses. |
| **Account** | An email and password on a `users` row, signing in to both cloud storage and share ownership. |
| **owner_user_id** | The internal `users.id` that owns a file or share. It replaced a bearer token that any first writer could claim. |
| **session token** | The random token handed to the browser and sent as `identityToken`. Only its SHA-256 hash is stored. |
| **resolve_verified_user_id** | The session lookup every authenticated route calls, instead of trusting an identity the client asserts. |
| **VerificationFailure** | Why a token was refused, returned inside `ApiError::Unauthorized`. It never carries the token or its hash. |
| **sign-in lockout** | A per-email lock after repeated wrong passwords, with or without an account behind it. It is separate from the per-IP rate limits. |
| **share** | A `shares` row pointing a `share_id` at one cloud file or folder path, not a copy of its content. The owner's autosave is the only write path, so a link can only ever show its own file. |
| **folder share** | A share of a folder path rather than file ids, so files moved in or out change what it shows. It is read-only for viewers. |
| **static folder link** | A `#share-folder=` link carrying a compressed snapshot of a folder. It works for local files and never updates. |
| **Shared folder viewer** | What someone opening a folder link sees: a read-only tree rooted at the folder, with an import button. |
| **share overview** | The owner's live links as `POST /shares/overview` reports them. Rust derives everything the share UI shows from it. |
| **shared via** | The innermost shared folder above a file or folder (`SharedVia`). |
| **selection unit** | One thing a multi-select action applies to: a file, or a folder whose files are all ticked. |
| **app log** | The append-only, 30-day `app_log` table recording what the worker did. Query it before guessing at production behaviour. |
| **trace ID** | The ID the worker gives each request, returned with every failure so a report can be looked up in `app_log`. |
| **flow** | `app_log.flow`: which part of the system a row is about. New flows are new `LogEvent` variants in the same table. |
| **subject** | `app_log.subject`: what a row is about, so rows of one account or session join across kinds. |
| **token tag** | The first 12 hex characters of a session token's hash, enough to tell tokens apart in `app_log`. |

### Cloud outbox

| Term | Definition |
|------|-----------|
| **Outbox** | The persisted `Queue` of not-yet-acknowledged cloud edits, plus the pure functions in `crates/cloud-outbox` that advance it. |
| **Lane** | One file's ordered queue of messages. Lanes drain independently, so one halted file never blocks another. |
| **Message** | One queued cloud operation with its recorded attempts. A `SaveContent` coalesces into a directly preceding one that isn't in flight. |
| **History** | Every message the outbox knows about: those still queued plus recently delivered ones. |
| **Looper** | The TS loop that sends what `begin_send` returns and feeds the answer to `on_result`. |
| **Halt** | A lane stopping instead of retrying (`NeedsSignIn`, `NeedsMerge` or `Failed`) until a `Resolution` moves it back. |
| **Base snapshot** | The last content known to match the server, used as the base of a three-way merge. |

## Web integration

The React app (`web/`) runs the compiler in a dedicated worker (`web/src/worker/poentaro.worker.ts`) backed by the `poentaro-wasm` crate. The main thread sends source text and asset bytes, and the worker posts structured results back.

Features below are implemented in Rust and exposed through the WASM boundary. The web passes raw UI state and renders what comes back.

### Storage abstraction

`StorageBackend` (`web/src/storage/types.ts`) has two implementations: `localBackend.ts` over `localStorage`, and `cloudBackend.ts` over the worker's `/files/*` routes. `useStorageBackend` picks between them.

- Structural operations apply locally at once and queue for the server through the **Outbox**, so the app keeps working offline.
- An `unauthorized` answer clears the stored session. Any other load failure retries with backoff, capped at 30 s.
- There is no OAuth: sign-in is a plain `POST` to the worker's `/auth/*` routes. Signing in never itself switches the backend or starts a share.

### File tree (path rules, source-level)

`src/file_tree/` is pure Rust and owns every file and folder rule. `web/src/fileStore.ts` calls straight into it through `crates/poentaro-wasm/src/component/guest_file_store.rs`.

- Store operations are pure functions over `FileStoreState`, returning a new state or a typed `FileStoreError`. Nothing is mutated on error.
- Rename and move are the same operation, and keep content and file id. A collision is an error, never auto-numbered, apart from duplicate and restore, which pick a unique name.
- `move_folder` is all-or-nothing and folders never merge. On the cloud it is one `rename-files` batch, so a half-delivered move can't exist.
- `FileTree::view_rows` and `selection.rs` decide what the file list and multi-select show. The web keeps only the expanded folders and ticked file paths.
- The worker knows only files, so an empty folder persists as a **Placeholder file**, and it knows nothing of folders beyond the share prefix.

### Source editing

`src/source_edit/` backs the toolbar and the Edit Parts, Edit Metadata and preset controls, so no TypeScript parses source.

- Edits are best effort: an unknown part or a parse failure returns the source unchanged.
- Selection-scoped edits return `RangeEditResult`, with the caller's ranges remapped into the new source so the selection survives.
- Edits go through `source_edit/anyfix_edit.rs`, which rewrites only the node it changes and formats only that section, so other sections keep their comments and layout.
- `update_metadata_field` re-emits the whole section in canonical order. `update_preset_field` rewrites one line, so the user's formatting stays.
- `shift_part_octave` rewrites written notation, unlike a part's MIDI-only `octave_offset`.

### Split-track export

`src/split_track.rs` compiles once, then renders one PDF, MIDI, WAV or MP3 per track, zipped into a single archive. A lyric part stays with the track it sings along to.

### Part declarations (source-level)

`list_part_declarations_from_source` (`src/part_info.rs`) returns raw fields from each `parts` line, before `follow [X]` inheritance, for the Edit Parts modal.

### Rename-symbol support (source-level)

`src/symbols.rs` is built on spans `ParsedDocument` already tracks, with no separate parse pass. `rename_edits` only returns edits, which callers apply.

### Keyword highlighting (source-level)

`src/highlight.rs` takes every token from the parser's own reading, so editor colors and CodeLens positions can't drift from the grammar. Monaco's Monarch tokenizer keeps only vocabulary-free rules (`web/src/monacoPoentaroLanguage.ts`).

### Score formatting (source-level)

`format_score` (`src/format_source.rs`) drops `[Key]` lines made redundant by implicit fill, sorts the rest into `parts` order and collapses whitespace. A key's droppable lines must be a trailing run, since dropping an earlier lyric line would shift a later one into the wrong slot. `follow [X]` lines are never dropped, because a rest there is real content.

### Part stream editing (source-level)

`src/part_stream/` edits one part as a **Part stream**, called from the main thread by `web/src/components/partFocus/`.

- `apply_part_stream` is pure and runs on every keystroke for diagnostics. It rewrites only the measures whose chunk changed.
- A part is eligible only if its stream splits back into exactly the same chunks, so applying an untouched stream never changes the source. The exception is a misbarred measure, which applying re-bars.
- `format_part_stream` re-bars by changing whitespace only, and applies without an undo stop, so a keystroke and its re-barring undo together.

### Pitch description (source-level)

`pitch_description::describe_selection` (`src/pitch_description/`) returns a description only when the selection is exactly one note or chord, so the web drawer never decides whether to open. Spelling goes by letter from the measure's key tonic. Guitar and piano diagrams are rendered in Rust as SVG, using `currentColor` so the page theme colors them.

### WASM boundary mechanism (`crates/poentaro-wasm`)

`crates/poentaro-wasm/wit/world.wit` is the single source of truth for every export and its types, so it is also the list of exports. `wit-bindgen` and `jco` generate Rust and TS types from it, which makes a shape mismatch a compile error. `web/package.json`'s `build:wasm-component` is the only wasm build path.

- `component/` holds the one `Guest` impl, and is the only place that names both WIT types and the crate's internal types.
- `web/src/poentaroWasm.ts` re-exports the generated types with no reshaping layer, so a field renamed in `world.wit` is a type error at every consumer.
- WIT record names must not equal `<variant>-<case>` of any variant: jco would generate two declarations that TypeScript merges into an unconstructible type (`poentaroWasmDts.test.ts` guards this).
- Each `Tag` field's `data-*` attribute name lives only in `web/src/dataAttributes.ts`. CSS and Playwright steps use its selector builders and never hand-type a `[data-tag=…]` selector.

## Synced Share worker (`crates/live-share-worker`)

A standalone Cloudflare Worker, separate from the `web/` build and the wasm component, backed by D1. Migrations live in `live-share-worker/migrations/`. `.github/workflows/live-share-worker.yml` applies them and deploys on pushes to `master`. A daily cron prunes `app_log` and expired `origin_migrations`.

### Route list and generated client

`handlers::routes()` is the one list of routes. Each registration feeds both the router and an OpenAPI spec derived from the handler's signature. `web`'s `build:worker-types` generates TS types from it, along with the share id format and deploy config, so no path, body shape or error literal is restated in TS. These files are gitignored and can't go stale.

`POST /e2e/reset` exists only when `SYNCED_SHARE_E2E_RESET_EMAILS` is set, which only the local Playwright config does. Otherwise it answers like any unknown path.

### Shares

- A **share** points at a file rather than copying it. An earlier design kept a copy, and a stale-render race could write one file's content to another file's link.
- The public response carries nothing about the owner: no file id, `owner_user_id` or token (`protocol::SyncedDoc`).
- Owner routes are owner-scoped in SQL and answer `not_found` otherwise. Starting is idempotent, so re-sharing from any device gives the same link. `share_id`s are generated on the server.
- A stopped share, or one whose file is in the bin, reads as ended and returns no content.
- A **folder share** matches active files by whole path segments at read time, so `Hymns` never covers `Hymns 2/...`, and `%` and `_` are literal. Renaming a folder re-points its link when the batch is delivered.
- File names are full slash paths, and `files::validate_file_name` runs the same `TreePath` rules as the web.

### Accounts (email + password)

One account proves identity for both cloud files and share ownership. Sign-up is open, with no email verification or sending: the admin resets a forgotten password with `queries/admin/link_credentials_to_user.sql`. The rules live in `src/auth/`, D1-free and tested. The frontend holds none.

- **Passwords:** PBKDF2-HMAC-SHA256 through WebCrypto at 100 000 iterations (the Workers cap), stored self-describing so the count can change later. Minimum 12 characters.
- **No account enumeration:** an unknown email and a wrong password get the same answer, cost the same hashing and count toward the same lockout.
- **Sessions:** only the token's SHA-256 hash is stored. A session lasts 60 days from last use, and the expiry only moves once less than 59 days remain, so most requests don't write. A D1 failure is a retryable `internal` and keeps the token.
- **Changing a password** deletes the account's other sessions, but not the caller's.
- **Lockout:** 5 wrong passwords in a row lock the email for 15 min × 2^level. While locked, the password isn't checked.
- **Rate limits:** Workers bindings keyed on `CF-Connecting-IP`, 20 sign-ins and 3 sign-ups per 60 s.

### Version history

`files.content` stays the head, and each save that changes the text adds a reverse patch to `file_versions`, keeping the newest 100. Recording is best effort, so an error never fails the save. `reachable_versions` stops at the first gap in the chain. A restore puts the text in the editor and saves normally, so it adds a version and never rewrites history. The logic is pure and property-tested in `src/file_history.rs`.

### Origin migration

The app moved from `jianpu-generator.pages.dev` to `poentaro.pages.dev`. Browsers keep `localStorage` per origin, so a plain redirect would strand every local file and sign the user out. The old origin instead serves a bridge page (`web/bridge/`, `web/src/bridge/`) that uploads saved data to the worker and redirects with `?migrate=<id>`, keeping the path, query and fragment that share links rely on.

- The handoff is anonymous and rate limited, and its random id is the only credential. Claiming deletes the row as it reads it, so a handoff is read at most once. It expires after 1 h, and the worker never parses the payload.
- On arrival, `prepareStorage` runs before the app renders. A carried entry is saved only where the new origin has none. Carried files merge under a top-level folder (`merge_migrated_file_store`), and a store the merge rejects is kept rather than lost.
- The old origin's data is never deleted. `.github/workflows/pages.yml` deploys the bridge to the old project, once the app has deployed.
- The old GitHub Pages site only redirects, from a separate repo, and carries no saved files.

### Rename to Poentaro

The app was first called jianpu-generator, and every name meaning the app was renamed. The notation once called jianpu is now numbered notation.

What still says `jianpu`, on purpose:

- **Cloud resources**, which can't be renamed without recreating them. See "Naming" in `AGENTS.md`.
- **Legacy names read for old data**, each next to its current name:
  - The extension: `file-extension.json`, read by both Rust and TS.
  - Browser storage: `web/src/legacyStorage/`. `renameLegacyStorage` moves `jianpu:` keys last, so an interrupted rename finishes on the next visit.
  - File names: `file_tree::rename_legacy_extensions` locally and D1 migration `0011_poentaro_file_extension.sql` in the cloud. A taken name keeps its legacy one.
  - Embedded source: `source_embed` also reads the old SVG and PDF tags.
  - Preset notation: `notation: jianpu` reads as `Notation::Numbered`.

### App log (observability)

An append-only `app_log` table records what the worker did, so production behaviour can be answered from data. Query it with the saved SQL in `crates/live-share-worker/queries/observability/`; see "Debugging production" in `AGENTS.md`.

- **Rows:** typed by `app_log::LogEvent` (`src/app_log.rs`). `Outcome` is derived from the variant, never chosen at a call site.
- **Schema evolution:** rows must keep decoding for their 30 days, so only add fields, each with `#[serde(default)]`. A changed shape is a new variant.
- **Writes:** best effort, so a failed insert never fails the request. No password, raw token or full hash is ever stored, only a **token tag**.
- **Failures:** every non-2xx answer goes through `handlers::fail`, which logs one `request_failed` row under the request's **trace ID** and returns that ID. The web shows it with the error.
- **Retention:** the daily cron deletes rows older than 30 days, in bounded batches. Locally, `GET /cdn-cgi/local/scheduled` fires it under `cf dev`.

### Backups

`.github/workflows/d1-backup.yml` exports production daily to a private Backblaze B2 bucket, as `daily/` and, on the 1st of each month, `monthly/`. Lifecycle rules (`backup-lifecycle.json`) keep them for 30 and 365 days.

- **Why B2:** R2 needs a payment method on the Cloudflare account, and B2 keeps backups outside the account they protect. GitHub artifacts can be downloaded by any signed-in user of this public repo, and KV or a second D1 database would sit in the same account.
- **Keys:** the workflow's key can only list buckets and write files, so a leaked secret can't read or delete. The read-only key is in each laptop's Keychain. Only the master key, in the owner's password manager, can delete backups or change the rules. The worker has no access to the bucket.
- **Silent failure:** a failed run only emails the owner, and GitHub turns off scheduled workflows after 60 days without commits. To check a backup ran, see the latest Actions run or list the bucket: `uvx b2==5.0.0 ls b2://jianpu-live-share-backups/daily/`.
- **Restoring:** within 7 days, D1 Time Travel restores production to any minute. Otherwise `just restore-backup <date>` loads a dump locally. To restore production, download and gunzip it, run it with `wrangler d1 execute <database> --remote --file` into a new, empty database, then point `d1DatabaseId` at that database.

### D1 query checking

`sqlx`'s SQLite build doesn't target wasm, so it appears only as a build dependency, applying migrations to a local shadow database, and as a dev dependency. `tests/query_syntax.rs` prepares every file in `queries/` against that database. It catches unknown tables, unknown columns and invalid SQL, but not parameter types, result column names or D1-versus-SQLite differences.

## Cloud outbox (`crates/cloud-outbox`)

The pure state machine deciding what a signed-in user's cloud edits do next: what to send, when to retry and when to stop and ask. Every function takes a `Queue` and returns a new one, and the crate never reads a clock, a random number or the network: `now_ms` and `jitter_unit` are passed in. It is exposed through the `outbox-*` WIT exports.

- **Lanes:** one per file, so a halted file never blocks another. A folder move sits alone on a synthetic `batch:<message_id>` lane, so a half-delivered move can't exist.
- **Ordering:** a batch waits for pending creates of its files and for earlier operations on them, and later operations on a member wait for the batch (`scheduling::is_blocked`). `SaveContent` is never blocked.
- **Conflicts:** a revision conflict halts the lane as `NeedsMerge`. A clean three-way merge saves at once, and a conflicting one opens the merge editor.
- **Name collisions:** the worker answers `name_taken`, or `names_taken` for a batch. The outbox rewrites the names and resends.
- **Persistence:** versioned by `QUEUE_SCHEMA_VERSION`. Delivered messages are kept separately under `sent:` keys for at most 7 days, 200 entries or 5 MB, always keeping the newest.

### TypeScript shell (`web/src/storage/outbox/`)

The shell only does I/O, timers and rendering, and every decision is a wasm call. `outboxLooper.ts` runs on the leader tab only, under `outboxLock.ts`. The UI lives in `web/src/components/syncPanel/`, `outboxHistory/` and `mergeEditor/`.
