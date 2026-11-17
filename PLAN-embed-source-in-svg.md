# PLAN: Embed source in SVG (PR 1 of 2)

## Goal

Embed the original `.poentaro` source text as a hidden, non-visual payload inside
generated SVG output, and provide a way to extract it back out — so that if a
`.poentaro` source file is lost, it can be recovered from a previously exported
SVG (or, eventually, PDF — see `PLAN-embed-source-in-pdf-and-web-import.md` for
PR 2).

This is **PR 1**: SVG-only embed + extract, proven with a Rust round-trip
test. No PDF support and no web UI in this PR — those are PR 2, deliberately
split off because of open questions around `usvg` metadata preservation and
the size of the web-side change.

## Background / where things live

- SVG serialization is manual string-building (no SVG crate) in
  `src/serializer/mod.rs`. `serialize_doc` (`src/serializer/mod.rs:10-19`)
  builds the `<svg ...>...</svg>` wrapper around already-rendered elements.
- `serialize(documents: &[SvgDocument]) -> Vec<String>` (`src/serializer/mod.rs:6-8`)
  is called from two places in `src/lib.rs`:
  - `render_svgs_with_parts` (`src/lib.rs:210-228`), the common path used by
    `render_svgs` (`src/lib.rs:231-233`, takes only a parsed `Score`, **no**
    source string available) and by `render_svgs_from_source_filtered_with_lyrics`
    (`src/lib.rs:268-291`, **does** have `source: &str` in scope already, used
    today only for `compile(source, ...)` / `list_parts_from_source`).
  - `render_svgs_with_highlight_range` (`src/lib.rs:339`, similar shape).
- CLI SVG generation: `generate_svg` in `src/cli/generate.rs:160-206` reads the
  file twice — once inside `parse_and_group`, and again explicitly at line 194
  via `super::read_source(&opts.input)` — storing it as `content: String`,
  which is passed into `render_svgs_from_source_filtered(&content, ...)`
  (line 202). So the raw source is available at the CLI layer and inside
  `render_svgs_from_source*`, but currently does not reach `serializer::serialize`.
- No existing metadata/comment/non-visual element anywhere in the SVG output
  today (no `<title>`, `<desc>`, `<metadata>`, XML comments). This is new
  territory.

## Design

### Embedding format

Add a `<metadata>` element as the first child of `<svg>` in `serialize_doc`:

```xml
<metadata id="poentaro-source">BASE64_ENCODED_SOURCE</metadata>
```

- Use `<metadata>` (not `<desc>`/`<title>`) — it's the SVG-spec-correct place
  for non-visual payload and isn't surfaced by renderers as a tooltip/label.
