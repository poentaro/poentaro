# Plan: switch share links from lz-string to brotli (via WASM)

## Status

Implemented. `compress_share_payload`/`decompress_share_payload` added to
`crates/poentaro-wasm`, `web/src/shareUrl.ts` now compresses with brotli and
base64url-encodes for the URL hash, falling back to `lz-string` then plain
JSON for old links. Round-trip tested against the fixtures from the
measurement table (`crates/poentaro-wasm/src/tests.rs`) and the e2e share-link
suite (`web/e2e/share.spec.ts` and friends).

Supersedes the original domain-specific bit-packing plan below. Kept the old
investigation notes (marked historical) since they document real dead ends;
don't repeat that research.

## Goal

Reduce share link length by replacing `JSON.stringify({filename, content}) →
lz-string` (`web/src/shareUrl.ts`) with brotli compression, encoded as
base64url for the URL hash.

## Real measurements that drove this decision

Compared three approaches against actual fixture files using the real
`lz-string` library (from `web/node_modules`) and Node's built-in
`zlib.brotliCompressSync` (same algorithm a WASM brotli encoder would use),
plus an order-0 Shannon-entropy estimate over the real parsed AST for the
domain-specific bit-packing option (see `examples/bitpack_estimate.rs`,
throwaway — safe to delete once this plan is settled or keep for future
codec spot-checks).

| File | payload bytes | lz-string (current) | brotli + base64url | bit-packing (plan v1, w/ headroom) | bit-packing (order-0 entropy ideal) |
|---|---|---|---|---|---|
| new_file_template.poentaro | 146 | 173 | 148 | — | — |
| simple.poentaro | 208 | 255 | 198 | — | — |
| fixtures/follow_and_key.poentaro | 923 | 762 | 459 | — | — |
| reference.poentaro | 1803 | 1365 | 932 | — | — |
| 彌勒淨土鄉.poentaro (real song, 813 note-stream tokens) | 2723 | 1992 | **1444** | ~3600 (+81%) | ~1633 (−18%) |

Brotli at max quality beats lz-string on every fixture tested, by a growing
margin as files get larger (−14% on a 146-byte file, −27% on the real song,
−40% on a mid-size fixture) — even after paying the full 33% base64url tax.
It also beats the *theoretical best case* for domain-specific bit-packing
(the order-0 entropy lower bound), while requiring no custom codec, no
`.poentaro`-grammar coupling, and no forward-compat/versioning scheme.

**Decision: do brotli-over-wasm instead of bit-packing.** Bit-packing's
realistic win (~15-25%, and only once a real entropy/Huffman coder is built —
naive fixed-width bit-packing as originally scoped is actually **worse** than
today, ~+50-80%) doesn't justify a custom Rust codec, static code-table
calibration, and the versioning discipline that a hand-rolled wire format
demands. Brotli gets a bigger win for a fraction of the implementation cost.

## Why the original plan rejected brotli, and why that reasoning didn't hold up

The original investigation (kept for context, see "Historical" section below)
rejected brotli on the theory that the base64url tax would "cancel much of
the ratio gain." Measured, it doesn't: brotli's ratio advantage over
lz-string is large enough to absorb the 33% encoding tax and still come out
ahead on every fixture size tested, including small ones. The lesson: don't
reason about compression tradeoffs from first principles when the actual
libraries and fixtures are sitting right there — measure it.

## Implementation location: Rust/WASM, not a JS brotli lib

Same rationale as the original plan: the web app already compiles
`crates/poentaro-wasm` and calls into it from the browser
(`web/src/worker/poentaro.worker.ts`, `web/package.json` `build:wasm` scripts).
Add brotli compress/decompress there rather than pulling in a separate JS
brotli package, keeping one dependency surface instead of two.

- Add a `brotli` (or similar pure-Rust) crate dependency to
  `crates/poentaro-wasm/Cargo.toml`.
