# TODO: unenforced cross-boundary invariants (Rust ↔ TS)

Each item below is a "simple" invariant (finite tags, field names, fixed shapes) that a
type system could enforce but currently doesn't — a typo/rename compiles clean on both
`cargo build` and `tsc` and only breaks at runtime. See the "Cross-boundary invariants
(Rust ↔ TS)" section in `AGENTS.md` for the general principle (tagged unions / make
illegal states unrepresentable) to apply when fixing these.

Clear them one at a time — each is independent and can be its own commit.

---

## [x] 1. GM instrument/percussion `"N: Name"` labels (highest risk)

The leading GM program number is embedded in a free-text label string and re-parsed by
splitting on `:` — the string is the *only* channel carrying the number, on both sides,
in three independent hand-written places.

- Rust parses it back out: `src/part_info.rs:32-43` (`instrument_program_to_label`,
  `instrument.value.split(':').next().and_then(|prefix| prefix.trim().parse::<u8>().ok())`).
  `InstrumentInfo` has no separate numeric field.
- Rust also *builds* the percussion version independently: `src/gm_percussion.rs:21`
  (`format!("{}: {}", entry.key, entry.name)`).
- TS hand-authors all 128 GM instrument entries in this exact shape:
  `web/src/utils/gmInstruments.ts:41+`.
- TS re-implements the same `split(':')` parsing independently, twice:
  `web/src/components/SoundfontSearchModal.tsx:87` and `:202`.

**Failure mode:** a typo'd/duplicated/missing leading number in any of the 128 entries
compiles fine everywhere; the instrument silently resolves to the wrong (or "Unknown")
GM program — wrong sound plays, and/or the wrong label round-trips through `# parts`.

**Direction:** give `InstrumentInfo` (and the percussion equivalent) a real `program: u8`
field instead of deriving it from the label text; expose it to TS as a proper field
instead of a string to re-split.

---

## [x] 2. `TransparentRectRoleOut` → `data-variant` kebab-case strings

Same shape as the `data-tag`/`TagOut` issue below (item 6), but for `data-variant` — and
with more call sites. (This item's own "already-fixed `data-tag` issue" aside was stale:
at the time this item was fixed, `data-tag` was *not* actually centralized anywhere —
`TagOut` was still hand-typed as `data-tag` strings independently in `PreviewSvgRenderer.tsx`
and ~7 other TS files, exactly the shape `AGENTS.md`'s "Cross-boundary invariants" section
names as its running example. See item 6 for the actual fix.)

- Source: `TransparentRectRoleOut` enum, `crates/poentaro-wasm/src/svg_types.rs:20-30`.
- Hand-mapped to kebab-case strings in `web/src/components/PreviewSvgRenderer.tsx:45+`
  (`transparentRectRoleToDataVariant`), plus a standalone literal
  `data-variant="playback-cursor-rect"` at line 308 for `PlaybackCursorRect`.
- Re-typed as raw strings independently in:
  - `web/src/preview.css:220,227,234,241,248`
  - `web/src/index.css:52,57,61-62,66-67`
  - `web/src/components/usePlaybackCursor.ts:50,92,104`
  - `web/src/components/previewRangeHighlights.ts:107,121`
  - `web/src/components/previewLabelRangeHighlights.ts:38,100,154,189,243,276,331`

**Failure mode:** a mismatch anywhere compiles fine; the CSS rule or `querySelector`
silently matches nothing — hover states, drag highlights, playback-cursor lookup quietly
stop working for that element, with zero error anywhere.

**Direction:** one shared TS constant/type for the 9 `data-variant` values (e.g. a
`const DATA_VARIANT = {...} as const` derived list), with the switch and every CSS/TS
consumer referencing it instead of re-typing the literal. (Note: `preview.css`/`index.css`
are plain CSS, so they can't reference a TS constant directly — consider a codegen step,
a shared list with a test that asserts the CSS file contains each value, or moving those
selectors to be generated/injected rather than hand-written.)

