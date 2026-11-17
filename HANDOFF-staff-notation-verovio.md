# Handoff: Western staff notation view via MEI + Verovio

## Goal

A toggle in the web app that switches the preview from numbered notation to western
staff notation. Engraving (note spacing, beaming, collisions, line breaking) is
delegated to [Verovio](https://www.verovio.org) (C++ → wasm, LGPL-3.0); Rust
only translates the grouped `Score` into MEI. The frontend stays dumb: it
hands the MEI string Rust produced to Verovio and displays the SVG.

## Status

| Part | State |
|---|---|
| `src/mei_export/` (Score → MEI) | Done, experimental, unit-tested (`src/mei_export/tests.rs`) |
| CLI `poentaro generate mei <file>` | Done |
| Verovio rendering | Wired into the web worker (`web/src/worker/staffMessageHandlers.ts`), lazily `import()`ed on first staff render; Vite emits it as its own ~8 MB chunk (`poentaro.worker` stays ~465 kB) |
| WASM `render_mei` (TODO 1) | Done — part visibility filtered in Rust (`render_mei_from_source_with_visibility`) |
| Web toggle (TODO 3) | Done — Numbered/Staff toggle above the preview, remembered **globally per browser** (`localStorage` `poentaro:preview-notation:v1`, `useStaffNotation.ts`); switch to per-file if wanted |
| e2e (TODO 5) | Partly — `web/e2e/features/staff-notation-view.feature` (toggle, hidden part, remembered after reload); the click-to-cursor scenario waits on TODO 4 |
| Click / selection / playback (TODO 4) | Not started |
| Staff PDF export | Done — Export > PDF / PDF (ZIP) follow the preview's notation; Verovio's pages go back to Rust (`src/staff_export/`) for the PDF. Verovio's `<title class="labelAttr">` tooltips must be stripped or usvg pushes centred text (the title) off the page |
| Licence note (TODO 18) | Done — README "Credits", pinned `verovio@6.3.0` |

Spike result on the production score `choir/把根留住.poentaro` (16 parts:
SATB + divisi, sustain, pluck, chords, 3 lyric lines; 69 measures): Verovio
rendered 4 condensed A4 pages in ~200 ms (Node, M-series Mac), with correct
key/time signatures, clefs (incl. octave-down treble for Tenor), ties
across bars, slurs, beams, chord symbols, rehearsal marks and both verses.

Size of the Verovio module (`verovio@6.3.0`, `dist/verovio-module.mjs`, wasm
embedded): **7.0 MB raw, 2.3 MB gzip, 1.8 MB brotli**. Must be lazy-loaded
only when the toggle is first turned on.

## Reproducing the spike

The repo's `ba-gen-liu-zhu.poentaro` is an older copy of the same song. The
current one lives in production (`files` table, `name = 'choir/把根留住.poentaro'`):

```sh
cd crates/live-share-worker
pnpm exec wrangler d1 execute jianpu-live-share --remote --json \
  --command="SELECT content FROM files WHERE name = 'choir/把根留住.poentaro' AND trashed_at IS NULL" \
  | jq -r '.[0].results[0].content' > /tmp/baogen.poentaro
cd ../.. && cargo run -- generate mei /tmp/baogen.poentaro   # → /tmp/baogen.mei
```

Render with Verovio (in any scratch dir, `npm i verovio@6.3.0`):

```js
// render.mjs — node render.mjs in.mei out-prefix
import createVerovioModule from 'verovio/wasm';
import { VerovioToolkit } from 'verovio/esm';
import fs from 'node:fs';
const [input, outPrefix] = process.argv.slice(2);
const tk = new VerovioToolkit(await createVerovioModule());
tk.setOptions({
  pageWidth: 2100, pageHeight: 2970, scale: 40,
  breaks: 'smart', condense: 'encoded', condenseFirstPage: true,
  lyricSize: 4.5, footer: 'none', header: 'auto',
});
tk.loadData(fs.readFileSync(input, 'utf8'));
console.log(tk.getLog());
for (let p = 1; p <= tk.getPageCount(); p++)
  fs.writeFileSync(`${outPrefix}-${p}.svg`, tk.renderToSVG(p));
```

## Local e2e quirk

If e2e global setup fails with `no such table: password_credentials`, the
local `cf dev` D1 state is behind the migrations (`cf d1 migrations apply`
printed `[]` without applying 0006/0007). Apply the missing
`live-share-worker/migrations/*.sql` files to the Miniflare sqlite file under
`crates/live-share-worker/.wrangler/state/v3/d1/` with `sqlite3`.

## Verovio gotchas already learned

- Empty-staff hiding needs **`scoreDef optimize="true"` + `condense: "encoded"`**.
  `condense: "auto"` hid nothing, whether or not the staffGrp had a bracket.
- A `<slur>` must be encoded in the measure of its **start** note, or Verovio
  warns "slur … is not encoded in the measure of its start" and may misdraw.
- Notes' `xml:id`s survive into the SVG as `id="…"` — the export uses
  `n{start}-{end}` (the note's `event_span`) for source mapping.
- `<harm>`/`<reh>` on a staff that condensing hides disappear with it; the
  export hangs them on the top staff that has notes in that measure.

## TODO

Roughly in priority order. Each item should come with a unit test in
`src/mei_export/tests.rs` (or a sibling `*_tests.rs`).

### Wire it into the app

1. **WASM export.** Add a `render-mei`-style function to
   `crates/poentaro-wasm` (WIT in `crates/poentaro-wasm/wit/world.wit`) that takes
   the source plus the same hidden/soloed track state the numbered preview uses
   and returns the MEI string. Track filtering must happen in Rust
   (`filter_tracks`), not TS — see "Web frontend must stay dumb" in `CLAUDE.md`.
   Update `ARCHITECTURE.md` "WASM exports".
2. **Verovio in the worker.** Add `verovio` to `web/package.json`, load it in the
   worker **lazily** (dynamic `import()` on first toggle), keep one
   `VerovioToolkit` alive, and render MEI → SVG pages there. Check what Vite
   does with the 7 MB `.mjs` (it embeds the wasm as base64; consider the
   separate `verovio-toolkit-wasm.js` + caching headers).
3. **Toggle UI.** A numbered / staff switch in the preview toolbar, persisted
   per file or globally (decide with the user). Show a loading state while
   Verovio downloads.
4. **Click / selection / playback mapping.** Parse `id="n{start}-{end}"` on
   clicked `<g class="note">` back to a source span and reuse the existing
   note-selection plumbing (`note_spans.rs`). Playback highlighting: Verovio's
   `getElementsAtTime(ms)` / `renderToTimemap()` can drive the cursor, or map
   our own `note_timings_seconds` via the same ids.
5. **e2e coverage.** Gherkin scenario(s) under `web/e2e/features/`: toggle to
   staff view, see a staff SVG; click a note, editor cursor jumps to it.

### Notation fidelity

6. **Chord symbol line.** Chords currently jump between staves (whatever top
   staff has notes that measure). Options: a dedicated staff with
   `lines="0"` for chords, or `<harm>` on staff 1 plus making staff 1 never
   condense. Pick whatever looks like a lead sheet / choir score.
7. **Tuplets.** `GroupedNote::tuplet` / `resolution_multiplier` are ignored;
   durations are just divided back down. Emit `<tuplet num numbase>`
   around the notes and check `duration::split_duration` with odd values.
8. **Split notes at beat boundaries.** `split_duration` is greedy
   longest-first, so a quarter that starts on the off-beat of 2/4 is written
   as one quarter, not two tied eighths. Follow standard metric notation:
   don't let a note hide the middle of a 4/4 bar or a compound beat.
9. **Sequence / repeats.** `Score.sequence` is ignored — the score is laid out
   in written order. At minimum print the sequence line (like the numbered
   header does); better, emit repeat barlines / D.S. markers where the
   sequence is a simple repeat.
10. **Percussion parts.** Currently written as rests. Emit a one-line
    percussion staff (`lines="1"`, `clef.shape="perc"`) with unpitched notes.
11. **Grace notes, dynamics, nav markers, tempo.** Check which numbered
    directives exist (`syntax.md`) and map them: `bpm=` → `<tempo>`,
    navigation markers → `<dir>`/`<repeatMark>`, etc.
12. **Clef heuristic.** `staves::choose_clef` uses the average sounding pitch
    (bass below MIDI 53) and `octave_offset < 0` for octave-down treble. Test
    on more scores; consider letting the user override per part later
    (would be new syntax → update `syntax.md`).
13. **Divisi on one staff.** Real choir scores put S1/S2 on one staff as two
    layers (stems up/down) and often SATB on two staves. Consider an option
    to merge parts sharing a prefix or a `follow[...]` link into one staff.
14. **Instrument parts.** Decide whether non-vocal parts (sustain, pluck) are
    shown by default in staff view; maybe staff view shows only parts with
    lyrics/vocal soundfonts unless toggled.
15. **Lyrics details.** Held syllables (`Syllable::held`) and melisma should
    draw extender lines (`<syl con="u">` / `wordpos`); lyrics under chord
    parts (`LyricsSlice::targets_chords`) aren't handled at all.
16. **Header.** Subtitle and author are not shown (only title goes into
    `meiHead`; Verovio's auto header needs `<pgHead>` or `respStmt` work).
17. **Unclosed slurs.** A slur opened in a part whose next measure is an
    implicit-fill rest (e.g. `A2` in measures 37–38 of 把根留住) is never
    closed and silently dropped; Verovio then draws the sibling part's slur
    oddly. Decide whether to close it at the last note or report a warning.

### Hygiene

18. **Verovio version pinning + licence note.** LGPL-3.0 is fine when loaded
    as a separate unmodified module; note it in the README/credits.
19. **Fixture test.** Once stable, add an MEI snapshot test for a small
    multi-part score (not the production file), so exporter regressions show
    up in review.