- Expose `#[wasm_bindgen] pub fn compress_share_payload(payload: &str) ->
  Vec<u8>` and `pub fn decompress_share_payload(bytes: &[u8]) -> String` in
  `crates/poentaro-wasm/src/lib.rs`, following the existing pattern used by
  `render`, `generate_wav`, etc. in that file.
- JS side (`web/src/shareUrl.ts`) only needs to base64url the resulting
  bytes for the URL hash (e.g. via a small helper, or reuse
  a `Uint8Array <-> base64url` utility) — no more `lz-string` dependency for
  this path.

## Open design questions / next steps

1. Pick the Rust brotli crate (pure-Rust `brotli` crate is the obvious
   choice — avoids a C/WASM-toolchain dependency; confirm it compiles
   cleanly to `wasm32-unknown-unknown` target used by `build:wasm`).
2. Decide brotli quality/window-size params — max quality (11) was used in
   the measurements above; check WASM binary size and compression latency
   are acceptable for typical file sizes before locking that in.
3. Decide compatibility story for old share links still lz-string-encoded
   in the wild — likely: try brotli decode first, fall back to the existing
   `lz-string` decode path (`decodeShareHashSuffix` already has a similar
   fallback chain for legacy plain-JSON links, so this is a small addition,
   not a new pattern).
4. Round-trip test using `reference.poentaro`, `彌勒淨土鄉.poentaro`, and the
   other fixtures used in the measurement table above.
5. Remove `lz-string` from `web/package.json` once the old-link fallback
   period (if any) ends.

---

## Historical: original domain-specific bit-packing plan (superseded)

Kept for the grammar research, which is still accurate and may be useful if
bit-packing is revisited later (e.g. layered on top of brotli, though that
combination wasn't measured and isn't recommended without new evidence).

### Original goal

Reduce share link length by replacing the current
`JSON.stringify({filename, content}) → lz-string` encoding
(`web/src/shareUrl.ts`) with a domain-specific binary encoding for the
note-stream (notes/chords), while keeping lyrics/metadata/text as plain text.

### Why not just swap compressors (original reasoning — since revised above)

Investigated brotli as a drop-in replacement for lz-string. Rejected at the
time:
- Browser support for `CompressionStream('brotli')` is inconsistent; would
  need a WASM brotli fallback lib.
- Binary output needs base64url encoding, which adds ~33% overhead back,
  cancelling much of the ratio gain.
- lz-string's `compressToEncodedURIComponent` is already tuned for URL-safe
  output with no extra encoding tax.
- Net (at the time, unverified): brotli likely doesn't beat lz-string for
  these short/medium payloads.

**This conclusion was wrong** — see the measurements above.

### Chosen direction (superseded): domain-specific bit-packing

Rationale: general compressors have zero foreknowledge of `.poentaro` grammar.
We already own the parser/AST, so we can encode semantic tokens directly
(small fixed-cardinality fields) instead of ASCII text, then still lz-string
the result for any residual repetition.

**Scope**: only the note-stream (`notes`/`notes+lyrics`/`chords` row tokens:
pitch, duration, octave, ties, slurs, chord degree/triad/extension/bass).
Lyrics text, titles, metadata stay as plain UTF-8 text — free text doesn't
benefit from a closed-vocabulary token model.

### Ground-truth grammar research (see full findings, gathered via source + fixtures, not syntax.md alone)

Key files:
- Lexer: `src/parser/score/timed_parser/timed_lexer.rs`
- RD parser: `src/parser/score/timed_parser/timed_rd_parser.rs`
- Note head: `src/parser/score/timed_parser/note_head.rs`
- Chord head: `src/parser/score/timed_parser/chord_head/mod.rs`
- Duration/octave suffixes: `src/parser/score/timed_parser/duration.rs`
- Groups/slurs: `src/parser/score/timed_parser/groups.rs`
- AST types: `src/ast/parsed.rs` (`NumberedPitch`, `Accidental`, `ParsedNote`, `ParsedRest`, `ScoreEvent`, `TriadQuality`, `Extension`, `BassDegree`)
- Fixtures to use for round-trip testing: `reference.poentaro` (exercises every token type), `simple.poentaro`, `彌勒淨土鄉.poentaro` (real-world edge cases), `fixtures/follow_and_key.poentaro`