---

## [x] 3. `PartMode` wasm string protocol (`"chords"`/`"notes"`/`"percussion"`/`"follow[...]"`)

Two independent hand-written parsers for the same concept, not derived from the serde
tags that already exist for it.

- Rust hand-parses these literal strings: `src/source_edit/mod.rs:12-23` (`PartMode::parse`).
- This is entirely independent of `PartDeclarationModeOut`'s existing serde tags
  (`crates/poentaro-wasm/src/types.rs:74`), consumed as TS's `PartMode`
  (`web/src/types.ts:16`).
- TS reconstructs the raw strings to send across the boundary in
  `web/src/worker/poentaro.worker.ts:114-118` (`modeToWasmString`).

**Failure mode:** typed on the TS input (`mode: PartMode`) but not on the Rust parser
side — if the two string sets drift, `PartMode::parse` silently returns `None` and
`update_part_declaration_source` (`src/source_edit/part_declarations.rs:93-95`) falls
through to `return source.to_owned()`: the "Edit Parts" UI's mode change is silently
dropped with no error shown to the user.

**Direction:** have `PartMode::parse` and `PartDeclarationModeOut` derive from (or be
tested against) the same tag list, or route the wasm call through the existing
`PartDeclarationModeOut` tagged union instead of a hand-built string.

---

## [x] 4. `TextStyle` field/kind names mirrored as bare string lists

- Rust matches field-name strings in `src/parser/text_style_parser.rs:33,40,47,54,61,68,75`
  (`"font_size"`, `"horizontal_padding_pt"`, `"vertical_padding_pt"`, `"bold"`,
  `"italic"`, `"underline"`, `"font_family"`) and kind-name strings in
  `src/parser/metadata_parser.rs:161-169` (`"part_legend"`, `"measure_number"`,
  `"section_label"`, `"page_number"`, `"part_label"`, `"note_dash"`, plus
  `title`/`subtitle`/`author`/etc.).
- TS mirrors both lists as untyped `as const` string arrays:
  `web/src/utils/textStyleFields.ts:5-19` (`textStyleKinds`) and `:25-53`
  (`textStyleNumericComponents`, `textStyleBooleanComponents`, `fontFamilyValues`).

**Failure mode:** a name typo'd differently between the two lists compiles cleanly on
both sides; the Rust parser silently treats the corresponding source line's field as
unrecognized (ignored, or a generic diagnostic rather than a specific one), or the TS
editor UI offers a field the Rust parser never recognizes, so edits to it are silent
no-ops on the next round-trip.

**Direction:** expose the Rust-side kind/field name lists to TS (wasm-exported constant
or generated from the same source) instead of a hand-copied array, or add a test that
fails when the two lists diverge.

---

## [x] 5. Monaco directive-keyword regex vs. Rust directive lexer (low/cosmetic)

**Fixed** in `3abeb0a`: the wasm `highlight-tokens` export reports keywords from the
real Rust lexers, which drive Monaco semantic tokens. Monarch keeps only
vocabulary-free rules. (The Edit Parts/Metadata CodeLens regex is a separate
leftover — see item 14.)

- Rust lexes directive keys structurally, e.g. `src/parser/score/timed_parser/timed_lexer.rs:174-175`
  plus scattered handling for `key=`, `time=`, `label=`,
  `merge_duplicate_measures_across_parts=`, `hide_resting_parts=` across the
  timed/interleaved directive parsers.
- TS hardcodes the same keyword set for editor syntax coloring only:
  `web/src/monacoPoentaroLanguage.ts:39`
  (`/\b(bpm|key|time|label|merge_duplicate_measures_across_parts|hide_resting_parts)(?=\s*=)/`).

**Failure mode:** cosmetic only — Rust never consults this regex, so a drift just means a
valid/new directive keyword fails to get syntax-highlighted (or an invalid one gets
highlighted as if valid). Lowest priority; fix opportunistically.

