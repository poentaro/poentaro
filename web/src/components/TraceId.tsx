import { CheckIcon, CopyIcon } from '@radix-ui/react-icons'
import { type CSSProperties, useEffect, useState } from 'react'

const COPIED_FOR_MS = 1500

const rowStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '4px',
  marginTop: '4px',
  fontSize: '12px',
  opacity: 0.8,
}

const codeStyle: CSSProperties = {
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
  userSelect: 'all',
}

const copyButtonStyle: CSSProperties = {
  display: 'inline-flex',
  background: 'none',
  border: 'none',
  padding: '2px',
  cursor: 'pointer',
  color: 'inherit',
}

/**
 * The worker's trace ID for a failed request, with a copy button, so a user
 * can send it and the failure can be looked up in `app_log`. Renders nothing
 * without an ID (the request never reached the worker).
 */
export function TraceId({ traceId }: { traceId: string | null | undefined }) {
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (!copied) return
    const timer = setTimeout(() => setCopied(false), COPIED_FOR_MS)
    return () => clearTimeout(timer)
  }, [copied])

  if (!traceId) return null

  async function copy() {
    if (!traceId) return
    try {
      await navigator.clipboard.writeText(traceId)
      setCopied(true)
    } catch {
      // Clipboard refused (e.g. no permission): the ID stays selectable.
    }
  }

  return (
    <span style={rowStyle} data-testid="trace-id">
      Trace ID <code style={codeStyle}>{traceId}</code>
      <button
        type="button"
        style={copyButtonStyle}
        aria-label={copied ? 'Copied' : 'Copy trace ID'}
        onClick={() => void copy()}
      >
        {copied ? <CheckIcon aria-hidden /> : <CopyIcon aria-hidden />}
      </button>
    </span>
  )
}
