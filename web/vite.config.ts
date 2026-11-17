import { spawn } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import react from '@vitejs/plugin-react'
import { loadEnv, type Plugin, type ViteDevServer } from 'vite'
import { defineConfig } from 'vitest/config'
import fontsManifest from '../fonts/fonts.json'
import deployConfig from './src/generated/live-share-worker/deployConfig.json'

const CARGO_COMPONENT_ARGS = [
  'component',
  'build',
  '--manifest-path',
  '../crates/poentaro-wasm/Cargo.toml',
  '--profile',
  'release-wasm',
  '--no-default-features',
  '--features',
  'wav,mp3,pdf,midi',
  '--target',
  'wasm32-unknown-unknown',
] as const

const JCO_TRANSPILE_ARGS = [
  'transpile',
  '../target/wasm32-unknown-unknown/release-wasm/poentaro_wasm.wasm',
  '--instantiation',
  'async',
  '-o',
  '../crates/poentaro-wasm/pkg-component',
] as const

const WASM_COMPONENT_JS = path.resolve(
  __dirname,
  '../crates/poentaro-wasm/pkg-component/poentaro_wasm.js',
)

function isRustSource(file: string): boolean {
  return (
    file.endsWith('.rs') ||
    file.endsWith('Cargo.toml') ||
    file.endsWith('Cargo.lock')
  )
}

function wasmDevPlugin(): Plugin {
  let server: ViteDevServer | undefined
  let building = false
  let queued = false
  let debounceTimer: ReturnType<typeof setTimeout> | undefined

  const repoRoot = path.resolve(__dirname, '..')

  function runCargoComponentBuild(): Promise<void> {
    return new Promise((resolve, reject) => {
      const child = spawn('cargo', [...CARGO_COMPONENT_ARGS], {
        cwd: __dirname,
        stdio: 'inherit',
      })

      child.on('exit', (code) => {
        if (code === 0) {
          resolve()
          return
        }
        reject(
          new Error(`cargo component exited with code ${code ?? 'unknown'}`),
        )
      })
      child.on('error', reject)
    })
  }

  function runJcoTranspile(): Promise<void> {
    const jcoBin = path.join(
      __dirname,
      'node_modules',
      '.bin',
      process.platform === 'win32' ? 'jco.cmd' : 'jco',
    )

    return new Promise((resolve, reject) => {
      const child = spawn(jcoBin, [...JCO_TRANSPILE_ARGS], {
        cwd: __dirname,
        stdio: 'inherit',
      })

      child.on('exit', (code) => {
        if (code === 0) {
          resolve()
          return
        }
        reject(new Error(`jco transpile exited with code ${code ?? 'unknown'}`))
      })
      child.on('error', reject)
    })
  }

  async function rebuild() {
    if (building) {
      queued = true
      return
    }

    building = true
    try {
      console.log('[poentaro-wasm] Rebuilding...')
      await runCargoComponentBuild()
      await runJcoTranspile()
      console.log('[poentaro-wasm] Rebuild complete')

      const wasmModule = server?.moduleGraph.getModuleById(WASM_COMPONENT_JS)
      if (wasmModule) {
        server?.moduleGraph.invalidateModule(wasmModule)
      }
      server?.ws.send({ type: 'full-reload' })
    } catch (error) {
      console.error('[poentaro-wasm] Rebuild failed:', error)
    } finally {
      building = false
      if (queued) {
        queued = false
        void rebuild()
      }
    }
  }

  function scheduleRebuild(file: string) {
    if (!isRustSource(file)) {
      return
    }

    clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      void rebuild()
    }, 300)
  }

  return {
    name: 'poentaro-wasm-dev',
    apply: 'serve',
    configureServer(devServer) {
      server = devServer

      devServer.middlewares.use((req, res, next) => {
        if (req.url?.includes('.wasm')) {
          res.setHeader('Cache-Control', 'no-store')
        }
        next()
      })

      const watchPaths = [
        path.join(repoRoot, 'crates/poentaro-wasm/src'),
        path.join(repoRoot, 'src'),
        path.join(repoRoot, 'Cargo.toml'),
        path.join(repoRoot, 'crates/poentaro-wasm/Cargo.toml'),
      ]

      for (const watchPath of watchPaths) {
        devServer.watcher.add(watchPath)
      }

      devServer.watcher.on('change', scheduleRebuild)
      devServer.watcher.on('add', scheduleRebuild)
      devServer.watcher.on('unlink', scheduleRebuild)
    },
  }
}

