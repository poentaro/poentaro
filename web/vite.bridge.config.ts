import path from 'node:path'
import { defineConfig } from 'vite'
import { workerDeployConfigPlugin } from './vite.config'

// Builds the bridge page (`bridge/index.html`, `bridge/entry.ts` -> `src/bridge/main.ts`) the
// app's old origins serve after the move to `poentaro.pages.dev` -- see
// "Origin migration" in `ARCHITECTURE.md`. Deployed instead of the app to
// the old Cloudflare Pages project, which serves `index.html` for every path.
export default defineConfig({
  root: path.resolve(__dirname, 'bridge'),
  base: process.env.VITE_BASE_PATH ?? '/',
  envDir: __dirname,
  plugins: [workerDeployConfigPlugin()],
  // Its own dependency cache: sharing the app's `node_modules/.vite` while
  // both dev servers run (e2e) makes each re-optimize over the other, which
  // answers the app's requests with "504 Outdated Optimize Dep" and reloads
  // the page mid-test.
  cacheDir: path.resolve(__dirname, 'node_modules/.vite-bridge'),
  // `bridge/entry.ts` imports `src/bridge/`, outside the root.
  server: { fs: { allow: [__dirname] } },
  build: {
    outDir: path.resolve(__dirname, 'dist-bridge'),
    emptyOutDir: true,
  },
})
