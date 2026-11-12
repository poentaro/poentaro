// cf config for the Rust Synced Share worker (`.github/workflows/
// live-share-worker.yml` deploys it on every push to master touching
// `crates/live-share-worker/**`; `wrangler.config.ts` holds the build step).
//
// `name` deliberately does NOT reuse the old TypeScript worker's name
// (`jianpu-live`): the two stay distinct Workers so deploying this one never
// silently overwrites the still-running old one. `web/.env`,
// `web/.env.local` and `.github/workflows/pages.yml`'s `VITE_SYNCED_SHARE_HOST`
// point at this worker's `*.workers.dev` host, built from this name.
import { bindings, defineConfig, triggers } from 'cf/config'
import deploy from './deploy.json' with { type: 'json' }

// Local-run overrides, `{ "<var name>": "<value>" }` as JSON in
// `WORKER_VAR_OVERRIDES` (cf has no `--var` flag): e2e's
// `web/playwright.config.ts` enables the test-only `POST /e2e/reset` route
// with it. Never set on deploy, so production only ever sees the values
// declared below.
const overrides: Record<string, string> = JSON.parse(
  process.env.WORKER_VAR_OVERRIDES ?? '{}',
)

// Local-run override of both sign-in rate limits below, in requests per
// minute per IP: e2e signs dozens of accounts in from one IP within a
// minute. Never set on deploy.
const rateLimitOverride = process.env.RATE_LIMIT_PER_MINUTE_OVERRIDE
  ? Number(process.env.RATE_LIMIT_PER_MINUTE_OVERRIDE)
  : undefined

export default defineConfig({
  worker: {
    // Still `jianpu`: renaming a Worker or a D1 database means recreating
    // it (see "Naming" in AGENTS.md). Name anything new `poentaro-…`.
    name: 'jianpu-live-share-worker-rs',
    compatibilityDate: '2024-09-23',
    // `worker-build`'s default output layout: a `build/` dir here containing
    // a JS shim entry point plus the compiled `.wasm` module.
    entrypoint: 'build/worker/shim.mjs',
    // Daily `app_log` prune (the `#[event(scheduled)]` handler in
    // `src/lib.rs`), at an off-peak, non-round minute.
    triggers: [triggers.scheduled({ schedule: '17 3 * * *' })],
    env: {
      // D1 binding. The name ("DB") must match `D1_BINDING` in
      // `src/handlers/mod.rs` -- do not rename one without the other.
      // Migrations live at `../../live-share-worker/migrations` (see
      // `build.rs` for why); pass `--dir` to `cf d1 migrations apply`.
      DB: bindings.d1({
        name: 'jianpu-live-share',
        id: deploy.d1DatabaseId,
      }),
      // Per-IP (`CF-Connecting-IP`) rate limits on `POST /auth/sign-in` and
      // `POST /auth/sign-up`, on top of the per-email lockout in D1. The
      // names must match `SIGN_IN_RATE_LIMITER`/`SIGN_UP_RATE_LIMITER` in
      // `src/handlers/auth.rs`. `namespace` is an id unique within the
      // Cloudflare account.
      SIGN_IN_RATE_LIMITER: bindings.rateLimit({
        namespace: '1001',
        simple: { limit: rateLimitOverride ?? 20, period: 60 },
      }),
      SIGN_UP_RATE_LIMITER: bindings.rateLimit({
        namespace: '1002',
        simple: { limit: rateLimitOverride ?? 3, period: 60 },
      }),
      // Per-IP limit on `POST /origin-migrations` (the one-time handoff of
      // a browser's saved data from the old origin, see
      // `src/origin_migration.rs`): a browser uploads once, so a low limit
      // only stops someone using it as free storage. The name must match
      // `ORIGIN_MIGRATION_RATE_LIMITER` in `src/handlers/origin_migrations.rs`.
      ORIGIN_MIGRATION_RATE_LIMITER: bindings.rateLimit({
        namespace: '1003',
        simple: { limit: rateLimitOverride ?? 5, period: 60 },
      }),
      // Per-API-key limit on `POST /agent` (an AI agent editing its owner's
      // scores, see `src/handlers/agent.rs`), keyed by the key's id, not the
      // IP. Each request can hold many calls, so this bounds requests, not
      // edits. The name must match `AGENT_RATE_LIMITER` there.
      AGENT_RATE_LIMITER: bindings.rateLimit({
        namespace: '1004',
        simple: { limit: rateLimitOverride ?? 60, period: 60 },
      }),
      // The worker never reads it; it is declared so this crate is the
      // single source of deployment config -- `tests/export_openapi.rs`
      // publishes `deploy.json` to
      // `web/src/generated/live-share-worker/deployConfig.json`.
      SYNCED_SHARE_PUBLIC_HOST: bindings.text(deploy.workerHost),
      ...Object.fromEntries(
        Object.entries(overrides).map(([name, value]) => [
          name,
          bindings.text(value),
        ]),
      ),
    },
  },
})