// `Set` dedupes in case two roles happen to point at the same font file (as
// when experimenting with a single typeface for both serif and sansSerif).
const DEPLOYED_ASSET_FILES = [
  ...new Set([
    fontsManifest.serif.filename,
    fontsManifest.sansSerif.filename,
    fontsManifest.monospace.filename,
    'GeneralUser_GS.sf2',
  ]),
]

const fontsDir = path.resolve(__dirname, '..', 'fonts')

// Cloudflare Pages rejects any single deployed file over 25 MiB, so assets
// above that (currently just the soundfont) are split into parts plus a
// manifest that `useAssetLoader` reassembles client-side.
const MAX_DEPLOYED_FILE_BYTES = 20 * 1024 * 1024

function splitLargeFile(srcPath: string, outDir: string, name: string): void {
  const data = fs.readFileSync(srcPath)
  const chunkCount = Math.ceil(data.byteLength / MAX_DEPLOYED_FILE_BYTES)
  const chunkSize = Math.ceil(data.byteLength / chunkCount)
  const parts: string[] = []
  const partBytes: number[] = []
  for (let i = 0; i < chunkCount; i++) {
    const partName = `${name}.part${i}`
    const chunk = data.subarray(i * chunkSize, (i + 1) * chunkSize)
    fs.writeFileSync(path.join(outDir, partName), chunk)
    parts.push(partName)
    partBytes.push(chunk.byteLength)
  }
  fs.writeFileSync(
    path.join(outDir, `${name}.manifest.json`),
    JSON.stringify({ totalBytes: data.byteLength, parts, partBytes }),
  )
}

function copyFontsPlugin(): Plugin {
  return {
    name: 'copy-fonts',
    apply: 'build',
    closeBundle() {
      const outFonts = path.resolve(__dirname, 'dist', 'fonts')
      fs.mkdirSync(outFonts, { recursive: true })
      for (const name of DEPLOYED_ASSET_FILES) {
        const srcPath = path.join(fontsDir, name)
        if (fs.statSync(srcPath).size > MAX_DEPLOYED_FILE_BYTES) {
          splitLargeFile(srcPath, outFonts, name)
        } else {
          fs.copyFileSync(srcPath, path.join(outFonts, name))
        }
      }
    },
  }
}

function serveFontsPlugin(): Plugin {
  return {
    name: 'serve-fonts',
    apply: 'serve',
    configureServer(server) {
      server.middlewares.use('/fonts', (req, res, next) => {
        const filePath = path.join(fontsDir, req.url ?? '')
        if (!filePath.startsWith(fontsDir + path.sep)) {
          next()
          return
        }
        if (!fs.existsSync(filePath)) {
          next()
          return
        }
        res.setHeader('Content-Type', 'application/octet-stream')
        res.setHeader('Cache-Control', 'public, max-age=3600')
        fs.createReadStream(filePath).pipe(res)
      })
    },
  }
}

/** Where `wrangler dev` serves the worker locally (see `dekit.yaml`). */
const LOCAL_WORKER_HOST = 'localhost:8787'

/**
 * Defaults the Synced Share worker host to the deployed worker's (from its
 * `deploy.json`/`cloudflare.config.ts`, exported as `deployConfig.json` by
 * `build:worker-types`) -- or, under `vite` dev, to the local `wrangler dev`
 * one. An explicitly set value (e2e's, or a personal
 * `.env.local`) still wins.
 */
export function workerDeployConfigPlugin(): Plugin {
  return {
    name: 'worker-deploy-config',
    config(_, { mode, command }) {
      const defaults = {
        VITE_SYNCED_SHARE_HOST:
          command === 'serve' ? LOCAL_WORKER_HOST : deployConfig.workerHost,
      }
      const explicit = loadEnv(mode, __dirname, 'VITE_')
      return {
        define: Object.fromEntries(
          Object.entries(defaults)
            .filter(([name]) => explicit[name] === undefined)
            .map(([name, value]) => [
              `import.meta.env.${name}`,
              JSON.stringify(value),
            ]),
        ),
      }
    },
  }
}

export default defineConfig({
  base: process.env.VITE_BASE_PATH ?? '/',
  plugins: [
    react(),
    workerDeployConfigPlugin(),
    wasmDevPlugin(),
    serveFontsPlugin(),
    copyFontsPlugin(),
  ],
  worker: {
    format: 'es',
  },
  server: {
    fs: {
      allow: ['..'],
    },
  },
  test: {
    // Vitest's default include glob also matches `*.spec.ts`, which is the
    // extension Playwright's e2e suite (web/e2e/**) uses — restrict to the
    // unit tests under src/ so `vitest run` doesn't try to execute those.
    include: ['src/**/*.test.{ts,tsx}'],
  },
})
