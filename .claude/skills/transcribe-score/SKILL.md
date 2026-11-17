---
name: transcribe-score
description: Transcribe a real, public-domain musical score into this repo's .poentaro format — either starting a new file or adding a part/voice to one already transcribed — by finding the actual source and parsing it programmatically, instead of composing freehand. Use when asked to transcribe, continue, or add a part to a .poentaro file "from the real score" / "from the original" / by composer+opus, or when a requested part doesn't exist in whatever source was already used for the file.
---

# Transcribing real scores into `.poentaro`

The goal is never to compose "in the style of" a piece — it's to find the
real score and transcribe it faithfully, matching whatever conventions the
file's already-transcribed measures established. If the file has no existing
measures yet, you're establishing those conventions from scratch and should
say so explicitly.

`scores/Air on G String.poentaro` (Bach, BWV 1068) is a worked example of this
whole procedure, including a part (`d`, second violin) added later from a
*different* source edition than the one used for the rest of the file — see
§0 and §1 below for why that came up and how it was resolved.

## 0. Decide what you actually need to find

- If the file has existing measures, treat them as ground truth for
  conventions — don't re-derive them, verify new material against them.
- If the requested part/voice **doesn't exist** in whatever source was used
  for the rest of the file (e.g. the file was transcribed from a reduction —
  flute + guitar — and the new part is "second violin"), don't compose it
  freehand and don't assume it's secretly in the existing source under a
  different name. Go find the edition that actually contains that voice —
  see §1.

## 1. Finding the source

- **Mutopia Project** (mutopiaproject.org) — public domain, LilyPond source
  always available (`piece-info.cgi?id=N` → download link), easiest to parse
  programmatically.
- **IMSLP** (imslp.org) — huge library, but mostly scanned PDFs. Check its
  "Sheet Music" section anyway (even if you already have a source from
  elsewhere) — it lists *every* edition/arrangement uploaded, including which
  ones ship non-scan engraving files (MusicXML, Capella, Finale, or a
  Mutopia-hosted LilyPond source) versus scan-only PDFs. Search for an
  edition whose editor/description mentions the instrumentation you need.
- A composer's work can have **multiple distinct catalog entries** for
  different arrangements of the same piece (e.g. a reduction vs. the
  original full scoring) — these are typically separate pieces/IDs, not
  files under one entry. A composer's catalog *listing* page can be
  incomplete; if a targeted web search turns up a piece-info id the listing
  didn't show, trust the direct id lookup.