**Direction:** lowest priority given no functional impact — consider only if touching
this file anyway; a shared keyword list (wasm-exported or generated) would still be the
correct fix in principle.

---

## [x] 6. `TagOut` → `data-tag` kebab-case strings (the running example in `AGENTS.md`)

- Source: `TagOut` enum, `crates/poentaro-wasm/src/svg_types.rs`.
- Was hand-mapped to kebab-case strings in `web/src/components/PreviewSvgRenderer.tsx`'s
  `groupAttrsForTag`, and re-typed independently as raw `data-tag="..."` string literals
  in `web/src/components/clickableElementId.ts` (its own hand-typed
  `switch (dataset.tag)`), plus hand-formatted `` `[data-tag="..."][data-...="${x}"]` ``
  selector strings and hand-parsed `dataset.partIndex`/`Number(...)` fields duplicated
  across `usePlaybackCursor.ts`, `previewRangeHighlights.ts`,
  `previewLabelRangeHighlights.ts`, `previewLabelSelection.ts`, and `Preview.tsx` — 52
  occurrences total, the largest surface of any item in this file.

**Failure mode:** a typo'd/renamed `TagOut` variant compiles clean everywhere; the
corresponding `querySelector`/`closest`/`dataset` read silently matches nothing or
returns `undefined` — click handling, playback cursor, drag-range selection, and
scroll-to-measure quietly stop working for that element, with zero error anywhere.

**Fix:** `web/src/dataAttributes.ts` (renamed from `dataVariant.ts`, which already held
the equivalent `DATA_VARIANT`/item 2 fix) now also holds `DATA_TAG`
(`satisfies Record<TagOut['type'], string>`), the writer (`groupAttrsForTag`) and reader
(`tagFromElement`) for a group's `data-*` attributes, and typed selector-builders
(`groupTagSelector`/`measureGroupSelector`/`measureGroupByEndSelector`/
`noteGroupSelector`) — every consumer above now calls into these instead of re-deriving
the tag ↔ attribute mapping itself.

**Follow-ups intentionally left out of that fix's scope:**

- `SvgElementOut.variant: Option<String>` on text elements is a **separate**,
  already-raw Rust string, unrelated to `TransparentRectRoleOut`/`DATA_VARIANT` — a
  styling hook, not an identity concern. Not covered by `DATA_VARIANT` or this item;
  possible future item if it ever needs the same treatment.
- `ClickableElementId` (Rust `selection_range_types.rs`, hand-mirrored in
  `clickableElementId.ts`) is a separate parallel enum used for the *input* direction
  (TS → Rust, `resolve_selection_range` args) and uses different field-name casing
  (`sourcePartIndex`) than `TagOut` (`source_part_index`). Unifying the two Rust-side
  enums so there's only one vocabulary crossing the boundary in both directions would be
  a larger, separate Rust-side change. `clickableElementIdFromElement` was updated to
  derive from `tagFromElement` instead of re-parsing `dataset` independently (removing
  the TS-side duplication), but the two-enum split itself remains.

---

## [x] 7. Synced Share worker HTTP routes, bodies and error statuses

ts-rs only generated the worker's body *shapes*. Everything else was restated
by hand in TS: route paths (`handlers/mod.rs` vs. four separate `fetch` call
sites and e2e's `page.route` globs), which request type went with which path,
`as Wire.X` casts on every response, the OAuth routes' bodies (never exported
at all), `401`/`404`/`409` status numbers, and the untyped
`{code: "name_taken"}` literal.

**Fix:** `handlers::routes()` registers each route into both the worker
`Router` and an OpenAPI spec, deriving path params, request body, success
response and the `ApiError` failure union from the handler's own signature;
`web/` generates `schema.ts` from that spec (openapi-typescript) and calls
the worker only through `syncedShare/workerClient.ts` (openapi-fetch). See
ARCHITECTURE.md's "Route list and generated client".