- Base64-encode the raw UTF-8 source before embedding. Avoids XML-escaping
  edge cases (source containing `<`, `&`, or `]]>` if we'd used CDATA) and
  makes extraction a simple substring-then-decode, not an XML-aware parse.
- Use the `base64` crate if already a dependency; otherwise add it (check
  `Cargo.toml` first — `svg2pdf`/`pdf-writer` deps may pull one in
  transitively already, but don't rely on a transitive dep, add it directly
  if needed).
- Only embed when source is actually available. `render_svgs(score: &Score)`
  (the `Score`-only API) has no source and should NOT attempt to embed
  anything — no placeholder, no partial embed. Threading source onto `Score`
  itself is explicitly out of scope for this PR (would require changes to
  parsing/`Score`/`Header` structures — bigger and unrelated to the
  serializer change).

### Threading the source through

- Change `serializer::serialize` to accept an optional source:
  `serialize(documents: &[SvgDocument], source: Option<&str>) -> Vec<String>`.
  Update `serialize_doc` similarly to take `Option<&str>` and conditionally
  emit the `<metadata>` tag.
- `render_svgs_with_parts` (`src/lib.rs:210`) gains an `Option<&str>` source
  parameter (or a sibling function — see call-site fan-out below) and passes
  it through to `serializer::serialize`.
- `render_svgs` (`Score`-only, `src/lib.rs:231`) calls with `None`.
- `render_svgs_from_source_filtered_with_lyrics` (`src/lib.rs:268`) already
  has `source: &str` in scope — pass `Some(source)` through to
  `render_svgs_with_parts`.
- `render_svgs_with_highlight_range` (`src/lib.rs:339`) — check whether it has
  source available in its call chain; thread through the same way if so, or
  pass `None` if that path is only ever `Score`-based (verify at
  implementation time, the explore-agent research above didn't fully resolve
  this one).
- Keep the function signature changes additive/minimal — prefer adding a
  parameter over duplicating functions, but if `render_svgs_with_parts` has
  many call sites where threading an extra param is awkward, a
  `render_svgs_with_parts_and_source` sibling is acceptable too. Use
  judgement at implementation time; don't over-engineer this.

### Extraction

Add a new function, e.g. in `src/serializer/mod.rs` or a new
`src/source_embed.rs` (pick whichever reads more naturally once the embed
code exists — a dedicated module is probably cleaner since extraction has
nothing to do with serialization per se):

```rust
pub fn extract_embedded_source(svg: &str) -> Option<String>
```

- Find the `<metadata id="poentaro-source">...</metadata>` substring (simple
  string search for the opening/closing tags is fine — we control the exact
  format we emit, no need for a general XML parser), base64-decode the
  contents, and return as `String`. Return `None` if the tag isn't present or
  decoding fails (e.g. a hand-edited or third-party SVG) — do not panic or
  error, this needs to be a graceful "not found" for the future web Import
  flow (PR 2).

### Testing

- Per project convention, test cases live in separate files, not inlined.
  Add a round-trip test: render a known `.poentaro` fixture to SVG via
  `render_svgs_from_source`, assert `extract_embedded_source` on the output
  recovers the exact original source string byte-for-byte.
- Add a test that `render_svgs` (the `Score`-only path, `None` source) does
  NOT emit a `<metadata id="poentaro-source">` tag at all.
- Add a test that `extract_embedded_source` returns `None` gracefully on an
  SVG string with no embedded metadata (e.g. a plain `<svg></svg>`).
- Check whether an existing fixture/golden-SVG test snapshots full SVG output
  byte-for-byte (`tests/` directory) — if so, those goldens will need
  regenerating since every SVG now gains a `<metadata>` prefix. Find and
  update them as part of this PR, don't leave the suite red.

## Explicitly out of scope for this PR

- PDF embedding (`src/pdf.rs`) — see PR 2 plan. Needs an empirical check of
  whether `usvg::Tree::from_str` (used in `src/pdf.rs:46`) preserves
  `<metadata>` elements when parsing the SVG before `svg2pdf::to_chunk`
  re-emits it as a PDF XObject. If it strips it, PR 2 will need a different
  approach for PDF (e.g. PDF XMP metadata written directly via `pdf_writer`).
- Web UI / wasm-bindgen extraction function / Import button — see PR 2 plan.
- `Score`-level source storage — out of scope, `render_svgs(&Score)` stays
  source-less.

## Acceptance criteria

- [ ] `cargo run -- generate svg simple.poentaro` produces an SVG with a hidden
      `<metadata id="poentaro-source">` tag containing the base64-encoded
      original source.
- [ ] A round-trip Rust test proves `extract_embedded_source(svg) == original_source`.
- [ ] `render_svgs(&Score)` (no source available) emits no metadata tag, and
      `extract_embedded_source` returns `None` for such output.
- [ ] No visual change to rendered SVG output (metadata tag is non-rendering).
- [ ] `syntax.md`/`ARCHITECTURE.md` updated only if this changes user-facing
      `.poentaro` syntax or a documented layer/type signature — likely not
      needed since this is purely an internal serializer change, but check
      `ARCHITECTURE.md`'s description of the serializer layer and update if
      the `serialize` signature change is documented there.