- **IMSLP fetch gotcha**: don't trust `WebFetch`'s summary of an IMSLP work
  page — its HTML→markdown pass over these JS-heavy pages is unreliable
  (repeated fetches of the same URL can return different, sometimes
  hallucinated-looking summaries, including false claims that a section
  "wasn't in the provided content"). Fetch the raw wikitext instead and grep
  it yourself:
  ```
  curl 'https://imslp.org/api.php?action=parse&page=<Page Title>&format=json&prop=wikitext'
  ```
  extract `.parse.wikitext['*']`, then search it directly (e.g. for
  `musicxml`, an editor's name, or an instrument name).
- Always confirm public-domain/license status; prefer plain-text engraving
  markup (LilyPond, MusicXML) over image/PDF-only scores.
- **IMSLP now blocks direct engraving-file (zip/Capella/MusiXTeX) downloads**
  behind a JS bot-check captcha on the `/wiki/File:...` route — not scriptable,
  don't try. **PDF downloads still work via curl** if you use the *direct*
  asset URL (`https://s9.imslp.org/files/imglnks/usimg/.../IMSLP######-....pdf`)
  rather than the `/wiki/File:...` page — grab that direct link from a web
  search result snippet or the page's rendered HTML, not from the wikitext
  API (which only gives the wiki filename, not the CDN path).
- **When literally no plain-text source exists** (checked Mutopia, IMSLP's
  engraving-file list, and there's no downloadable MusicXML elsewhere) but
  IMSLP has a **typeset** (not scanned) PDF — description says "Typeset",
  not "Normal Scan" — check whether it was produced by a notation program
  (MuseScore, Finale, Capella): these embed noteheads/clefs/accidentals as a
  private-use-area **SMuFL font** (commonly named `MScore`) rather than
  drawing them as raster images. That turns the PDF into a fully
  programmatically-parseable OMR source — see §2b below. This is
  meaningfully more reliable than eyeballing a rendered image for anything
  beyond a handful of measures (dense ledger-line pitches and beam/dot
  rhythms are exactly what's easy to misjudge by eye).

## 2. Parsing a LilyPond source

Hand-roll a small parser rather than reading rhythm by eye. Key rules:

- Strip markup that doesn't affect the note stream: comments (`%...`), grace
  notes (`\acciaccatura{...}`/`\appoggiatura{...}`), `\trill`. **Then also
  strip every remaining `\word` command generically**
  (`re.sub(r'\\[a-zA-Z]+', '', text)`) — an unstripped command like
  `\fermata` mis-tokenizes into spurious extra notes (its individual letters
  match the note-token regex, e.g. `f`,`e`,`r`→rest,`a`,`a`), silently
  inflating total duration with no explicit error. This only surfaces later
  as an unexplained cumulative-duration mismatch, so strip proactively.
- Tokenize pitch as `letter, accidental(is/es), octave-marks('/,),
  force-mark(!/?), duration, dots` — in that order. A force-mark regex placed
  before the octave-marks silently misparses.
- Compute absolute octave via LilyPond's `\relative` "nearest fourth" rule:
  each note's octave is the closest to the previous note's pitch, then
  adjusted by its own `'`/`,` marks. A chord's first listed note updates the
  reference for what follows.
- Convert pitch → movable-do scale degree relative to the piece's key.
- Track duration inheritance: a note with no explicit duration number
  inherits the last *explicitly written* duration (including inside chords).
- **Check whether the source needs repeat-unnesting.** A `\repeat volta N
  {...} \alternative {...}` wrapper sometimes lives only in a separate
  rhythm/skip "cue" voice, while the actual note voices already write every
  bar out flat and literal (including a physically-duplicated pair of bars
  for the two alternative endings). Detect this by comparing the two
  alternative-ending bars' text byte-for-byte: if identical (or
  near-identical), the voice is already a flat linear pass — just delete one
  copy of the duplicate bar before parsing, rather than reimplementing
  unnesting logic. If the endings differ or there's a genuine nested repeat,
  you do need to manually unnest into a linear pass, matching however the
  existing `.poentaro` file already handles repeats (usually: no repeats at
  all, just written out once).

## 2b. OMR-parsing a notation-software PDF (no LilyPond/MusicXML available)

Worked example: `scores/Minuet in G.poentaro`'s G-minor companion section
(BWV Anh. 115), transcribed entirely from an IMSLP PDF with no plain-text
source anywhere. Use `pymupdf` (`pip3 install pymupdf`, imported as `fitz`).

- **Detect the font first**: `page.get_text("dict")`, collect the set of
  `span["font"]` values. A music-glyph font (often literally named `MScore`)
  alongside `TimesNewRoman*`/`FreeSerif` text spans confirms this is a
  parseable OMR source, not a raster scan.
- **Use `"rawdict"`, not `"dict"`, for glyph extraction.** `"dict"` spans can
  report the *same* `origin` for every character in a multi-char span even
  when the underlying content stream actually positions each glyph
  independently (seen here: three stacked noteheads collapsed to one
  reported position, silently dropping 2 of 3 notes). `"rawdict"` exposes
  each span's `chars[]` with correct per-glyph `origin`. Deduplicate by
  `(round(x,3), round(y,3), code)` afterward — MuseScore's PDF export does
  sometimes draw a true duplicate glyph at the exact same spot (bold
  simulation) and that dedup is legitimate; a `"dict"`-level collapse is not.
- **Identify SMuFL codepoints** by cross-referencing the [SMuFL glyph
  table](https://w3c.github.io/smufl/) or just cross-checking counts against
  what's visually in the score (e.g. "5 gClef + 5 gClef15mb-ish glyphs" for a
  5-system piece with a transposed lower staff strongly suggests those two
  codepoints). Codepoints used in this transcription: `U+E0A4`
  noteheadBlack, `U+E0A3` noteheadHalf, `U+E1E7` augmentationDot (also reused
  for repeat-sign dots — disambiguate by proximity to a notehead vs. a
  barline), `U+E260`/`E261`/`E262` flat/natural/sharp, `U+E4E5` restQuarter.
- **Staff geometry comes for free from `page.get_drawings()`**: filter
  `type=='s'` drawings whose `items` is a list of exactly 5 `'l'` (line)
  entries — that's one staff-line group, and its `rect` gives you the top/
  bottom line y-coordinates *and* left/right x-bounds. In MuseScore's export
  these are drawn **once per measure**, which hands you exact measure
  x-boundaries for free — no separate barline detection needed. Group these
  by y0 to identify systems and RH/LH (or per-instrument) staff pairs.
- **Pitch from y-position**: line spacing is uniform (5pt in a 400dpi-source
  PDF at 1x scale here, i.e. half-step = 2.5pt). `step = round((bottom_line_y
  - notehead_y) / half_step)` gives a diatonic step count above the bottom
  line's pitch (treble clef bottom line = E4); map `step % 7` through
  `['E','F','G','A','B','C','D']` for the letter and `4 + step // 7` for the
  octave. This is exact, not approximate — use it as ground truth over any
  visual reading.
- **Verify the per-note octave field, don't just trust it — a `step // 7`
  boundary bug can silently mislabel specific letters by a whole octave.**
  Caught post-hoc in `Minuet in G.poentaro`'s BWV Anh. 115 section: every
  single letter-C and letter-D note across the whole piece (both staves) had
  been extracted one octave too low, while every other letter was correct —
  a fencepost error at the C-boundary in whatever `step`→octave conversion
  ran, not a random OMR noise. It reproduced faithfully into the `.poentaro`
  output (each buggy note was internally consistent with its own wrong
  octave, so `check`/render showed no error) and was only caught because a
  human listener noticed degree 4/5 sitting an octave below its neighbors.
  To verify before trusting the extraction: group notes by `(system, staff
  role)`, and for each note compute `y + half_step * (letter_pos +
  7*recorded_octave)` — this should be a near-constant across the group (the
  y-axis calibration constant). Take the **mode** of that constant (most
  notes are correct, so the mode reflects the true calibration), then
  recompute each note's step from `y` against that fitted constant and
  compare to its recorded step. Any note where they disagree by exactly one
  octave (not a smaller/random amount) is a systematic mislabel, not noise —
  fix by shifting that note (and only that note) by the disagreement.
- **A "15mb" (quindicesima bassa) marking is easy to miss**: it may render as
  just a small "15" below the clef, easy to mistake for a stray page-layout
  artifact. If a piano piece's second staff is also in treble clef (not bass
  clef), check for this — it means that staff's *sounding* pitch is the
  notated pitch **minus two octaves**, applied *after* the y-position pitch
  computation above, before doing any scale-degree/octave-mark math.
- **Accidental/dot association needs y-tolerance, not just x-proximity**: an
  augmentation dot for a note sitting *on* a staff line is engraved in the
  adjacent *space* (offset by one half-step, e.g. 2.5pt), not at the note's
  exact y. Use `abs(dy) <= half_step + 0.1` (not `< 1.3`) when matching a dot
  or accidental glyph to its notehead, or you'll silently miss dotted notes
  on-line while still catching dotted notes in-space.
- **Beam/rhythm detection is the hard part — verify by construction, not by
  eye.** Quarter and eighth (and sixteenth) notes share the *identical*
  notehead codepoint; only the beam distinguishes them, and there's no
  reliable x-proximity threshold that works for both stem-up and stem-down
  notes (the stem attaches at the notehead's left edge for one direction,
  right edge — offset ~one notehead-width, e.g. 6.5pt — for the other).
  Instead:
  1. Extract note **stems** as `get_drawings()` items with `type=='s'`,
     exactly one `'l'` item, and a plausible stem-length height (roughly
     10–30pt; shorter is a flag/dot artifact, taller is a barline).
  2. Extract **beams** as `type=='f'` (fill-only) items with a small height
     (a slanted rectangle, `<20pt` tall — taller `'f'` items at a fixed
     narrow x-width across the whole page height are the grand-staff brace,
     not a beam).
  3. For each notehead, find its stem (matching x within the up/down offset
     window, y near one end of the stem), then check whether the stem's
     **far endpoint** (the end away from the notehead) falls inside a beam
     rect's bounding box (small x/y tolerance, e.g. 2–3pt).
  4. **Count how many separate beam rects contain that endpoint** — 1 beam
     = eighth note, 2 overlapping beams (a full-length primary beam plus a
     shorter secondary one covering only part of the group) = sixteenth
     note. Don't assume "beamed = eighth"; a beam count check is what
     catches eighth-plus-two-sixteenths groupings (`♪ ♬`-style), which look
     almost identical to a plain 3-note eighth beam at a glance.
  5. **Automate a per-measure/per-voice duration-sum check** (should equal
     the time signature's beat count) as you refine this pipeline, and iterate
     until every measure/voice passes — this is what actually catches beam-
     detection bugs (an x-window too narrow silently drops notes to "quarter
     when it should be eighth," which shows up as a measure summing to more
     beats than it should). Treat "0 mismatches across every measure" as the
     bar for trusting the extraction, not "looks right on the ones I
     spot-checked."
- **Rests**: same font, distinct codepoints per duration (`U+E4E5` =
  quarter rest). Assign to a measure/voice the same way as noteheads (by x
  and staff-y), and fold into the duration-sum check above.
- **Chord/harmony reduction for non-chord parts**: when a `notes`-kind part
  (unlike a `chords`-kind part) hits a genuine multi-pitch simultaneity in
  the source (e.g. a block chord in one hand), this syntax's notes parts
  can't represent a chord — pick one representative pitch (top note for a
  melody/RH-ish part, bottom/bass note for a sustain/LH-ish part) and let the
  `chords` part carry the full harmony at that beat.
- **Movable-do accidentals for a minor-key piece**: this repo's degree
  numbers (`1`–`7`) are always relative to the tonic's **major** scale,
  regardless of the piece's actual mode. For a natural-minor piece, expect
  `3b`/`6b`/`7b` as the *default* (no accidental glyph in the source needed)
  for scale-degrees 3/6/7, and a bare (unmarked) `7` exactly when the source
  has an explicit raised leading tone (harmonic minor) — because raising the
  natural-minor `b7` by a semitone lands exactly back on the major scale's
  (unmarked) reference pitch. Derive each note's *effective* accidental by
  simulating standard notation's per-measure, per-letter accidental
  persistence (an explicit accidental on a letter holds for the rest of that
  measure for subsequent notes of the same letter, absent a new one),
  seeded by the key signature's default-flatted letters — don't hand-wave
  this from the printed accidentals alone.

## 3. Establishing alignment and scaling

1. Parse the already-transcribed portion of the source.
2. Find the duration-scaling factor between source and `.poentaro` measures
   (e.g. "1 real 4/4 bar = 2 numbered measures", every duration doubles).
   Verify on a simple bar (a single held whole note scales to the expected
   measure count).
3. Confirm by matching scale-degree sequences (ignoring octave) across
   several consecutive bars, not just one. Minor discrepancies (an
   ornamental note added or dropped) are normal — note them, don't assume
   your parser is wrong.
4. **Bucket the continuous, un-barred note stream directly into
   numbered-measure-sized windows** — i.e. `(real bar length) / scaling
   factor` units per bucket — rather than bucketing into real-bar-sized
   chunks and then figuring out how each bar's notes split across its N
   mapped measures. The numbered measure grid does not respect real barlines,
   only cumulative duration; bucketing directly avoids a whole class of
   "these notes don't obviously split evenly" confusion.
5. When a note's duration straddles a bucket boundary: split at every
   numbered-measure (16 quarter-beat) boundary it crosses, then, on each
   resulting per-measure fragment, apply the half-bar grouping-validation
   rule (no note/rest may start before quarter-beat 8 and cross it, unless
   it starts at 0 — see `syntax.md`). Chain tie marks through every fragment;
   only the final fragment carries the note's original tie-to-next flag.
   **Automate this check** (assert no fragment starts in `(0,8)` and ends
   past `8`) rather than eyeballing each measure — it scales correctly to
   pieces with dozens of measures.

## 4. Reverse-engineering conventions (don't assume, derive)

- **Melody**: usually literal (degree + scaled duration), occasionally
  dropping or adding a fast ornamental note versus the strict source — that
  is a known, acceptable liberty already present in this repo's
  transcriptions, not something to force onto a *new* part you're adding.
  Write new parts literally/mechanically unless there's a specific reason
  not to, and say so.
- **Octave notation**: don't assume marks track the source's absolute
  register — verify. If there's already a **verified part in the same file**
  covering the same bars from the same or a closely related source voice
  (e.g. this new part's counterpart, like Violin I when adding Violin II),
  derive the octave-mark rule as an explicit formula calibrated against it,
  instead of eyeballing renders note by note:
  1. `scaleposition(note) = letter_index(c=0,d=1,e=2,f=3,g=4,a=5,b=6) + 7 ×
     lilypond_relative_octave` (monotonic in pitch, +7 per real octave).
  2. From one note in the verified part with a known degree, solve for
     the tonic reference's scale position such that
     `floor((scaleposition − reference) / 7) == 0` matches "no mark".
  3. For any other note: band = `floor((scaleposition(note) − reference) /
     7)`. `0` → no mark, `>0` → that many `'`, `<0` → that many `,`.
  4. Verify against 2–3 more already-written measures before trusting it —
     a scale degree 7 sitting immediately below a degree 1 must land one
     band *lower*, even though it's only a semitone away; that's the case
     most likely to expose a formula bug.
  If no such already-verified counterpart exists, fall back to
  render-and-inspect (§6) note-by-note instead.
- **Gotcha — bare `7` next to `1` (caught late, after a whole piece was
  transcribed with this mistake repeated ~4 times in one file): this is not
  only a formula-derivation risk, it trips up plain note-by-note literal
  transcription too.** degree numbers are diatonic *positions*, not
  chromatic distance: an unmarked `7` is a 7th **above** an unmarked `1`
  (10-11 semitones), while the leading tone you actually hear immediately
  before/after a tonic in the source is almost always the neighbor a
  semitone **below** it, i.e. `7,` (one octave-comma down), not bare `7`.
  Every time the parsed source shows degree-1 and degree-7 landing in the
  *same* LilyPond octave number (or one apart, adjacent by letter), that's
  the semitone-neighbor relationship and needs the comma — don't transcribe
  the bare digit just because the source's octave mark on that note "looks
  unchanged" from the previous note. Before finishing, grep the new measures
  for a `1` immediately followed by a comma-less `7` (or vice versa) and
  re-derive each hit's register from the parsed source data rather than
  assuming it's fine.
- **Chords**: derive the exact reduction rule (root+quality from combined
  harmony+bass notes, slash-bass from whatever the bass actually plays,
  harmonic-rhythm granularity) from one already-transcribed measure compared
  against the parsed source for that same instant — don't assume a
  one-chord-per-measure or triad-only convention.
- **Chord symbols are Nashville-number style, not diatonic-auto-quality**: a
  bare degree (`1`, `2`, `5`...) always means **major**, regardless of
  whether that scale degree is diatonically minor/diminished in the key —
  you must explicitly append `m`/`o`/etc. any time the source harmony is
  minor or diminished (e.g. the diatonic ii chord in a major key is written
  `2m`, not `2`). This also means a bare-degree secondary dominant (e.g.
  V/V, built on the *raised* 2nd degree) needs no extra quality marking —
  it's already just the plain major chord `2`.
- **Piano/keyboard sources sometimes split one hand into two written voices**
  for a few bars only (LilyPond `<< {\stemUp ...} {\context Voice = "ii"
  {\stemDown ...}} >>`), typically to add a decorative inner/tenor line
  above the real bass for a climactic passage. When reducing to "two notes
  parts" (one per hand), use the `\stemDown`/`"ii"` voice as that hand's
  actual notes-part line — it's the real bass motion — and fold the
  `\stemUp` voice's pitches into the chord-part's harmony reasoning for
  that beat instead of inventing a third notes part. Confirm which voice is
  "real" by checking whether it continues the surrounding bars' bass-line
  logic (stepwise/functional motion) versus being a one-off addition.
- **Scaling factor is not always a doubling** — verify it fresh per piece,
  don't assume the previous piece's ratio. `Air on G String` needed 1 real
  4/4 bar → 2 numbered measures; `Minuet in G` (3/4) needed a plain 1:1
  mapping (one real bar = one numbered measure, no rescaling of durations at
  all). Always confirm on a simple single-note-per-bar measure before
  trusting it for the rest of the piece.
- Be upfront with the user about which choices are "verified against the
  source" versus "a reasonable interpretation" (chord quality on ambiguous
  chromatic passages is often genuine judgment, not a mechanical fact).

## 5. Writing the new measures

- Convert each note's duration via the scaling factor, then to numbered
  suffixes: plain digit = quarter (4 quarter-beats), `_` = eighth, `=` =
  sixteenth, `.` = dotted (+50%), trailing `-` = +1 beat (4qb) each.
- **Tie-mark placement gotcha** (undocumented, verified by checking existing
  files and the rendered tie arc): when a note has an explicit duration
  suffix (`_`, `=`, `.`), the tie goes **after** the suffix (`4=~`, not
  `4~=` — the documented order mis-tokenizes into a wrong duration). When a
  note has no suffix (plain digit, optionally with trailing `-` extensions),
  the tie goes right after the pitch/octave, before the dashes (`4~---`).
- Sum durations per measure and confirm they total the measure capacity
  before running the tool.

## 6. Validating

- `cargo run -- check "<file>"` — must print only `"<file>": ok`. Any
  diagnostic names the problem and which line/part; isolate with a scratch
  file containing just the suspect measure if needed. `check` only catches
  *syntax* errors, not wrong-but-valid pitches like the bare-`7`-next-to-`1`
  gotcha above — that needs the source-data cross-check, not just this.
- `cargo run -- generate svg "<file>"` — must render with **no pink
  (error-highlight) measures**. Always run `check` too, since `generate`
  doesn't print diagnostics to the console.
- Rasterize and visually inspect: `rsvg-convert -o out.png "<file>.N.svg"`,
  zooming into specific measures/dots with `rsvg-convert -z <N>` + a
  Python/Pillow crop when checking fine detail (e.g. multi-comma octave
  dots, tie arcs).
- Sanity-check the musical ending — does it land on a plausible final
  chord/note given the other parts (e.g. a new inner voice landing on a
  chord tone under the melody's tonic)?
- Clean up generated `.svg`/`.png` output from the repo root before
  finishing (this repo doesn't commit generated output).

## After transcribing

If you learn something generalizable that isn't already covered above (a new
gotcha, a better technique, a source worth remembering), update this file —
that's the point of it being a skill instead of one-off scratch notes.
