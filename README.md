# Poentaro

Poentaro (Esperanto for "a tally of points", so a score) lives at
<https://poentaro.pages.dev>. It used to be called jianpu-generator, at
`jianpu-generator.pages.dev`, which now hands your saved files over and
redirects (see "Origin migration" in [`ARCHITECTURE.md`](ARCHITECTURE.md)).

It renders `.poentaro` scores (numbered musical notation; files from before
the rename end in `.jianpu`) to SVG, PDF and MIDI, with a
web editor and a Cloudflare worker for synced sharing.

- Input syntax: [`syntax.md`](syntax.md)
- Rendering pipeline and glossary: [`ARCHITECTURE.md`](ARCHITECTURE.md)

## Setup

The toolchain (Rust, Node, pnpm, just, …) is pinned in [`mise.toml`](mise.toml),
which CI uses too. Install [mise](https://mise.jdx.dev), then from the repo root:

```sh
brew install mise   # or see https://mise.jdx.dev/getting-started.html
mise trust
mise install
```

Activate mise in your shell (e.g. `mise activate fish | source` in
`~/.config/fish/config.fish`) so the pinned versions are on `PATH`, or prefix
commands with `mise exec --`. Using a system Node instead breaks the `cf` CLI,
which needs Node ≥ 22.18 to load `cloudflare.config.ts`.

## Common commands

```sh
cargo run -- generate svg simple.poentaro   # render a score to SVG
just dev                                  # web app + live-share worker locally
```

## Production database (D1)

The live-share worker logs auth events to an `app_log` table in its D1
database (id in
[`crates/live-share-worker/deploy.json`](crates/live-share-worker/deploy.json)).
Authenticate once with `pnpm exec cf auth login`, then run a saved query from
[`crates/live-share-worker/queries/observability/`](crates/live-share-worker/queries/observability/)
(see "Debugging production" in [`AGENTS.md`](AGENTS.md)):

```sh
cd crates/live-share-worker
pnpm exec cf d1 query "$(jq -r .d1DatabaseId deploy.json)" \
  --sql="$(cat queries/observability/auth_rejections.sql)"
```

## Credits

The staff-notation view engraves scores with
[Verovio](https://www.verovio.org) (`verovio@6.3.0`, pinned in
[`web/package.json`](web/package.json)), licensed under the
[LGPL-3.0](https://github.com/rism-digital/verovio/blob/develop/COPYING). It is
loaded unmodified, as a separate module, only when the staff view is first
opened.
