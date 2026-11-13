import { useCallback, useEffect, useRef, useState } from 'react'
import { useAccountAuth } from '../storage/accountAuth'
import {
  type AccountResult,
  type ApiKeySummary,
  createApiKey,
  listApiKeys,
  revokeApiKey,
} from '../storage/accountRequests'
import {
  AccountDialogFrame,
  ErrorBanner,
  footnoteStyle,
} from './accountDialogParts'
import { RevokeApiKeyDialog } from './RevokeApiKeyDialog'
import { SpinnerLabel } from './SpinnerLabel'
import {
  bodyStyle,
  buttonStyle,
  primaryButtonStyle,
} from './storageSettingsModalStyles'

export interface ApiKeysModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const promptStyle: React.CSSProperties = {
  margin: 0,
  padding: '8px 10px',
  border: '1px solid #cbd5e0',
  borderRadius: '4px',
  background: '#f8fafc',
  fontSize: '12px',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'anywhere',
  maxHeight: '40vh',
  overflowY: 'auto',
}

const keyRowStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '8px',
  fontSize: '13px',
}

const keyUseStyle: React.CSSProperties = {
  flex: 1,
  color: '#555',
  fontSize: '12px',
}

function day(millis: number): string {
  return new Date(millis).toLocaleDateString()
}

function keyStatus(key: ApiKeySummary): string {
  if (key.expired) return 'Expired'
  const used =
    key.lastUsedAt === null || key.lastUsedAt === undefined
      ? 'Never used'
      : `Used ${day(key.lastUsedAt)}`
  return `${used} · Expires ${day(key.expiresAt)}`
}

/**
 * Lets the signed-in owner's AI agent edit their scores: creates an API key
 * and shows, once, the prompt that carries it, and lists and revokes keys.
 * The worker writes the prompt, so the host in it is always the real one.
 */
export function ApiKeysModal({ open, onOpenChange }: ApiKeysModalProps) {
  const [accountAuth, setAccountAuth] = useAccountAuth()
  const [keys, setKeys] = useState<ApiKeySummary[]>([])
  const [prompt, setPrompt] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [traceId, setTraceId] = useState<string | null>(null)
  const [revoking, setRevoking] = useState<ApiKeySummary | null>(null)
  const token = accountAuth?.token ?? null
  // The parent passes a new `onOpenChange` on every render. Read through a
  // ref, it doesn't make the dialog reload its keys or clear its prompt.
  const onOpenChangeRef = useRef(onOpenChange)
  useEffect(() => {
    onOpenChangeRef.current = onOpenChange
  })

  /** The value of `result`, or `null` after showing why it failed. */
  const settle = useCallback(
    <T,>(result: AccountResult<T>): T | null => {
      if (result.ok) return result.value
      setTraceId(result.traceId)
      if (result.failure.kind === 'session-ended') {
        setAccountAuth(null)
        onOpenChangeRef.current(false)
      } else if (result.failure.kind === 'message') {
        setError(result.failure.message)
      }
      return null
    },
    [setAccountAuth],
  )

  useEffect(() => {
    if (!open || !token) return
    setPrompt(null)
    setCopied(false)
    setError(null)
    setTraceId(null)
    void listApiKeys(token).then((result) => {
      const listed = settle(result)
      if (listed) setKeys(listed)
    })
  }, [open, token, settle])

  async function create() {
    if (!token || busy) return
    setBusy(true)
    setError(null)
    const created = settle(await createApiKey(token))
    setBusy(false)
    if (!created) return
    setPrompt(created.prompt)
    setCopied(false)
    setKeys((current) => [created.key, ...current])
  }

  async function revoke(id: string) {
    setRevoking(null)
    if (!token) return
    setError(null)
    // A revoke succeeds with no value, so `settle`'s `null` can't tell.
    const revoked = await revokeApiKey(token, id)
    settle(revoked)
    if (!revoked.ok) return
    setKeys((current) => current.filter((key) => key.id !== id))
  }

  async function copyPrompt() {
    if (!prompt) return
    try {
      await navigator.clipboard.writeText(prompt)
      setCopied(true)
    } catch {
      window.prompt('Copy this into your AI agent:', prompt)
    }
  }

  return (
    <AccountDialogFrame
      open={open}
      onOpenChange={onOpenChange}
      title="AI agents"
      testId="api-keys-modal"
    >
      <div style={bodyStyle}>
        {error && <ErrorBanner traceId={traceId}>{error}</ErrorBanner>}
        {prompt ? (
          <>
            <pre style={promptStyle} data-testid="api-key-prompt">
              {prompt}
            </pre>
            <button
              type="button"
              style={primaryButtonStyle}
              onClick={() => void copyPrompt()}
            >
              {copied ? 'Copied' : 'Copy prompt'}
            </button>
            <p style={footnoteStyle}>
              Paste it into your AI agent. Its key isn't shown again.
            </p>
          </>
        ) : (
          <button
            type="button"
            style={primaryButtonStyle}
            disabled={busy}
            onClick={() => void create()}
          >
            <SpinnerLabel pending={busy} label="Create key" />
          </button>
        )}
        {keys.map((key) => (
          <div key={key.id} style={keyRowStyle} data-testid="api-key-row">
            <code>{key.prefix}…</code>
            <span style={keyUseStyle}>{keyStatus(key)}</span>
            <button
              type="button"
              style={buttonStyle}
              onClick={() => setRevoking(key)}
            >
              Revoke
            </button>
          </div>
        ))}
      </div>
      <RevokeApiKeyDialog
        prefix={revoking?.prefix ?? null}
        onCancel={() => setRevoking(null)}
        onConfirm={() => revoking && void revoke(revoking.id)}
      />
    </AccountDialogFrame>
  )
}