---

# Audit 2026-09-28: remaining hand-kept invariants

These came from a read-only audit covering every `world.wit` export, the Rust diagnostic
text, Rust-produced formats, duplicated constants, the worker ↔ D1 ↔ web path, and
`web/e2e` + `web/scripts`. The items are ordered by severity. The areas found clean are
listed at the end.

---

## [x] 8. Part index → part name remapping re-derived in TS (silent functional)

- Rust drops hidden parts, then re-numbers the rest: `src/filters.rs:6-17`
  (`apply_track_filter`), applied before indices are assigned in
  `src/note_spans.rs:64-65` (lyric spans likewise). Part order comes from `doc.tracks`
  (`src/grouper/mod.rs:44`), but `list-parts` returns `doc.declarations` order
  (`src/part_info.rs:124`).
- TS re-implements that compaction to map `sourcePartIndex` → abbreviation:
  `web/src/hooks/useNoteSelection.ts:105-110`,
  `web/src/hooks/useMeasureRangeSelection.ts:177-182`. A third, unused copy is in
  `web/src/utils/noteTimingsPartIndex.ts`.

**Failure mode:** suppose the filter semantics change (e.g. hidden parts keep their
index) or declaration order stops matching track order. Then "play selection" mutes or
solos the wrong parts. The `?.abbreviation` + `.filter` drops unknown indices, so
nothing errors.

**Direction:** have note/lyric spans (or the selection runs) carry the part
abbreviation, so TS never maps index → name itself. Delete
`noteTimingsPartIndex.ts`.

**Fix:** Rust resolves each part's name from `PartRow::name` in the same loop that
assigns the compacted index, after `apply_track_filter` has run. It carries the name as
`part-abbreviation` on `note-span`, `lyric-span`, `note-selection-run` and
`lyric-selection-run` (WIT). `useNoteSelection` and `useMeasureRangeSelection` read
`partAbbreviation` off the runs through `distinctPartAbbreviations`. They no longer take
`parts`/`enabledTracks`. The "selection touches every visible part" check counts the
distinct abbreviations in the already-filtered `noteSpans`. `noteTimingsPartIndex.ts`
is deleted. `tests/main/span_part_abbreviation.rs` covers a hidden middle part.

---

## [x] 9. Share-id length/charset triplicated (silent functional)

- Rust: `crates/live-share-worker/src/share_id.rs:28` (`SHARE_ID_LENGTH = 11`), `:36`
  (`ID_CHARSET`).
- TS: `web/src/syncedShareUrl.ts:22-23`, `web/functions/index.ts:20-21`. They are linked
  only by "must match" comments.

**Failure mode:** new share links stop parsing (`parseSyncedShareFromHash` returns
`null`), so the viewer silently opens the plain app. Link previews also fall back to
the generic title.

**Direction:** put a `pattern` on the `share_id` path param in the OpenAPI spec (or
export a generated constant), and read it on the TS side.

**Fix:** `share_id::share_id_format()` builds `{length, pattern}` from `SHARE_ID_LENGTH`
and `ID_CHARSET`. The OpenAPI spec's `share_id` path param gets it as its schema
(`pattern`, `minLength`/`maxLength`). `tests/export_openapi.rs` also writes it to the
gitignored `web/src/generated/live-share-worker/shareIdFormat.json`, next to
`openapi.json`, via `build:worker-types`. `syncedShareUrl.ts` and the Pages Function both
import that JSON, so neither has its own length or charset. A missing file fails
`tsc -b`.

---

## [x] 10. `set-layout-fonts` takes three positional `list<u8>`s; TS picks the role for each (silent layout)

