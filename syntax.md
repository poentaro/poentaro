# Poentaro: `.poentaro` Syntax Reference

This document describes the input syntax accepted by **Poentaro** (formerly jianpu-generator) as implemented today. File extension: `.poentaro` (files saved before the rename end in `.jianpu`; they still open and import).

---

## File structure

A document is a list of sections written in [anyfix expressions](https://github.com/wongjiahau/anyfix-expr), each followed by a comma, in any order:

```poentaro
# A comment runs from `#` to the end of the line.

metadata {
    title "Part options",
    title style { font size 32, bold: yes },
    max measures per system 1,
},

parts [
    "Piano" [P]: notes { sound 1, volume: 80 %, octave: + 1 },
    "Echo" [E]: follow [P] { volume: 50 % },
    "Violin" [V]: notes { instrument: violin },
    "Words" [Pv1]: lyrics [P],
],

sequence [Verse, Chorus WITHOUT [E], "Last chorus" ONLY [P]],

preset "Piano only" { visible parts [P], notation: numbered },

score """
    bpm=100 key=D4 time=3/4 label="Verse"
    [P] 1 2 3
    [Pv1] do re mi

    [P] 4# 5 6,      // the comma and the hash are note syntax here

    key=+2 label="Last chorus" break
    [P] 1 2 3

    label="Chorus"
    [P] 3 2 1
    """,
```

- `metadata`: **optional**
- `parts`: **required**
- `sequence`: **optional**
- `score`: **required**
- `preset "Name"`: **optional**, any number; see [Presets](#presets)

Text in any other form still opens in the editor, where it shows syntax errors until it's fixed.

The score is one multiline string, closed by the same run of quotes that opened it. A score line that starts with `"""` needs the string opened with `""""`. Lines may be indented; the indentation isn't part of the score.

**Canonical form.** The formatter drops every `:` it doesn't need and spaces out symbols, so `title : "Song"` becomes `title "Song"` and `volume : 80%` becomes `volume: 80 %`. Both spellings read the same.

**All-caps words are keywords.** `S1`, `M` and `A2` split a run of words, so every part reference is written in brackets: `follow [P]`, `lyrics [M]`, `WITHOUT [A]`. A sequence label that is all caps is written as a string: `"A" WITHOUT [S]`.

**Part names are quoted.** The name is a string, and an abbreviation needs no quotes unless it holds whitespace, `,`, `#`, `"`, a bracket, `.` or `:`. Its words, numbers and symbols are joined without spaces, so `A1&T` still reads `A1&T` after the formatter spaces it out as `A1 & T`.

---

## Comments

Outside the score string, `#` starts a comment that runs to the end of the line. Inside it, `//` does, on its own line or trailing other content, and `#` is note and key syntax.

```text
metadata {
    title "My Song",  # shown in the header

    # this whole line is a comment
    author "Jane Doe",
},
```

A `#` inside a string (e.g. `title "Song #1"`) is not a comment. Outside the score string, a sharp key is therefore written as a string: `key: "F#3"`.

---

## Metadata

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `title` | no | none | Piece title (rendered in header) |
| `author` | no | none | Author name (rendered in header) |
| `subtitle` | no | none | Subtitle line |
| `max measures per system` | no | `4` | Maximum number of measures per system line before wrapping. A ceiling, not a fixed count: a system also wraps early when the next measure wouldn't fit the page width (e.g. in the guitar view, whose note heads are wider) |
| `row height` | no | `24` | Vertical spacing of one part row (pixels) |
| `note number width` | no | `8` | Horizontal space per note column (pixels) |
| `parts list columns` | no | `4` | Number of columns in the parts list header |
| `part label width pt` | no | `40` | Fixed width (points) of the part-label column at the start of each system, shared by every system in the score regardless of how many measures/columns that system's music needs |
| `page margin pt` | no | `25` | Blank space (points) on every side of the page, between the paper's edge and the header, music and page number. A wider margin also narrows each system, so measures may wrap differently |
| `merge duplicate measures across parts` | no | `yes` | Score-wide default for whether identical measures from different parts are merged into a single row (`yes`/`no`); can be overridden from a given measure onward with the `merge_duplicate_measures_across_parts=` directive line; see [Directive lines](#directive-lines) |
| `hide resting parts` | no | `yes` | Score-wide default for whether an all-rest part is omitted from a measure where other parts have content (`yes`/`no`); can be overridden from a given measure onward with the `hide_resting_parts=` directive line; see [Directive lines](#directive-lines) |
| `hide system dividers` | no | `no` | Whether the horizontal divider line between systems is omitted (`yes`/`no`) |
| `directive row offset` | no | `(0, 0)` | Translation `(x, y)` (points) applied to every rendered directive row (bar number, section label, key, bpm, time signature), moving that row's text without affecting the layout of anything else. Not applied to the `sequence` summary header. |

### Text styles

Every rendered text kind (`title`, `subtitle`, `author`, `sequence`, `part legend`, `measure number`, `section label`, `page number`, `running title`, `part label`, `lyrics`, `notes`, `chords`, `note dash`) is configured the same way, as the kind followed by `style`:

```text
<kind> style { font size N, horizontal padding pt N, vertical padding pt N, bold: yes/no, italic: yes/no, underline: yes/no, font family: serif/sans serif/monospace }
```

Any subset of the seven fields may be given, in any order; omitted fields fall back to their default.

| Component | Meaning |
|-----------|---------|
| `font size` | Font size in points |
| `horizontal padding pt` | Horizontal padding (points) reserved before the element, widening its column's spacing. `0` is allowed |
| `vertical padding pt` | Extra vertical padding (points) added above/below the element. `0` is allowed and means none |
| `bold` | Whether the element renders bold (`yes`/`no`) |
| `italic` | Whether the element renders italic (`yes`/`no`) |
| `underline` | Whether the element renders underlined (`yes`/`no`) |
| `font family` | Which of the app's three embedded font roles the element renders in: `serif`, `sans serif`, or `monospace`. Accepted on every kind, including `notes`/`chords`/`note dash`. Those three kinds' glyph widths are re-measured against whichever font this resolves to, as part of column layout, so an override there can't desync rendered glyph widths from the widths layout computes. |

`running title` is the small title shown alone (no subtitle or author) at the top of every page after the first; its text is always `title`. Pages after the first also omit the part list and `sequence` line, so they have more room for music.

Defaults by kind:

| Kind | `font size` default | `horizontal padding pt` default | `vertical padding pt` default | `bold` default | `italic` default | `underline` default | `font family` default |
|------|----------------------|----------------------------------|--------------------------------|-----------------|-------------------|----------------------|-------------------------|
| `title` | `row height * 1.5` | `0` | `4` | `no` | `no` | `no` | `serif` |
| `subtitle` | `row height * 0.8` | `0` | `4` | `no` | `yes` | `no` | `serif` |
| `author` | `row height * 0.6` | `0` | `4` | `no` | `no` | `no` | `serif` |
| `sequence` | `12` | `0` | `0` | `no` | `no` | `no` | `sans serif` |
| `part legend` | `row height * 0.6` | `0` | `4` | `no` | `no` | `no` | `serif` |
| `measure number` | `10` | `0` | `0` | `no` | `no` | `no` | `sans serif` |
| `section label` | `12` | `0` | `0` | `yes` | `yes` | `no` | `sans serif` |
| `page number` | `row height * 0.6` | `0` | `0` | `no` | `no` | `no` | `sans serif` |
| `running title` | `row height * 0.6` | `0` | `0` | `no` | `yes` | `no` | `serif` |
| `part label` | `12` | `0` | `0` | `no` | `no` | `no` | `sans serif` |
| `lyrics` | `row height * 0.6` | `4` | `12` | `no` | `no` | `no` | `serif` |
| `notes` | `lyrics font size` | `4` | `0` | `no` | `no` | `no` | `monospace` |
| `chords` | `lyrics font size` | `4` | `0` | `no` | `no` | `no` | `monospace` |
| `note dash` | `notes font size` | `4` | `0` | `no` | `no` | `no` | `monospace` |

The `notes` horizontal padding is also used for the multi-measure-rest bar's end insets and the tie/slur/underline/tuplet-bracket markings. The `lyrics` vertical padding is extra padding around a lyric syllable's hover/click-target box, added on top of the lyric font's own measured ascender+descender span. The `title` vertical padding adds space above and below the title row (a header text row is one `font size` tall plus this padding on each side, so `0` is the tightest; `title`, `subtitle`, `author` and `part legend` default to `4`), the `subtitle` and `author` vertical padding above and below the row they share (the larger of the two wins), the `part legend` vertical padding above and below each row of the part legend, and the `sequence` vertical padding above and below the `sequence` summary line; the rows below move down to make room. A part label's reserved column width is the separate `part label width pt` field (see the main metadata table above), not part of a style, because it's a layout constant. The `sequence` style's `bold`/`italic`/`underline`/`font family` style the summary line's per-label spans (independent of `section label`, even though a directive line's own inline `label="..."` renders with `section label`'s style; see [Directive lines](#directive-lines)).

Example:

```text
metadata {
    title "My Song",
    title style { font size 32, bold: yes },
    lyrics style { font size 18, vertical padding pt 6, font family: sans serif },
    notes style { horizontal padding pt 6, italic: yes },
    part label width pt 60,
},
```

---

## Parts section

A list of parts, each a quoted display name, an optional abbreviation in brackets, and a kind:

```text
parts [
    "<display name>" [<abbreviation>]: <kind> { <options> },
    "<display name>" [<abbreviation>]: follow [<target abbreviation>] { <options> },
],
```

### Name

| Form | Display name | Abbreviation (row label) |
|------|--------------|----------------------------|
| `"Alto 1 & Tenor" [A1&T]` | `Alto 1 & Tenor` | `A1&T` |
| `"Melody"` | `Melody` | `Melody` |

- The abbreviation is used as the row label and for `[Key]` prefix lines in the score.
- When it is omitted, the abbreviation equals the full display name.
- The display name is stored for future legend rendering; row labels use the abbreviation only.

### Kind

| Pattern | Meaning | Score lines per measure |
|---------|---------|-------------------------|
| `chords` | Chord-symbol row | 1 |
| `notes` | Notes | 1 |
| `percussion` | Unpitched GM drum hits | 1 |
| `follow [X]` | Inherit column layout from the part with abbreviation `X` | same as target |
| `lyrics [X]` | A lyric part: one verse of lyrics for the `notes`/`chords`/`follow` part with abbreviation `X` (see [Lyric parts](#lyric-parts)) | 1 per measure that writes it |

### Options

Every kind but `lyrics` takes options in braces: `notes { instrument: violin, sound 41, volume: 80 %, octave: - 1 }`.

`sound` selects the MIDI timbre, as a General MIDI program number (0–127). For example: `notes { sound 52 }` for Choir Aahs, or `follow [A] { sound 1 }`. If omitted on a `chords` or `notes` part, the default is the sound of the part's [instrument](#instrument-range), or program 0 (Acoustic Grand Piano) when it names none. On a `follow [X]` part, the sound is inherited from the target when omitted, unless the follow part names its own instrument.

For `percussion` parts, the number is instead a **GM percussion key** (e.g. `38` = Acoustic Snare, `36` = Bass Drum 1), not a GM program number: all percussion parts share MIDI channel 9 (the GM drum channel) and a single fixed GM Standard Kit program change; the number selects which drum sample within that kit each hit plays. If omitted, the default is key 38 (Acoustic Snare). The number is not checked against the melodic instrument catalog.

`volume` sets the MIDI volume for that part, as a percentage. The valid range is `0 %` to `100 %`, where `0 %` silences the part; a value above `100 %` emits a recoverable error and is clamped to `100 %`. If omitted on a concrete part, the default is 100%. On a `follow [X]` part, volume is inherited from the target when omitted and may be overridden.

`octave` shifts every note in that part up or down by N octaves in MIDI output only, written with its sign: `octave: - 1`, `octave: + 1`. The valid range is `-4` to `+4`. The offset does not change octave dots in the rendered SVG. If omitted on a concrete part, the default is 0. On a `follow [X]` part, the octave offset is inherited from the target when omitted and may be overridden. Values outside the range emit a recoverable error and are clamped to the nearest bound.

#### Instrument range

`instrument` names the instrument of a `notes` or `follow [X]` part. It sets the part's playable range (a range is never written by hand) and, when the part sets no `sound`, the sound it plays. For example: `"Violin" [V]: notes { instrument: violin, sound 41, volume: 80 % }`, `"Soprano" [S]: notes { instrument: soprano }`.

| Name | Instrument | Range | Default sound |
|------|------------|-------|---------------|
| `guitar` | Guitar | E2–B5 | `24: Acoustic Guitar (nylon)` |
| `violin` | Violin | G3–A7 | `40: Violin` |
| `cello` | Cello | C2–E5 | `42: Cello` |
| `liuqin` | 柳琴 | G3–G6 | `106: Shamisen` |
| `guzheng` | 古箏 | D2–D6 | `107: Koto` |
| `harmonica` | 12-hole chromatic harmonica | C4–C♯7 | `22: Harmonica` |
| `harmonica16` | 16-hole chromatic harmonica | C3–C♯7 | `22: Harmonica` |
| `bass` | Bass voice | F2–D4 | `52: Choir Aahs` |
| `tenor` | Tenor voice | C3–G4 | `52: Choir Aahs` |
| `alto` | Alto voice | G3–D5 | `52: Choir Aahs` |
| `soprano` | Soprano voice | C4–G5 | `52: Choir Aahs` |

General MIDI has no liuqin or guzheng, so they default to the closest plucked lute and zither. A set `sound` always wins over the default sound.

Each note whose sounding pitch (the key plus the part's `octave`) falls outside the range gets its own warning, pointing at the note, e.g. `V: note sounds as F3, below the violin's lowest note G3`. The warning doesn't stop rendering or playback. A part with no instrument is not checked. A `follow [X]` part inherits its target's instrument and sound, and may name its own instrument to override both (`"Violin 2" [V2]: follow [V] { octave: - 1 }`, `"Cello" [C]: follow [V] { instrument: cello, octave: - 1 }`).

An unknown name emits a recoverable error listing the valid names and is ignored. A name on a `chords` or `percussion` part emits a recoverable error and is ignored.

Rules:

- Duplicate abbreviations across tracks are an error.
- At least one track must be declared.
- `follow [X]` cannot be used for the first declared part.
- The target abbreviation `X` in `follow [X]` must refer to an already-declared part (declared before the follower).
- A `follow [X]` part that is not explicitly mentioned in a measure copies `X`'s content and is visually suppressed (row not rendered). This copies notes only: lyrics are never auto-copied to a follow part; a follow part gets a lyrics row only when it has its own `lyrics [...]` part (see [Lyric parts](#lyric-parts)).
- A `follow [X]` part can be partially or fully overridden using `[Key]` prefix lines in the score.

Example (multi-part vocal score with chords):

```text
parts [
    "main": chords,
    "Alto 1 & Tenor" [A1&T]: notes,
    "Alto 2" [A2]: notes,
    "Soprano 1" [S1]: notes,
    "Soprano 2" [S2]: notes,
],
```

Minimal single-part example:

```text
parts ["Melody": notes],
```

---

## Grammar

Read the way [anyfix-expr's README](https://github.com/wongjiahau/anyfix-expr) reads its own grammar: `name : …` is a rule, `|` is alternation, `[ ]` is optional, `{ }` is repetition and `"…"` is literal text. A rule names the tree anyfix-expr produces, not raw characters, so `key : value` means the call `key _` of `value`.

```text
document   : { section "," },
section    : metadata | parts | sequence | preset | score,

# ── metadata ────────────────────────────────────────────────────────────
metadata   : "metadata" "{" { field "," } "}",
field      : key ":" value,
key        : word { word },                 # `max measures per system`
value      : string | number | word { word } | sign number
           | "(" value "," value ")"        # directive row offset : (0, -4)
           | "{" { field "," } "}",         # a text style

# ── parts ───────────────────────────────────────────────────────────────
parts      : "parts" "[" { part "," } "]",
part       : display [ "[" abbreviation "]" ] ":" kind,
display    : string,                        # the abbreviation, too, when no [ ] follows
kind       : ( "notes" | "chords" | "percussion" ) [ options ]
           | "follow" "[" abbreviation "]" [ options ]
           | "lyrics" "[" abbreviation "]",
options    : "{" { option "," } "}",
option     : "instrument" ":" word          # violin, soprano, …
           | "sound" ":" number             # GM program, or GM drum key on percussion
           | "volume" ":" number "%"
           | "octave" ":" sign number,
abbreviation : string | ? words, numbers and symbols, joined without spaces ?,

# ── sequence ────────────────────────────────────────────────────────────
sequence   : "sequence" "[" { entry "," } "]",
entry      : label [ ( "WITHOUT" | "ONLY" ) "[" { abbreviation "," } "]" ],
label      : string | word,                 # a word that is not all caps

# ── presets ─────────────────────────────────────────────────────────────
preset     : "preset" string "{" { preset_field "," } "}",
preset_field : ( "visible" "parts" | "soloed" "parts" ) ":" "[" { abbreviation "," } "]"
             | "notation" ":" word
             | field,                       # any metadata field

# ── score ───────────────────────────────────────────────────────────────
score      : "score" string,                 # see the sections below

sign       : "+" | "-",
```

---

## Score section — measure groups

The `score` string is split into **measure groups** by **blank lines**. Each group is exactly one bar (measure).

```
bpm=92 key=C4 time=4/4 label="Verse 1"
[Melody] 5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_)
[Melody] 白陽旗旛在大道盛宏

[Melody] 3_ (1_1) 0_- 1= 1=
[Melody] 昌花花
```

### Group layout

1. **Optional directive line** — first line containing at least one directive keyword (`bpm=`, `key=`, `time=`, `label=`, `merge_duplicate_measures_across_parts=`, or `hide_resting_parts=`)
2. **Data lines** — most data lines begin with a `[Abbrev]` prefix (see below); a line with no prefix is an error (`score_line_missing_key_prefix`) and is dropped

Lines are trimmed; leading/trailing spaces on a line are ignored. A completely empty line separates measure groups (it is not a data line).

### Key-based part prefix (`[Abbrev]`)

Every data line must begin with `[Abbrev]` to route it to a specific part by abbreviation, including the first declared part:

```
[A2] 5 6 7 0
```

- Exactly one `[Key]` line may appear for a given part in a measure group (its single notes/chords slot); a second `[Key]` line for the same part is a `part [Key] has N lines but only 1 slot(s)` error. Lyrics go on their own lyric part's `[Key]` line (see [Lyric parts](#lyric-parts)).
- An unrecognised abbreviation is an error; the line is dropped.
- Parts not covered by any `[Key]` line use their `follow [X]` target's content when declared as such, or are filled with implicit rests/no-lyrics otherwise.
- A data line with no `[Abbrev]` prefix is a `score_line_missing_key_prefix` error; the line is dropped. Lyrics are no longer attached positionally — write them on a lyric part's `[Key]` line.
- A measure group with zero valid keyed lines is an error (`measure_no_data_lines`).

**Row label when parts render as one unison row:** when two or more parts' compiled content ends up identical for a system (a system being one printed line of music, spanning however many measures were packed onto it), the renderer merges them into a single row, labeled by concatenating the merged parts' own abbreviations with a space (e.g. `S1 S2`).

**Row label omitted when a system boils down to one all-rest row:** a row label only earns its place by distinguishing one row from another sharing the same system (a system being one printed line of music, spanning however many measures were packed onto it). When every row that would otherwise appear in a system — after all merging above — collapses to a single row, and that row's content is entirely rest (whether one resting measure or a run collapsed into a wide multi-measure rest bar), there's nothing else in the system for the label to distinguish it from, so the label is omitted. This applies regardless of *why* it's the system's only row — a genuinely single-part score, or every other part being hidden by `hide_resting_parts=` — but not when that lone row actually sounds something, and not when more than one row shares the system even if all of them happen to be resting.

**Example — only part C plays, A and B are not-mentioned:**

```poentaro
parts ["A": notes, "B": notes, "C": notes],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4

    [C] 5 6 7 0
    """,
```

Measure 2: C plays `5 6 7 0`. A and B have no explicit lines → filled with `0` (rest) and marked not-mentioned (rows suppressed).

**Example — key-based lines in one measure with a follow part:**

```poentaro
parts ["A": notes, "C": notes, "B": follow [A]],

score """
    [A] 1 2 3 4
    [C] 5 6 7 0
    """,
```

A: `1 2 3 4`. B: not mentioned → copies A's content via `follow`. C: `5 6 7 0`.

**Example — follow part with partial key override:**

```poentaro
parts [
    "Soprano" [S]: notes,
    "Soprano lyrics" [Sv1]: lyrics [S],
    "Alto" [A]: follow [S],
],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [Sv1] do re mi fa
    [A] 5 6 7 1
    """,
```

Soprano: notes=`1 2 3 4`, lyrics=`do re mi fa`. Alto: notes=`5 6 7 1` (key override), no lyrics — a follow part's notes-only override does not copy the target's lyrics; declare Alto its own `lyrics [A]` part if it needs lyrics.

---

## Directive lines

An optional first line of whitespace-separated `key=value` directives sets global values for that measure and onward (until overridden):

```
bpm=92 key=C4 time=4/4 label="Verse 1"
```

| Directive | Example | Effect |
|-----------|---------|--------|
| `bpm=` | `bpm=120` | Tempo (beats per minute) |
| `key=` | `key=C4`, `key=F#3`, `key=Bb4`, `key=+2`, `key=-1` | Key signature (`1` = this note), or a shift of the current key by `N` semitones |
| `time=` | `time=4/4`, `time=3/4` | Time signature |
| `label=` | `label="Verse 1"` | Section label rendered above the row group |
| `merge_duplicate_measures_across_parts=` | `merge_duplicate_measures_across_parts=no` | Overrides the `metadata` default from this measure onward (`yes`/`no`) |
| `hide_resting_parts=` | `hide_resting_parts=no` | Overrides the `metadata` default from this measure onward (`yes`/`no`) |
| `break` | `break` | Forces a new system (line) to start at this measure |

Rules:

- Multiple directives may appear on one line, separated by whitespace.
- `label=` value must be a quoted string; empty labels are rejected.
- Directives apply to **all** parts. They are stored on the first notes part and propagate through grouping.
- `label` applies only to the measure where it is declared (does not persist to the next bar) — this is true for rendering purposes and whenever no `sequence` section is present. When a `sequence` section **is** present, each label additionally denotes a *span* of measures for playback-order purposes: see [`sequence` — explicit playback order](#sequence--explicit-playback-order) below.
- `bpm`, `key`, and `time` persist until the next directive line overrides them.
- `key=+N` / `key=-N` changes key **relative to the key in effect**, by `N` semitones, and then persists like an absolute `key=`:
  - It resolves to the conventional spelling of the new tonic, i.e. the one with the fewest accidentals: `C`, `Db`, `D`, `Eb`, `E`, `F`, `F#`, `G`, `Ab`, `A`, `Bb`, `B`. So `key=C4` then `key=+1` is `Db4`, never `C#4`.
  - It carries across octaves: `key=B4` then `key=+1` is `C5`; `key=C4` then `key=-2` is `Bb3`.
  - Relative changes stack: `key=C4`, then `key=+2` (D4), then `key=+2` again is `E4`.
  - Before any absolute `key=`, it shifts the default key `C4` (`key=+2` as the first key is `D4`).
  - `N` must be a whole number from `-12` to `+12`; a value outside that range is an error and is clamped to the nearest bound.
  - `key=0` (or `+0` / `-0`) is a warning and leaves the key unchanged.
  - Rendering (`1=…`), playback and staff notation behave exactly as if the resolved key had been written.

  ```
  time=4/4 key=C4 bpm=120 label="Chorus"
  [Melody] 1 2 3 4

  key=+2 label="Last Chorus"
  [Melody] 1 2 3 4
  ```

  Here the last chorus is in D4 (`1` = D4).
- `merge_duplicate_measures_across_parts` and `hide_resting_parts` also persist until the next directive line overrides them; unset, they start from the `metadata` value (or its default of `yes`) for the first measure.
- `break` is a bare keyword (no `=value`). It applies only to the measure it's written on — it does not persist — and forces that measure to start a new system, even if `max_measures_per_system` hasn't been reached yet. It's a no-op if the measure would already be first in its system (e.g. it's the very first measure of the score, or the previous measure already filled its system). Since it changes where a system boundary falls, a `break` measure is never absorbed into a collapsed multi-measure rest run that would otherwise start before it — see [Multi-measure rests](#multi-measure-rests).

### Rendering

When `time=` or `bpm=` changes on a measure, the generator may add a **directive row** above the bar-number / section-label row for that system line. Time signature and BPM appear once on that row (not on each part row), aligned with each measure’s note-start column. They do not shift notes or lyrics horizontally. If neither value changes on any measure in the line, the directive row is omitted.

Note names: `A` `B` `C` `D` `E` `F` `G`, with optional `#` or `b` accidental, followed by octave digit (e.g. `4`). A relative `key=` value is instead a sign and a number of semitones (`+2`, `-1`).

### `sequence`: explicit playback order

A document may include an optional `sequence` section that states the playback order directly, as a list of section labels (the same labels set via `label="..."` on a measure's directive line):

```poentaro
parts ["Melody" [M]: notes],

sequence ["A", "B", "A"],

score """
    time=4/4 key=C4 bpm=120 label="A"
    [M] 1 2 3 4

    label="B"
    [M] 5 6 7 1
    """,
```

- Each entry is a label declared with `label="..."` in the score. Write it as a string, or as a bare word when it isn't all caps: `Verse`, `"A"`, `"Last chorus"`.
- A label's **span** covers its measure and every following measure up to (but not including) the next `label="..."` measure, or through the end of the score if there is no following label. Above, `A` spans just its own measure, and `B` spans from its measure to the end of the score.
- Labels may be repeated (e.g. `"A", "B", "A"`) to replay a span more than once.
- Each label must be declared **exactly once** in the score; declaring the same label on more than one measure is an error.
- Referencing a label that was never declared in the score is an error; that entry is skipped and the rest of the sequence still resolves.
- `sequence` only affects **MIDI/WAV playback order**: measures always render once, in written order, with normal bar numbers. However, SVG/PDF output does show the resolved order as a left-aligned line ("Sequence: A › B › A") on the first page, with a blank line of space above it, below the title/subtitle/author/part list. Each label is styled the same as an inline `label="..."` directive (bold, italic).

An entry may carry `WITHOUT [...]`, naming part abbreviations (as declared in `parts`) to omit from that specific occurrence's playback, e.g. a chorus written once but replayed several times with a voice dropping out on later repeats:

```text
sequence [Verse, Chorus WITHOUT [S, A2], Verse, Chorus WITHOUT [A2], Chorus],
```

- It affects **only that occurrence**: here, the first `Chorus` omits Soprano and Alto 2, the second omits only Alto 2, and the third (unmarked) plays every part.
- An abbreviation that matches no declared part is an error; that abbreviation is dropped and the rest of the entry (and sequence) still resolves.
- The score's written-out `Chorus` section always renders with every part, once. However, the omissions **are** shown on the "Sequence: ..." summary line (SVG/PDF, first page), right after the label in plain (non-bold/non-italic) text: `Sequence: Verse › Chorus (-S -A2) › Verse › Chorus (-A2) › Chorus`. This is a reader-facing note only, telling a performer which voices tacet on which repeat. The underlying `Chorus` measures are not duplicated or altered.

An entry may instead carry `ONLY [...]`, naming the **only** parts to keep for that occurrence (every other declared part is omitted), for the opposite case: a chorus written once but soloed down to a subset of voices on a later repeat:

```text
sequence [Chorus, Verse, Chorus ONLY [S]],
```

- Here the third `Chorus` plays only Soprano; every other declared part is silent for that occurrence. Naming every declared part is equivalent to no filter at all.
- Otherwise `ONLY` follows the same rules as `WITHOUT`: an unknown abbreviation is an error and is dropped; the written-out section always renders every part; the summary line shows the parts in brackets, e.g. `Sequence: Chorus › Verse › Chorus (S)`.

---

## Presets

A `preset "Name" { ... }` section saves a way of viewing and exporting the song: which parts are shown or soloed, the notation, and any `metadata` field to change. The preview's preset menu lists every preset; **Default** means no preset: the song's own `metadata`, every part, numbered notation.

```text
metadata { title "Amazing Grace", subtitle "SATB" },

preset "Soprano 1" {
    visible parts [main, S1],
    soloed parts [S1],
    notation: violin,
    subtitle "Soprano 1",
    max measures per system 3,
},
```

- The name is quoted and required. Names must be unique, and `Default` (in any case) is reserved.
- `visible parts`: a list of part abbreviations (as declared in `parts`) to show. Parts not listed are hidden and silent, the same as hiding them in the preview. Unset, every part is shown; empty (`visible parts []`), none is. An abbreviation that matches no declared part is an error and is dropped.
- `soloed parts`: the same kind of list, naming the parts to solo, the same as soloing them in the preview: once any part is soloed, only soloed parts are shown and heard, whatever `visible parts` says. Unset or empty, nothing is soloed.
- `notation`: `numbered`, `violin`, `cello`, `abc`, `guitar` or `harmonica`. Unset means numbered. The harmonica view tabs each part for a 12-hole chromatic harmonica in C, or a 16-hole one (hole 1 blow is C3, so middle C is hole 5) when the part names `harmonica16`. Files written before this was renamed say `jianpu`, which still reads as `numbered`. Violin and cello are drawn as a staff, which ignores text styles, so setting a text style in such a preset is a warning.
- Every `metadata` field may also appear. The preset changes only the fields it sets; the rest come from `metadata`. Header text (`title`, `subtitle`, `author`) can be changed but not removed.
- `visible parts`, `soloed parts` and `notation` belong only in a preset; in `metadata` they're an error.
- From the CLI, `--preset "Name"` exports under a preset (the file is named after it); it exports the preset's `soloed parts` when it has any, else its `visible parts`, and `--tracks` replaces both. The CLI draws numbered notation only, so `notation` doesn't apply there.

---

## Notes syntax

Note lines are a sequence of **atoms** (notes, rests, chords, extensions, groups). Whitespace is optional between atoms and is ignored inside `(…)` groups.

Example: `((1 1) 5 5)` is equivalent to `((11)55)`.

### Pitch and rest

| Token part | Meaning |
|------------|---------|
| `1`–`7` | Scale degree (movable do) |
| `0` | Rest |

### Duration suffixes

Duration is measured in **quarter-beats** (sixteenth-note units). In 4/4, one full beat = 4 quarter-beats; a full 4/4 bar = 16 quarter-beats.

| Suffix | Quarter-beats | Typical name (4/4) |
|--------|---------------|---------------------|
| *(none)* | 4 | Quarter note (one beat) |
| `_` | 2 | Eighth note |
| `=` | 1 | Sixteenth note |

Suffix order is flexible (`1_,'` and `1',_` are equivalent).

### Tuplets

`N:{notes}` brackets `N` notes to be played in the time normally taken by a standard "against" count (3-in-2, 2-in-3, 5-in-4, ...); `N:M:{notes}` overrides that with an explicit `M`. The brace opens right after the count, not before it — `3:{1_1_1_}`, not `{3:1_1_1_}`.

```
3:{1_1_1_} 2_ 3_ 4_ 5_ 6_    an eighth-note triplet, then five plain eighth notes
5:4:{1=1=1=1=1=}             a quintuplet of sixteenth notes, explicit 5-in-4
```

| `N` | Implied `M` (against count) |
|-----|------------------------------|
| 2 | 3 |
| 3 | 2 |
| 4 | 3 |
| 5 | 4 |
| 6 | 4 |
| 7 | 4 |
| 9 | 8 |

Any other `N` has no standard implied ratio — omitting `:M` is a recoverable error ("tuplet ratio for N is ambiguous; use `{N:M:...}` to specify explicitly"); write `N:M:{notes}` instead.

The bracket must contain exactly `N` notes/rests/repeat-atoms (each counts once, same rule as `(…)` group note-counting) — a mismatch at the closing `}` is a recoverable error, though the notes present are still emitted and rendered.

Tuplets nest with `(…)` slur/tie groups in either direction:

```
(3:{1_1_1_} 2_) 3_ 4_ 5_ 6_    slur group wrapping a triplet
3:{(1_1_) 1_} 2_ 3_ 4_ 5_ 6_   triplet wrapping a slur group
```

Unlike `(…)` groups, a tuplet **cannot span lines**: an unclosed `{` at the end of a line is a hard parse error, not a cross-line continuation.

**Note:** the measure-capacity check (below) currently compares each tuplet's *written* (nominal, uncompressed) duration against the bar, not its actual rescaled duration — so a tuplet that only fits the bar once compressed/expanded (the whole point of writing one) can be misjudged as too short or too long at parse time. The triplet example above works because its notes' nominal durations, ignoring the tuplet, already sum to the bar's capacity on their own. Until this is fixed, keep a tuplet's *nominal* duration matching what the bar needs, or use it as a measure's only content.

### Octave markers

| Suffix | Meaning |
|--------|---------|
| `'` | Raise octave (each `'` = one octave up) |
| `,` | Lower octave (each `,` = one octave down) |

`'` and `,` **cannot be mixed** on the same note.

Examples: `1'` (octave up), `1,,` (two octaves down), `3_,'` (eighth note, up one octave).

### Accidentals (`#` / `b`)

Append `#` (sharp) or `b` (flat) immediately after a scale-degree digit to raise or lower the pitch by one semitone.

| Notation | Meaning |
|----------|---------|
| `7#`     | Scale degree 7, raised one semitone (leading tone sharpened) |
| `1b`     | Scale degree 1, lowered one semitone |
| `4#`     | Scale degree 4, raised one semitone (tritone) |

Accidentals can be combined with octave modifiers and all duration modifiers: `7#'` (sharp 7, octave up), `1b_` (flat 1, eighth note), `4#.` (sharp 4, dotted).

Rests (`0`) do not accept accidentals.

### Modifiers

| Suffix | Meaning |
|--------|---------|
| `.` | Dotted (add half the base duration). Cannot combine with `=` (sixteenth) notes. |
| `..` | Double-dotted (add half the base duration, then a quarter of it). Only valid when the base duration divides evenly by 4 (see below). |
| `-` | Extend the previous **note or rest** by one beat (4 quarter-beats) |
| `-.` | Extend the previous **note or rest** by one *dotted* beat (6 quarter-beats) — the natural beat of a compound meter (e.g. 9/8) |
| `-..` | Extend the previous **note or rest** by one *double-dotted* beat (7 quarter-beats) |
| `~` | Tie this note to the next note (same pitch and octave required) |

Example: `2 - - -` is a whole note in 4/4 (equivalent to `2---`). Likewise, `0 - - -` (or `0---`) is a whole rest.

`-.`/`-..` are standalone extension atoms (the dot(s) must be glued directly after the `-`, with no space) — they are not the same as a `-` suffix followed by a separate dotted/double-dotted note. In 9/8, `1. -. -.` is a note held across the full measure: a dotted quarter (`1.`, 6 quarter-beats) plus two dotted-beat extensions (6 + 6), totaling 18 quarter-beats. Similarly, `1.. -.. -.. -..` in 7/4 is a note held across the full measure: a double-dotted quarter (`1..`, 7 quarter-beats) plus three double-dotted-beat extensions (7 + 7 + 7), totaling 28 quarter-beats.

A second dot (`..`) adds a quarter of the base duration on top of the dot's own half — e.g. a double-dotted quarter note is `4 + 2 + 1 = 7` quarter-beats. This only lands on a whole quarter-beat when the base duration is a multiple of 4 (quarter notes, half notes, whole notes, ...). Applying `..` to a note whose base duration isn't a multiple of 4 — e.g. an eighth note, `1_..` (base duration 2) — is a recoverable error: the second dot is dropped and the note falls back to being singly dotted (`1_..` behaves like `1_.`, duration 3), with a diagnostic surfaced on the measure. Typing three or more dots (`1...`) is not a distinct feature — it's silently treated the same as two.

You can also attach dashes as suffixes on a note or rest (`2---`, `0---`). Both forms may be mixed in one measure. Repeated rests (`0 0`, `0 0 0 0`) remain equally valid — `0---` and `0 0 0 0` both produce a whole rest in 4/4.

Shorter rests still use `_`, `=`, or `.` on a single `0` (`0_`, `0=`, `0.`).

### Tie and slur groups

Parentheses connect notes with tie/slur arcs (happi123-style 连音符). A group may span measures: the opening `(` can appear at the end of one bar and the closing `)` at the start of the next.

| Form | Meaning |
|------|---------|
| `(12)` | Slur/tie from 1 into 2 |
| `(433)` | Slur chain across 4→3→3 |
| `(6-7)` | Note 6 extended one beat (`6-`), slurred into 7 |
| `111(1` … `2)345` | Cross-measure slur: `(1` opens in bar 1, `2)` closes in bar 2 |
| `(3= (2_1_))` | Nested groups: outer slur 3→2→1, inner slur 2→1 |

Groups may be **nested**: a `(…)` inside another `(…)` adds an inner tie/slur arc while the outer group still connects all enclosed notes. Each nested group must still contain at least 2 notes.

A group must contain **at least 2 notes** (counting notes across a cross-measure open/close). Single-note groups like `(5)` trigger a non-fatal **warning** (`group_too_few_notes`); rendering still proceeds.

### Tie (`~`)

`~` is written immediately after the octave modifier and before any duration modifiers:

```
4~---4---       tie two 4-beat notes
4'~4'           tie two high-4 quarter notes
4~.4.           tie quarter to dotted quarter
4~---4~---4---  chain of three tied notes
(4~---4--- 3)   tie inside a slur
```

Rules:
- Pitch, accidental, and octave must all match the next note — otherwise a recoverable error is emitted and the arc is suppressed.
- `~` on a rest is an error.
- `~` on the last note of the piece (no following note) is an error.
- Ties span freely across measure boundaries.
- Ties may appear inside slur groups `(…)`.

A tie differs from a slur `(…)` in that it requires identical pitch, and carries distinct semantic meaning (duration extension vs. phrasing).

### Repeat the last note/chord (`r`, bare `_`/`=`)

`r` repeats the last sounded pitch/chord as a fresh one-beat attack (a new note, not a tie/sustain). A bare `_` or `=` — one **not** glued directly after a digit — repeats it as an eighth-note or sixteenth-note attack respectively:

```
5 r r __        note 5, then three more 5s: a beat, a beat, and two eighths
5 0 r           note 5, a rest, then another beat of 5 (rests are skipped)
5~_             note 5 tied into its own eighth-note repeat
5__~5           note 5, an eighth-note repeat, tied out into the next note 5
```

Rules:
- "Last pitched note/chord" skips over intervening rests, and persists across measure boundaries (like ties/slurs).
- `r` never takes suffixes: `r_`, `r.`, `r'` are two atoms in sequence (`r` then a fresh atom), not `r` with a suffix glued on. Write repeats as multiple `r`s instead.
- Using `r`/`_`/`=` with no prior pitched note/chord on the track is a recoverable error; the token is dropped.
- A `~` glued directly after a repeat atom (`r`/`_`/`=`) ties that repeat into the following note, following the same rules as any other tie (matching pitch required, dangling tie is an error).

**Gotcha — maximal munch:** whitespace is cosmetic everywhere else in this grammar, but not here. `5_` (glued, no space) is unchanged: it's still note 5 shortened to an eighth note. Only a `_`/`=` that is *not* glued directly after a digit is a repeat atom — so `5 _` (with a space) repeats note 5's pitch as an eighth note, while `5_` does not. There are two exceptions, both because the glued character can't be read as a suffix of the preceding note in that position:
- Right after a tie: `5~_` glues fine (the `~` already claimed that spot).
- Right after another occurrence of the *same* suffix character: `5__` is note 5 shortened to an eighth note (first `_`) plus a repeated eighth-note attack (second `_`), not a no-op double-shorten. Likewise `5==` is a sixteenth note plus a repeated sixteenth, and this chains — `5___` is a note plus two repeats. Mixing different suffix characters still combines onto one atom as before: `5_=` is a single sixteenth note.

Adjacent digits without spaces also start new notes: `505` is three quarter notes; `(12)31` is a group plus two more notes.

Trailing duration may be omitted when the remaining measure beats extend the last note. In 4/4, `1` is equivalent to `1---`; `1 2` is equivalent to `1 2--`.

### Inline directives (notes row)

These tokens may also appear in a notes line (uncommon; usually placed in `(...)` directive rows instead):

| Token | Meaning |
|-------|---------|
| `bpm=N` | Tempo change |
| `1=<Note><octave>` | Key change, e.g. `1=C4`, `1=Bb4` (only when followed by A–G) |
| `N/N` | Time signature change, e.g. `4/4` |

Note: `1=` followed by a digit pitch (e.g. `1=,`) is a sixteenth note, not a key change.

### Measure validation

Note and rest durations in a row must fill the measure capacity. For time signature `N/D`:

```
measure capacity = N × (16 / D) quarter-beats
```

(e.g. 4/4 → 16, 3/4 → 12). Too many quarter-beats is a parse error. A shortfall extends the last note or rest when possible (so a lone `0` filling an empty measure is equivalent to `0---`). Otherwise it is a parse error.

#### Grouping validation (4/4 only)

In 4/4, the parser rejects rhythm spellings that cross metrical boundaries without exposing the split:

1. **Half-bar boundary:** after beat 1, no single note/rest may span from before beat 3 into beat 3 or beyond (quarter-beat position 8). Use a beam group such as `(2_ 2_)` or a tie instead of a single long value (e.g. `1. 2. 3_ 4_` is invalid; `1. (2_ 2_) 3_ 4_ 0_` is valid). Long notes/rests starting on beat 1 (including a fully extended `1` or `1---`) are allowed.
2. **Dotted-eighth tail:** a dotted eighth note/rest at the start of a beat must be followed immediately by a sixteenth note/rest filling the remaining sixteenth (e.g. `1_. 2= 3_ …`); `1_. 2_ 3_ 4_` is invalid (`2_.` is a dotted eighth, not an eighth).

Other time signatures skip these checks for now. Violations are diagnostics attached to the note (half-bar-boundary crossing is a **warning**; the dotted-eighth-tail rule is a **recoverable error**) — the file still renders.

### Multi-measure rests

This isn't new input syntax — it's automatic rendering behavior. When 2 or more consecutive measures are entirely rests (on every currently-visible part, after any `--tracks` filtering) and none of them carries its own directive (label, navigation marker, time signature/BPM/key change) or diagnostic, they render as a single wide rest bar showing the collapsed measure count, instead of one rest measure per bar. A single isolated all-rest measure still renders normally. A lyric part with no syllables counts as resting, so when only lyric parts are visible (the notes they sing along to being hidden or not soloed), a run of blank lyric measures collapses too; while any visible part still plays in those measures, nothing collapses.

### Examples

| Token | Meaning |
|-------|---------|
| `1` | Quarter note on degree 1 |
| `3_` | Eighth note on degree 3 |
| `5=` | Sixteenth note on degree 5 |
| `1_.` | Dotted eighth note |
| `(12)` | Quarter notes 1 and 2, slurred/tied |
| `6,` | Degree 6, one octave down |
| `0` | Quarter rest |
| `0 0` | Half rest (two quarter rests) |
| `0 0 0 0` | Whole rest in 4/4 |
| `0_` | Eighth rest |
| `1. 1= 6=, (2_=2_)` | Mixed durations, octaves, and a slur group |

---

## Lyrics syntax

Lyrics lines are plain text tokenised into syllables:

| Script | Rule |
|--------|------|
| CJK (Chinese, Japanese, Korean) | Each character is one syllable |
| Latin | Space-separated words/syllables |

### Syllable break (`-` attached to a word)

A `-` **attached** to the end of a Latin syllable marks a word split across notes — the hyphen is part of the syllable text:

```
[Melody] 1 1 5 5
twin- kle twin- kle     ← "twinkle" split across two notes each
```

This is distinct from a **standalone** `-` surrounded by whitespace (held syllable, below).

### Held syllable (`-` within lyrics)

A `-` **inside** a lyrics line marks the **preceding** syllable as *held* — it stretches across tied notes:

```
[Melody] 4~4 3 2
he - world     ← "he" is held across the tied note
```

This is distinct from `-` on a notes line (duration extension) and distinct from `_` (see below).

### No-lyrics marker (`_`)

A lyrics line whose **entire** trimmed content is `_` means **zero syllables** for that part in this measure (instrumental bar):

```
[Melody] 1 2 3 4
do re mi fa

[Melody] 5 6 7 1
_
```

- `_` is valid **only** on lyrics columns.
- On notes or chord columns, `_` alone is a parse error (`_` is already the eighth-note duration prefix on notes lines).

### Empty lyrics

Empty lyrics lines are **not** allowed. Whitespace-only lines are treated as measure separators, not as empty lyrics. To express silence, write `_`.

### Lyrics–notes tally

In each measure, the number of lyric syllables must match the number of notes that take lyrics in the paired notes row:

- Each non-rest note head counts, except a **tie continuation** (same pitch immediately after a tied note, including across a bar line).
- Held-syllable markers (`-`) count as their own syllables — e.g. `你 - 好` is three syllables for three lyric slots.
- The `_` no-lyrics marker skips this check (zero syllables allowed regardless of notes).

Mismatch is a non-fatal **warning** (rendering continues, with empty-string syllables inserted for underflow), e.g. `[Soprano] lyrics underflow: ran out of syllables at syllable 3 (fewer syllables than notes)` or `[Soprano] lyrics overflow: 1 extra syllable(s) after all notes are consumed`.

### Lyric parts

Lyrics are declared as their own parts in `parts`, with `lyrics [X]` naming the `notes`, `chords` or `follow` part (declared earlier) whose notes they sing along to:

```poentaro
parts [
    "bass" [b]: notes,
    "Verse 1" [v1]: lyrics [b],
    "Verse 2" [v2]: lyrics [b],
    "Lyrics" [l]: lyrics [b],
],

score """
    time=4/4 key=C4 bpm=120
    [b] 1 2 3 4
    [v1] la la la la
    [v2] na na na na
    [l] da da da da
    """,
```

- A lyric part's line is written with its own `[Abbrev]` key, anywhere in the measure group; it is not tied to the line above it. A lyric part takes no sound, volume or octave settings.
- Lyric parts have their own abbreviation and display name, and share the abbreviation namespace with every other part (a duplicate abbreviation is an error).
- Each lyric part renders as its own row directly under its target's notes row, in declaration order, and is tallied and tie-paired against the notes row independently — a lyric part can have its own `-` held syllables and `_` no-lyrics marker.
- Each verse row's left-margin label shows that lyric part's own abbreviation (e.g. `v1`, `v2`); clicking it, or including it in a click-and-click range selection, selects every syllable that verse sings across the system.
- A lyric part not written in a measure group gets no row for that measure. If a later lyric part is written but an earlier one isn't, the earlier one is filled with `_` for that measure (a blank row). A measure with no lyric line at all has zero lyric rows for that part, not blank placeholders.
- A lyric line written without its target's notes line in that measure group still works: the target part is filled with rests (or follows its target).
- A second line for the same lyric part in one measure group is a `part [Key] has N lines but only 1 slot(s)` error.
- `lyrics [X]` where `X` is unknown, declared later, a `percussion` part or another lyric part is a `parts_lyrics_invalid_target` error; the lyric part is dropped.
- A `follow [X]` part never copies lyrics from `X`; give it its own `lyrics [...]` part.

#### Lyrics under a chord part (`lyrics [X]` where `X` is a `chords` part)

Chords are sparse (one attack per chord, `-` only extends), so a chord part's lyrics are **not** paired one-syllable-per-note. Instead the lyric line carries its own timing: each chord attack is paired with one **bracketed item**, and any syllables written outside brackets flow freely between them.

```poentaro
parts ["c": chords, "l": lyrics [c]],

score """
    [c] 1 -- 7
    [l] (~) 只剩下鋼琴 （陪）我
    """,
```

- A **bracketed item** is `( … )` or the full-width `（ … ）`; both are accepted on input, and both always render as ASCII `( … )`. It holds exactly one syllable. Several syllables in one bracket (`（陪我）`) is an error.
- **Tally**: in each measure the number of bracketed items must not exceed the number of chord *attacks*. A chord rest (`0`) and a `-` extension are not attacks. More brackets than attacks is an error, not a warning; fewer is fine, and the trailing attacks simply have no bracketed item. An unbracketed first syllable that takes a beat-1 chord (see below) counts as one of the bracketed items.
- **Pairing**: the Nth bracketed item sits at the beat of the Nth chord attack (so with fewer brackets than attacks, the first attacks are the ones paired), and the chord symbol is drawn above it.
- **Unbracketed syllables** (tokenised as in [Lyrics syntax](#lyrics-syntax)) are spread **evenly** between the neighbouring bracketed items; those after the last bracketed item spread evenly up to the bar line. They are not tallied.
- **Unbracketed first syllable**: when the first chord attack is on beat 1, an unbracketed first syllable takes that attack as if it were bracketed (it is drawn without brackets), and the following unbracketed syllables spread after it. When the first chord attack is **not** on beat 1 (the chord line starts with a rest), the unbracketed syllables before the first bracketed item are a pickup instead, spread evenly from the bar start. Here `天` sits under the `6` chord and `(好)` under the `5`:

```poentaro
parts ["c": chords, "l": lyrics [c]],

score """
    [c] 6 - 5 -
    [l] 天 空 (好) 藍
    """,
```

- **Layout**: the lyric row decides the measure's width (there are normally more syllables than chord attacks); the chord row adapts to it.
- The `_` no-lyrics marker still means zero syllables for the measure, and skips the tally.

---

## Chord syntax

Chord lines use Nashville number symbols. Duration works like notes: each token occupies one beat; `-` extends the previous chord.

| Token | Meaning |
|-------|---------|
| `0` | Chord rest |
| `-` | Extend previous chord one beat |
| `<symbol>` | Chord (see grammar below) |

### Chord symbol grammar

```
<chord>      ::= <degree> <accidental>? <triad>? <extension>? ("/" <bass>)?
<degree>     ::= 1–7
<accidental> ::= "#" | "b"
<triad>      ::= "m" | "o" | "+" | "sus2" | "sus4" | "sus"
<extension>  ::= "M7" | "7"
<bass>       ::= <degree> <accidental>?
```

Parsing checks longest suffix first (`M7` before `7`; `sus2`/`sus4` before bare `sus`; `m` before extension). Bare `sus` (no digit) means `sus4`, matching standard chord-chart convention.

| Input | Meaning |
|-------|---------|
| `1` | I major |
| `1m` | I minor |
| `1o` | I diminished |
| `1+` | I augmented |
| `1sus2` | I suspended 2nd |
| `1sus4` | I suspended 4th |
| `1sus` | I suspended 4th (alias for `1sus4`) |
| `17` | I dominant 7th |
| `1M7` | I major 7th |
| `1m7` | I minor 7th |
| `1#m7` | ♯I minor 7th (rendered `♯1m⁷`) |
| `3b` | ♭III major |
| `1/5` | I major, 5 in bass (e.g. C/G) |
| `6m/5` | vi minor, 5 in bass (e.g. Am/G) |

### Duration suffixes

Chord heads accept the same suffixes as notes: `_`, `=`, `.`, and suffix `-`. Octave markers (`'`, `,`) are not allowed on chord lines.

### Repeating the last chord

`r` and bare `_`/`=` work the same way as on notes lines — see [Repeat the last note/chord](#repeat-the-last-notechord-r-bare-_) above: `1 r` repeats chord `1` for another beat, and `1 _` repeats it as an eighth note.

### Tie and slur groups

Parentheses work identically to notes lines. Spaces inside groups are ignored. Examples: `(1-6m-)`, `(1 - 6m -)`.

Example:

```
[chords] 1 - 6m -
[Melody] _1 _1 _1 =1 =1 1_ 6, (6_)
```

---

## Percussion syntax

Percussion lines carry unpitched GM drum hits. Duration works like notes: each token occupies one beat; `-` extends the previous hit.

| Token | Meaning |
|-------|---------|
| `0` | Rest |
| `x` | Hit |
| `-` | Extend previous hit one beat |

Duration suffixes (`_`, `=`, `.`), tie/slur groups (`(...)`), and the repeat-last-atom shorthand (`r`, bare `_`/`=`) work the same way as on notes lines — see [Notes syntax](#notes-syntax). Octave markers (`'`, `,`) and accidentals are not allowed on percussion lines, since hits have no pitch.

Example — snare and bass drum hitting simultaneously:

```
parts ["Snare": percussion { sound 38 }, "Kick": percussion { sound 36 }],

score """
    [Snare] 0 x 0 x
    [Kick] x 0 x 0
    """,
```

---

## Not-mentioned parts

When a part is **not mentioned** in a measure (no `[Key]` line covers it), it is filled with rests (`0`) or no-lyrics (`_`). If, after filling, that part's row is all rests for the measure **and at least one other part in the same measure has real content**, the row is **not rendered** for that measure — the vertical space is reclaimed and rows below move up. This suppression is controlled by the `hide resting parts` metadata field (default `yes`); set it to `no` to always render every part's row, even when it's all rests. A lyric part with no syllables in a measure (not written, or `_`) counts as resting in the same way: it is hidden under `hide resting parts: yes` when another part has content, drawn as a blank row under `no`, and kept whenever every part in the measure is resting.

- A `follow [X]` part that is not mentioned copies `X`'s content (audio plays the same as X).
- A non-follow part that is not mentioned is filled with rests (`0`) or no-lyrics (`_`).
- Measures sharing a system line no longer need to render identical rows. A system's rows are the union of every part (and, per part, every verse) across all of its measures, in `parts` declaration order then verse order. A measure missing a row its system has (a part it doesn't mention, a resting part hidden via `hide resting parts`, or a verse it lacks) gets that row padded in as a full-measure rest or blank verse — it isn't clickable or selectable there, since no note/syllable actually sounds. Systems are otherwise packed greedily, up to `max measures per system` measures, ending a system early when the next measure wouldn't fit the page width; a single measure too wide for the page on its own still gets a system to itself, with a `measure_overflow` warning. Two things still start a new system line early: a `merge_duplicate_measures_across_parts=` change between measures — since that setting controls whether identical parts merge into one shared row, a change partway through would make the union ambiguous — and a `break` directive, which forces the boundary explicitly (see [Directive lines](#directive-lines)).

### Omitted lines — fill table

| Situation | Result |
|-----------|--------|
| Part not mentioned; declared as `follow [X]` | Copies X's content; row suppressed |
| Part not mentioned; no follow target; notes/chord slot | Silently filled with rests (`0`) |
| Part not mentioned; no follow target; lyrics slot | Silently filled with no-lyrics (`_`) |
| Data line missing `[Abbrev]` prefix | Error; line dropped (`score_line_missing_key_prefix`) |
| `[Key]` line with unrecognised abbreviation | Error; line dropped |
| No valid keyed lines in a measure group | Error (`measure_no_data_lines`) |

**Example — part B not mentioned:**

```poentaro
parts ["A": chords, "B": notes],

score """
    [A] 1 2m 3 4

    [A] 1 - - -
    [B] 1 2 3 4
    """,
```

Measure 1: A plays `1 2m 3 4`, B is not mentioned → filled with rests, row suppressed.
Measure 2: A plays `1 - - -`, B plays `1 2 3 4`.

---

## Part stream editor

The web editor's **Edit part as stream** menu picks **one part** of the `score` and edits it as a single continuous stream: no measure groups, no barlines, no directives. The stream replaces the source in the editor pane, one measure per line, each line prefixed by a read-only `[Abbrev]`. Around each measure, the other parts' lines show as dimmed, read-only **ghost lines**, exactly as the source writes them (a measure that doesn't write a part has no line for it), and the part toggles (hide/solo) choose which parts appear. Section labels and bar numbers show as read-only header rows above each measure. Parts that can't be streamed are listed but disabled, with the reason.

Inserting or deleting a note shifts every later note across the following barlines. The stream is written back into the measure-grouped source only on **Apply** (Cancel or Escape discards it), and only the measures whose content changed are rewritten — every other line (other parts, directives, comments) stays byte-for-byte.

```poentaro
parts ["Melody" [M]: notes, "Harmony" [H]: follow [M]],

score """
    time=4/4
    [M] 1 2
    [H] 3 4 5 6

    [M] 5 6 7 1'
    """,
```

Streams of this score, one measure per line:

```
M:  1 2 - -
    5 6 7 1'
H:  3 4 5 6
    @
```

### Live re-barring

The editor re-bars the stream on every keystroke, so each line always holds exactly one measure:

- Only whitespace changes: the break before each measure becomes one newline, and any other line break becomes a space. Notes that overflow a measure move to the next line, and deleting a note pulls the following notes back, rippling through every later measure. A newline typed mid-measure is collapsed.
- Notes past the last measure become new lines (new measures, with a header row and no ghost lines).
- While the stream has an error (e.g. an unclosed `(`, or an `@` past the end of the score), or during IME composition, it is left as typed.
- The keystroke and its re-barring undo as one step.

### Stream syntax

- The content is the part's normal line grammar (notes, chords, percussion or lyrics), without the `[Abbrev]` prefix.
- Whitespace and newlines are insignificant; measures are inferred from beat counts, using the time signature in effect at each measure (past the last measure, the last one's).
- `|` is an error anywhere — there are no barlines (except in a lyric stream under a chords part, see [Lyric streams under a chords part](#lyric-streams-under-a-chords-part)).
- Directives are hidden and never move. Inline directive tokens (`bpm=N`, `1=C4`, `N/N`, …) in a stream are errors; change them on the measure's directive line instead.
- Implicit trailing padding is spelled out: a measure written `1 2` in 4/4 shows as `1 2 - -`.
- A measure the part doesn't write shows as rests that fill it (`0 0 0 0` in 4/4).
- A misbarred measure — more beats than its time signature, or fewer that can't be implicitly padded — shows as written. Re-barring moves its extra (or missing) beats across the following barlines, so applying the stream untouched fixes the barring: `1 2 3 4 5` / `6 7 1 2` in 4/4 becomes `1 2 3 4` / `5 6 7 1` / `2 0 0 0`.
- A note together with its `-` extensions can't cross a barline (error: *crosses the barline of measure N; split it with a tie (`~`)*). Write `1 2 3 4~ 4 5 6` rather than `1 2 3 4 - 5 6`.
- A `-` with no note before it to extend (at the start of the stream) is an error. A part whose source measure starts with such a `-` can't be streamed.
- A tuplet `N:{…}` can't cross a barline either, and a measure can't contain both a tuplet and a `-` extension (the stream would be re-barred differently than the measure renders).
- An opening `(` or `N:{` right before a measure's first note belongs to that measure; `)`, `}` and `~` stay with the measure before.
- Bare `_` / `=` keep their usual meaning (repeat the last note/chord) in notes, chords and percussion streams.

**`@` — follow placeholder (`follow [X]` parts only).** One `@` stands for one whole measure the part doesn't write, so it copies `X` there. `@` must start a measure and nothing else may share its measure; `@` in a non-follow part, or past the end of the score, is an error.

**`_` — no-lyrics measure (lyric parts only).** As on a lyric line, `_` is one whole measure with no lyrics.

### Lyric streams

A lyric stream is cut into measures by its target's **lyric slots**: each measure takes as many syllables as its target has notes that aren't the continuation of a tie (ties carry across barlines). `-` held syllables count as syllables. A measure whose target has no slots takes nothing. If the stream runs out partway through a measure, that measure keeps what it has (a warning; it shows the usual lyrics-underflow warning after Apply). Syllables past the last slot are appended to the score as new measures, so lyrics can be written before the notes: each line of them is one measure, and `_` is a measure on its own. If every syllable past the last slot is `_` (placeholders pushed past the end by syllables typed before them), they are deleted instead.

### Lyric streams under a chords part

A lyric part bound to a `chords` part (or a `follow` of one) has no lyric slots to cut by — its unbracketed syllables flow freely (see [Lyrics under a chord part](#lyrics-under-a-chord-part-lyrics-x-where-x-is-a-chords-part)) — so its stream marks every measure boundary with `|`. The editor shows one measure per line, each ended by `|`:

```poentaro
parts ["c": chords, "l": lyrics [c]],

score """
    [c] 1 - 4 -
    [l] (天) 空 好 (藍)

    [c] 0 5 - -
    [l] 我 （在）想 你
    """,
```

```
(天) 空 好 (藍) |
我 （在）想 你
```

- Each `|` ends a measure; a trailing `|` at the end of the stream doesn't open another one. Moving a `|` moves syllables between the neighbouring measures.
- A measure the part doesn't write shows as `_`. An empty measure (`| |`) is an error; write `_` for a measure without lyrics.
- Each measure is checked like a lyric line under a chord part: bracket mistakes and more bracketed items than the measure's chord attacks (counting an unbracketed first syllable on a beat-1 chord) are errors in the stream.
- Measures past the end of the score are appended to it, so lyrics can be written before the chords: the chords part writes nothing there, so a bracketed item in such a measure is an error. Fewer measures is a warning: the measures after the stream's end lose their lyrics on Apply.

### Applying a longer or shorter stream

- **Shorter** (e.g. after deleting a note): the rest of the last measure is filled with rests, and later measures drop this part's line. A follow part writes explicit rests there instead (dropping the line would make it copy its target); a lyric part just drops its lines. A group whose only data line is this part keeps that line as a rest fill (`_` for lyrics) rather than becoming empty.
- **Longer**: each extra measure is appended to the end of the `score` as a new group holding only this part's line. A lyric stream marks each extra measure with a new line (under a notes part) or `|` (under a chords part).

Apply is refused while the stream has any error; warnings don't block it.

### Which parts can be edited

`notes`, `chords`, `percussion` and `follow [X]` parts, and lyric parts whose target is a `notes` or `chords` part (or a `follow` of one). A part is listed but disabled, with the reason, when:
- any of its measures has too many beats, too few beats that can't be implicitly padded, an inline directive, or a tuplet together with `-` (explicit, or implicit padding in a tuplet measure);
- it is a lyric part under a notes part with a syllable/note count mismatch in any measure other than a `_` one, or a lyric part under a chords part with a bracket error in any measure;
- its own stream wouldn't split back into exactly the same measures (so applying an untouched stream never changes the source);
- the score has a structural error (a section that doesn't parse, no declared parts, or a measure group error such as an unknown `[Abbrev]`) — then every part is disabled.

---

## Quick reference — special line forms

| Whole line | Column | Meaning |
|------------|--------|---------|
| `_` | lyrics only | No lyrics this bar |
| *(omitted)* | any | Rest fill or follow-target copy; row suppressed |
| `[Abbrev] <content>` | notes, lyrics, chord | Key-based line targeting the named part (or lyric part) by abbreviation |

---

## Complete minimal example

```poentaro
metadata { title "Demo", author "Author" },

parts [
    "Melody" [M]: notes,
    "Melody lyrics" [Mv1]: lyrics [M],
    "Harmony" [H]: follow [M],
    "Harmony lyrics" [Hv1]: lyrics [H],
],

score """
    bpm=120 key=C4 time=4/4 label="Verse"
    [M] 1 2 4 5
    [Mv1] do re mi fa

    [M] 1 2 4 5
    [Mv1] _
    [H] 3 5 6 7
    [Hv1] do re mi fa
    """,
```

Bar 1: Melody plays `1 2 4 5` / `do re mi fa`. Harmony is not mentioned → copies Melody, row suppressed.  
Bar 2: Melody plays `1 2 4 5` / `_` (no lyrics). Harmony uses a `[H]` key line to override its notes, and its own `[Hv1]` line for lyrics.