Corrections vs initial assumptions:
- **No barlines/repeat markers exist at all** — measures are blank-line-separated text blocks, not an in-stream token. Nothing to encode.
- **Chords ARE a closed vocabulary**, not free text: `degree(1-7) + accidental?(#/b) + triad?(m/o/+) + extension?(7/M7) + /bass?` — good bit-packing candidate too.
- **Octave is unbounded in the parser** (signed `i8`, no clamp on `'`/`,` count) — pick a generous fixed width + escape hatch, don't assume a doc-given range.
- **Standalone extension token (`-` alone) vs attached extension dash (`-` fused to a note) are semantically distinct** `ScoreEvent`s with different rendering/slur-close consequences — need an explicit bit to round-trip losslessly; not obvious from `syntax.md`.
- **Ties can chain across measure boundaries** — encoder can't assume duration/tie state resets per line/measure.
- `syntax.md` bug found: claims `~` must appear "before duration modifiers" — actually order-independent (confirmed by `3_~3=` in real fixture). Should fix in `syntax.md` at some point (separate from this task).

### Token fields to encode (note-stream)

| field | cardinality / notes |
|---|---|
| pitch | 1-7, or rest (8 states) |
| accidental | none / # / b (3) |
| octave | signed, unbounded — use fixed width (e.g. 4-5 bits) + escape for overflow |
| duration base | none / `_` / `=` (3) + dotted flag (1 bit); dotted+`=` is invalid (auto-corrected upstream, no need to encode) |
| extension dashes | count (stackable, +4 beats each) |
| extension token kind | attached-suffix vs standalone-token (1 bit, needed for lossless round-trip) |
| tie (`~`) | presence flag; illegal on rests |
| slur group | nesting counters (`group_membership`, `group_continuation`, both saturating `u8`), can span measures |
| chord degree/accidental/triad/extension/bass | closed enum, ~1900 combinations total |

Measured field-level entropy (bits/occurrence, real distribution from
彌勒淨土鄉.poentaro — see table above for context): `accidental` ~0,
`octave` 0.28, `group_membership` 0.38, `group_continuation` 0.25, `triad`
0.87, `duration` 1.97, `pitch` 2.62, `kind` 1.48. The skew (most fields
almost always take their default value) is exactly why fixed-width packing
loses to entropy coding — and why entropy coding still only ties/slightly
beats brotli rather than dramatically beating it.

### Versioning / forward-compat (still relevant if bit-packing is revisited)

Fixed-width bit-packing hard-codes field widths and enum cardinality into the wire format — adding new syntax later can't be represented by an old decoder. Decisions:
- Ship a **version byte** at the front of the payload.
- **Reserve headroom** in field widths now (e.g. 3 bits for duration base instead of the minimum 2) so small additions don't require a version bump.
- Use **escape codes** (one reserved enum value per field = "read N more bits") for genuinely novel future additions within the same version.
- Decide product stance: old links are allowed to say "this link used an older format and can't be opened" rather than promising indefinite backward compat forever. Tie this to the same discipline as `CLAUDE.md`'s syntax.md/ARCHITECTURE.md update rule — a note-stream grammar change should trigger a codec review in the same PR.

### Library choice (Rust, if bit-packing is revisited)

Recommendation: **`deku`** — derive-macro based declarative bit-level (non-byte-aligned) struct packing (`#[deku(bits = N)]` on fields/enum variants). Gives correct-by-construction encode/decode instead of hand-rolled bit shifting, which matters because a codec bug here means silent corruption of user share links.

Alternative if more manual control is wanted: `bitstream-io` (explicit `write(bits, value)` / `read(bits)` calls, no derive magic, used in audio codec work). Avoid `bitvec` — it's a bit-array type, not a serialization framework; would end up rebuilding what deku/bitstream-io already provide.