- WIT `world.wit:829`: `(directive-line-font, lyric-font, monospace-font)`, all
  `list<u8>` (`crates/poentaro-wasm/src/component/guest_metadata_and_misc.rs:45-53`).
  Which role each text kind renders in is decided in
  `src/serializer/text.rs:70,81`.
- TS: `web/src/worker/poentaro.worker.ts:66-73` calls
  `setLayoutFonts(fonts.tc, fonts.sc, fonts.mono)`. Here `tc` is the sansSerif role and
  `sc` is the serif role (`web/src/hooks/useFontsLoader.ts:25-27`). The mapping is kept
  only by a comment.
- The `generate-pdf`/`generate-split-pdfs` WIT param names `sans-serif-sc`/`sans-serif-tc`
  are stale: `sc` actually carries the serif font (`src/pdf.rs:146`). The PDF itself is
  unaffected, because fontdb loads all three fonts and resolves them by name
  (`src/pdf.rs:50-54`).

**Failure mode:** suppose Rust changes which role lyrics or directive lines use. The
preview then measures glyph widths with the wrong font, so spacing is slightly off or
text overlaps, and nothing errors.

**Direction:** take a record keyed by the `font-family` role (serif/sans-serif/monospace),
and let Rust decide which role measures which text. Rename the PDF params to match.

**Fix:** WIT `record font-bytes-by-family { serif, sans-serif, monospace }` uses the
`font-family` role names, which are also the `fonts/fonts.json` keys. It is the only font
parameter of `set-layout-fonts`, `generate-pdf` and `generate-split-pdfs`. The stale
`sans-serif-sc`/`sans-serif-tc` params are gone. On the Rust side,
`fonts::FontBytesByFamily` replaces `PdfFonts`, and `font_source` stores one face per
`FontFamily`, reached only through `face_for_family`. The "directive line font" and
"lyric font" concepts are deleted: each measuring function asks for the role its text
renders in. `useFontsLoader` and the worker pass the jco-typed `FontBytesByFamily`
through unchanged, so `sc`/`tc`/`mono` and their role comments are gone.

---

## [x] 11. `"N: Name"` soundfont label still carried as a string on the Edit Parts path

Item 1 added `program` to `instrument-info`. The settings round-trip is still string-based:

- TS builds the label: `web/src/utils/gmInstruments.ts:204`,
  `web/src/utils/gmPercussion.ts:12`.
- Rust builds it too: `src/gm_percussion.rs:21`, and `src/part_info.rs:37` for the
  `"{program}: Unknown"` fallback.
- Rust parses it back: `src/parser/parts_parser/instrument_matching.rs:84`
  (`find(": ")` + `u8` parse).
- `part-settings.soundfont` is `option<string>` (`world.wit:325`), and
  `web/src/components/SoundfontSearchModal.tsx:206,221` compares strings to highlight the
  current sound.

**Failure mode:**
- If the TS format drifts, a picked sound is written to the source in an unparseable
  form. Rust then shows a diagnostic and falls back to program 52 (Choir Aahs).
- If the percussion format drifts on either side, the modal stops highlighting the
  current sound (cosmetic).

**Direction:** carry `program: u8` in `part-settings`, let Rust format the source text,
and have TS look up entries by number.

**Fix:** WIT `part-settings.program: option<u8>` replaces `soundfont: option<string>`.
`update-part-declaration` takes `raw-instruments` and formats the quoted text itself.
`src/sound_label.rs` is now the only place that formats or parses `"N: Name"` (instrument
name, GM percussion name, or `"N: Unknown"`). `instrument-info.value` became the bare
`name`. The picker gets its labels from the new `list-sound-choices` export, and the
current sound's label from `part-declaration.sound-label` (display-only). It compares by
program number. `GM_PERCUSSION`, `SoundfontValue` and `web/src/data/gmPercussion.json`
are deleted: the JSON moved to `src/gm_percussion.json`, since only Rust reads it now.

---

## [x] 12. Link-preview Pages function bypasses the OpenAPI client (cosmetic)

