import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import type { HistoryRow } from '../../poentaroWasm'
import { EMPTY_FILTER_STATE } from './historyFilterState'
import { buildRowReport } from './historyLabels'
import { OutboxHistoryView } from './OutboxHistoryView'

function row(overrides: Partial<HistoryRow> = {}): HistoryRow {
  return {
    messageId: 'm1',
    fileId: 'f1',
    fileName: 'Ode to Joy.poentaro',
    kind: 'edit',
    status: 'sent',
    sizeBytes: 2048,
    atMs: 1_700_000_000_000n,
    attempts: [],
    revision: 3n,
    payloadJson: '{\n  "content": "3 3 4 5"\n}',
    payloadMatchesQuery: false,
    ...overrides,
  }
}

function render(rows: HistoryRow[], query = ''): string {
  return renderToStaticMarkup(
    <OutboxHistoryView
      rows={rows}
      totalCount={5}
      files={[{ fileId: 'f1', name: 'Ode to Joy.poentaro' }]}
      filter={{ ...EMPTY_FILTER_STATE, query }}
      onFilterChange={vi.fn()}
    />,
  )
}

describe('OutboxHistoryView', () => {
  it('lists each message with its status, size and a per-row copy button', () => {
    const html = render([row()])
    expect(html).toContain('Ode to Joy.poentaro')
    expect(html).toContain('Sent')
    expect(html).toContain('2.0 KB')
    expect(html).toContain('Copy report')
    expect(html).toContain('1 of 5 messages')
  })

  it('keeps payloads closed until a row is opened', () => {
    expect(render([row()])).not.toContain('Copy payload')
  })

  it('opens a row whose payload contains the search text and highlights it', () => {
    const html = render([row({ payloadMatchesQuery: true })], '3 3 4')
    expect(html).toContain('Copy payload')
    expect(html).toContain('<mark>3 3 4</mark>')
  })

  it('highlights the match in the file name', () => {
    expect(render([row()], 'joy')).toContain('<mark>Joy</mark>')
  })

  it('says so when nothing matches', () => {
    expect(render([])).toContain('No messages match.')
  })
})

describe('buildRowReport', () => {
  it('describes status and attempts but never the payload', () => {
    const report = buildRowReport(
      row({
        status: 'retrying',
        attempts: [
          {
            atMs: 1_700_000_000_000n,
            outcome: { tag: 'transient', val: { reason: 'offline' } },
          },
        ],
      }),
    )
    expect(report).toContain('Status: Retrying')
    expect(report).toContain('Network problem: offline')
    expect(report).not.toContain('3 3 4 5')
  })
})
