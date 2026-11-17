import { useEffect, useState } from 'react'

export type AssetStatus = 'loading' | 'ready' | 'error'

export interface AssetLoaderState {
  bytes: Uint8Array | null
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

export const ASSET_CACHE_NAME = 'poentaro-assets-v1'

function resolveAssetUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) {
    return path
  }
  const relative = path.startsWith('/') ? path.slice(1) : path
  return `${import.meta.env.BASE_URL}${relative}`
}

interface AssetManifest {
  totalBytes: number
  parts: string[]
  partBytes: number[]
}

function siblingUrl(resolvedUrl: string, fileName: string): string {
  const dir = resolvedUrl.slice(0, resolvedUrl.lastIndexOf('/') + 1)
  return `${dir}${fileName}`
}

// Some deployed assets (e.g. the soundfont) exceed Cloudflare Pages' 25 MiB
// per-file limit, so the build splits them into parts alongside a manifest.
// Assets without a manifest fall back to a plain single-file fetch below.
async function fetchManifest(
  resolvedUrl: string,
  signal: AbortSignal,
): Promise<AssetManifest | null> {
  try {
    const response = await fetch(`${resolvedUrl}.manifest.json`, { signal })
    if (!response.ok) return null
    return (await response.json()) as AssetManifest
  } catch {
    return null
  }
}

async function fetchChunked(
  resolvedUrl: string,
  manifest: AssetManifest,
  onProgress: (loadedBytes: number) => void,
  signal: AbortSignal,
): Promise<Uint8Array> {
  const merged = new Uint8Array(manifest.totalBytes)
  const offsets: number[] = []
  let offset = 0
  for (const size of manifest.partBytes) {
    offsets.push(offset)
    offset += size
  }

  let loaded = 0
  await Promise.all(
    manifest.parts.map(async (part, i) => {
      const response = await fetch(siblingUrl(resolvedUrl, part), { signal })
      if (!response.ok) {
        throw new Error(`HTTP ${response.status} fetching ${part}`)
      }
      const buffer = new Uint8Array(await response.arrayBuffer())
      merged.set(buffer, offsets[i])
      loaded += buffer.byteLength
      onProgress(loaded)
    }),
  )
  return merged
}

async function readCachedBytes(
  resolvedUrl: string,
): Promise<Uint8Array | null> {
  try {
    const cache = await caches.open(ASSET_CACHE_NAME)
    const cached = await cache.match(resolvedUrl)
    if (!cached) return null
    const buffer = await cached.arrayBuffer()
    return new Uint8Array(buffer)
  } catch {
    return null
  }
}

async function writeCachedBytes(
  resolvedUrl: string,
  bytes: Uint8Array,
): Promise<void> {
  try {
    const cache = await caches.open(ASSET_CACHE_NAME)
    await cache.put(
      resolvedUrl,
      new Response(new Uint8Array(bytes), {
        headers: { 'Content-Type': 'application/octet-stream' },
      }),
    )
  } catch {
    // Cache API may be unavailable (e.g. Playwright, private mode).
  }
}

export function useAssetLoader(url: string): AssetLoaderState {
  const [bytes, setBytes] = useState<Uint8Array | null>(null)
  const [status, setStatus] = useState<AssetStatus>('loading')
  const [loadedBytes, setLoadedBytes] = useState(0)
  const [totalBytes, setTotalBytes] = useState(0)

  useEffect(() => {
    let cancelled = false
    // Aborts this effect's in-flight downloads once it's cleaned up, rather
    // than letting them run to completion only for `cancelled` to discard
    // the bytes. These assets are tens of MB each (fonts, soundfont), and
    // `StrictMode` runs every effect twice in development -- so without
    // this, every page load downloaded each one twice over, holding
    // several of the browser's six per-host connections for seconds and
    // starving everything else the page needed to fetch meanwhile.
    const controller = new AbortController()
    const { signal } = controller
    const resolvedUrl = resolveAssetUrl(url)

    async function load() {
      try {
        const cachedBytes = await readCachedBytes(resolvedUrl)
        if (cachedBytes) {
          if (!cancelled) {
            setBytes(cachedBytes)
            setLoadedBytes(cachedBytes.byteLength)
            setTotalBytes(cachedBytes.byteLength)
            setStatus('ready')
          }
          return
        }

        const manifest = await fetchManifest(resolvedUrl, signal)

        let merged: Uint8Array
        if (manifest) {
          if (!cancelled) setTotalBytes(manifest.totalBytes)
          merged = await fetchChunked(
            resolvedUrl,
            manifest,
            (loaded) => {
              if (!cancelled) setLoadedBytes(loaded)
            },
            signal,
          )
        } else {
          const response = await fetch(resolvedUrl, { signal })
          if (!response.ok) {
            throw new Error(`HTTP ${response.status}`)
          }

          const total = Number(response.headers.get('content-length') ?? 0)
          if (!cancelled) setTotalBytes(total)

          merged = new Uint8Array(await response.arrayBuffer())
        }

        await writeCachedBytes(resolvedUrl, merged)

        if (!cancelled) {
          setBytes(merged)
          setLoadedBytes(merged.byteLength)
          setTotalBytes(merged.byteLength)
          setStatus('ready')
        }
      } catch {
        if (!cancelled) setStatus('error')
      }
    }

    load()
    return () => {
      cancelled = true
      controller.abort()
    }
  }, [url])

  return { bytes, status, loadedBytes, totalBytes }
}