- `web/functions/index.ts:29-32,45-52` does a raw `fetch` of `/shares/${shareId}` with a
  hand-typed `SyncedDocSummary {filename, ended}` and an untyped `.json()`.
- The Rust side is `SyncedDoc` (generated `web/src/generated/live-share-worker/schema.ts:770`).

**Failure mode:** crawler previews silently show the generic "簡譜" title.

**Direction:** `import type { components }` from the generated schema (type-only, adds
nothing to the bundle), or use openapi-fetch.

**Fix:** the function builds its own `openapi-fetch` client, `createClient<paths>`, with
`paths` imported type-only from the generated schema. The route, the `share_id` path
param and the `SyncedDoc` response body now all come from the spec, and the hand-typed
`SyncedDocSummary` is deleted. The function does not reuse `workerClient.ts`, which
would pull SPA code into the edge bundle. `SHARE_QUERY_PARAM` now lives only in the
dependency-free `web/src/shareQueryParam.ts`, which both `syncedShareUrl.ts` and the
function import. The worker host is still hard-coded; item 16 covers it.

---

## [x] 13. `web/scripts/reset-e2e-cloud-db.ts` runs raw SQL against the worker's D1 schema (silent, test infra)

- Lines 45-54 hard-code `shares.file_id`, `files.owner_user_id`,
  `user_identities(user_id, provider, provider_user_id)` and the `'github'` literal.
  The Rust side is `GITHUB_PROVIDER` in `crates/live-share-worker/src/identity/github.rs:19`,
  and the worker's own queries are in `crates/live-share-worker/queries/*.sql`.
- The `catch` at lines 71-79 treats every error as "no local D1 yet".

**Failure mode:** after a schema rename, the reset silently does nothing. Rows pile up
across local runs, and "source 2"-style name assertions start failing in a way that
looks like flakiness.

**Direction:** move the reset next to the worker's checked queries (or into a test-only
route). At minimum, only swallow "no such table".

**Fix:** the reset is now the worker's test-only `POST /e2e/reset` route
(`crates/live-share-worker/src/e2e_reset.rs`, `src/handlers/e2e_reset.rs`). Its two
deletes are the checked queries `queries/delete_{shares,files}_of_provider_users.sql`,
covered by `tests/query_syntax.rs` and bound with `GITHUB_PROVIDER`. The route is only
registered when the var `SYNCED_SHARE_E2E_RESET_GITHUB_USER_IDS` is set. The real
`wrangler.toml` never sets it, so without it the path answers like any unknown route.
Its value is the allowlist of GitHub ids, which `playwright.config.ts` builds from
`KNOWN_GITHUB_USER_IDS`. The var name reaches TS through the generated
`e2eResetConfig.json`. `web/e2e/global-setup.ts` calls the route through the typed
`openapi-fetch` client, after Playwright's `webServer` is up, and any failure aborts the
run. `reset-e2e-cloud-db.ts` is deleted, so TS no longer names any table, column or
provider.

---

## [x] 14. Edit Parts / Edit Metadata CodeLens uses its own section-header regex (minor)

- TS: `web/src/components/Editor.tsx:230,239` trims the line, then tests
  `/^#\s*parts$/` and `/^#\s*metadata$/`.
- Rust: `section_header_kind` (`src/parser/section_splitter.rs:58-59`) requires `#` at
  column 0 and runs after `//` comments are stripped.

**Failure mode (already diverges):**
- `  # parts` gets a lens that Rust ignores.
- `# parts // note` is a real section in Rust but gets no lens.

**Direction:** drive the lenses from the `section-header` spans that `highlight-tokens`
already returns. This may need the section kind added to the token.

**Fix:** a new WIT export, `section-headers(source) -> list<section-header>`, returns
`{kind: section-kind, span}` (Rust `highlight::section_headers`). It is built from the
same `split_sections` result as the `section-header` highlight tokens, so it only
reports headers the parser recognizes. This is a separate export, not an optional field
on `highlight-token`, so keyword tokens can't carry a section kind that is always
`none`. `web/src/monacoSectionCodeLensProvider.ts` places one lens per header whose
kind has a command. The commands are a `Partial<Record<SectionKind, Command>>`,
typechecked against the generated enum. Both regexes are deleted. Covered by
`section-header-code-lens.feature` (commented header and indented header) and
`src/highlight_tests.rs`.

---

## [x] 15. Part volume/octave limits exist only in TS (cosmetic, plus a Rust bug)

- TS: `web/src/components/PartRow.tsx:173-174` (volume 1–100), `:15-25` (octave ±4).
- Rust accepts volume 0–255 (`src/parser/parts_parser/lexer.rs:302-308`) and any `i8`
  octave (`:310-322`).

**Failure mode:** a source with `+5` or `150%` shows a blank select or a pinned slider
in the modal.

**Direction:** expose the limits from Rust, and report out-of-range values as a
diagnostic.

**Related Rust-only bug:** `src/midi/mod.rs:193` computes `(volume * 127 / 100) as u8`,
so any volume above 100% emits a MIDI data byte above 127, which is invalid.

**Fix:** `PART_SETTING_LIMITS` (`src/parser/parts_parser/setting_limits.rs`) is now the
only definition of the ranges: volume `0%`–`100%`, octave offset `-4`–`+4`. The parser
clamps an out-of-range value to the nearest bound and reports a diagnostic on its token
(`PartsVolumeOutOfRange`, `PartsOctaveOffsetOutOfRange`), and `update_part_declaration`
clamps too. So no `part-declaration` or MIDI input ever holds an out-of-range value. The
new WIT export `get-part-setting-limits` feeds the Edit Parts slider bounds and octave
options (`usePartSettingLimits`), and `OCTAVE_OPTIONS` and the literal `1`/`100` are
deleted. The minimum volume is `0`, because Rust and `syntax.md` already accepted `0%`;
the slider's old `min={1}` was the odd one out. Covered by
`edit-parts-modal-setting-limits.feature` and `setting_limits_tests.rs`. The MIDI
overflow is fixed separately, in the commit after this one.

---

## [x] 16. Deployment config duplicated across the worker and web (loud / cosmetic)

- GitHub OAuth client id: `crates/live-share-worker/wrangler.toml:72`,
  `.github/workflows/pages.yml:60,151`, `web/.env:19`.
- Worker host: `.github/workflows/pages.yml:65,152`, `web/.env:20`, and hard-coded in
  `web/functions/index.ts:27`.

**Failure mode:** GitHub rejects the sign-in (loud), or link previews fall back to the
generic title (cosmetic).

**Direction:** read each value from a single source at build time.

**Fix:** `crates/live-share-worker/wrangler.toml`'s `[vars]` is now the only place the
sign-in client id (`SYNCED_SHARE_GITHUB_CLIENT_ID`) and the worker's public host
(`SYNCED_SHARE_PUBLIC_HOST`, new; the worker itself never reads it) are written.
`tests/export_openapi.rs` parses the toml (`toml` dev-dependency) and publishes both to
the generated `web/src/generated/live-share-worker/deployConfig.json` via
`build:worker-types` (already a `prebuild`/`predev` step, so `pages.yml` needs no extra
step). `web/vite.config.ts` defaults `VITE_SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID` /
`VITE_SYNCED_SHARE_HOST` to it (an explicitly set value, e.g. `.env.local`'s local
worker host or e2e's, still wins), and `web/functions/index.ts` imports the JSON. The
copies in `web/.env` (now deleted, along with the dead storage-backend
`VITE_GITHUB_OAUTH_*` vars) and both `pages.yml` build steps are gone, and `vite` dev
defaults the host to the local `wrangler dev` one in `vite.config.ts`, so no `.env*`
file is needed. A personal gitignored `web/.env.local` can still override any value.

---

## [x] 17. e2e-only couplings to Rust output (fail loudly in e2e, never in prod)

- `data-guitar-frets`: `src/pitch_description/guitar_diagram_svg.rs:37` ↔
  `web/e2e/features/steps/selection-pitch-drawer.steps.ts:337`.
- `web/playwright.config.ts:102-105` hard-codes the worker's variable names and the
  `{client_id}` placeholder. These must match
  `crates/live-share-worker/src/oauth.rs:55,62,74`, `identity/github.rs:29` and
  `identity.rs:56`.
- The mock server's `/applications/e2e-test-client-id/...` paths
  (`web/e2e/mock-github-oauth-server.ts:144,156`) must equal the client id the worker
  runs with in e2e.

**Direction:** low priority. Export the variable names from one place, or add a check
that the worker actually read each override.

**Fix:** the worker now publishes the names itself: `e2e_reset_config` became
`e2e_worker_config` (`crates/live-share-worker/src/e2e_worker_config.rs`), which reads
the now-`pub(crate)` consts in `oauth.rs`, `identity/github.rs` and `identity.rs` (plus
the new `CLIENT_ID_PLACEHOLDER`) and `tests/export_openapi.rs` writes them to
`e2eWorkerConfig.json` (replacing `e2eResetConfig.json`). `web/playwright.config.ts`
builds every `--var` from that JSON. The e2e client id is defined once in
`web/e2e/e2eGithubClientId.ts`: playwright passes it to the worker as the client-id var
(dropped from the committed `.dev.vars`) and the mock GitHub server builds its
`/applications/<id>/{grant,token}` paths from it.

Not removed: `data-guitar-frets`. It is a test-only attribute written by a Rust format
string and read by one Playwright step; the WASM component isn't reachable from the
e2e step process, so the name can't be imported from a generated type. The Rust unit
tests in `src/pitch_description/tests.rs` pin the attribute, and a rename fails the
e2e step loudly, so drift is caught, not prevented.

---

## Checked in that audit and found clean

- **Every WIT export:**
  - `enabled-tracks`/`disabled-lyrics` are abbreviations that round-trip from `list-parts`.
  - `directive-row-offset` is passed through unchanged.
  - Rename, range edits and whole-source rewrites are typed.
  - The `describe-selection` strings are display-only.
  - SVG/PDF source embedding and extraction are both in Rust.
  - `highlight-tokens` is typed.
  - Two exports have no TS caller: `get-measure-index-at-offset` and `greet`.
- **Diagnostic text:** nothing in `web/src` or `web/e2e` matches on Rust messages. The
  error strings e2e asserts are written by TS.
- **Export output:** TS never parses WAV/MP3/MIDI/PDF bytes. Download names are built in
  TS, and the names inside the zip are Rust-only. There are no duplicated audio
  constants.
- **Instruments:**
  - The category/role/articulation unions are owned by TS; Rust only fuzzy-matches them.
  - Percussion names come from the shared `web/src/data/gmPercussion.json`.
  - Every parse call passes `GM_INSTRUMENTS`.
- **Fonts:** file names and family names come from `fonts/fonts.json` on both sides
  (Rust via `build.rs`), keyed by the generated `FontFamily` type.
- **Demo and new-file templates:** covered by `tests/main/demo_source.rs` and
  `tests/main/new_file_template.rs`.
- **Worker:**
  - CORS allows GET/POST and only `Content-Type`, which covers every route in the
    schema; the token travels in the body.
  - The OAuth code, state and `redirect_uri` go through typed bodies.
  - The e2e mocks use the generated `ApiError` and the typed `WorkerPath`.
- **Section labels:** the preview click (`previewSelection.ts:38`) and the section
  toolbar both use labels from the same parsed directive. The only weakness is that
  duplicate labels collide in `find`.
