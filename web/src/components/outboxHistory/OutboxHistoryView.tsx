import { type ReactElement, useState } from 'react'
import type { HistoryRow } from '../../poentaroWasm'
import { copyDiagnostics } from '../syncPanel/copyDiagnostics'
import { Highlighted } from './Highlighted'
import {
  AGE_OPTIONS,
  EMPTY_FILTER_STATE,
  type HistoryFilterState,
  isFilterActive,
  KIND_OPTIONS,
  SIZE_OPTIONS,
  STATUS_OPTIONS,
  TRIES_OPTIONS,
} from './historyFilterState'
import {
  attemptLine,
  buildRowReport,
  KIND_LABEL,
  STATUS_LABEL,
  sizeLabel,
  timeLabel,
} from './historyLabels'
import './OutboxHistory.css'

export interface OutboxHistoryViewProps {
  rows: HistoryRow[]
  /** How many messages exist before filtering. */
  totalCount: number
  files: ReadonlyArray<{ fileId: string; name: string }>
  filter: HistoryFilterState
  onFilterChange(next: HistoryFilterState): void
}

type SelectField = Exclude<keyof HistoryFilterState, 'query'>

function FilterSelect(props: {
  label: string
  field: SelectField
  anyLabel: string
  options: ReadonlyArray<{ value: string; label: string }>
  filter: HistoryFilterState
  onFilterChange(next: HistoryFilterState): void
}): ReactElement {
  const id = `outbox-history-${props.field}`
  return (
    <label className="outbox-history__field" htmlFor={id}>
      {props.label}
      <select
        id={id}
        value={props.filter[props.field]}
        onChange={(event) =>
          props.onFilterChange({
            ...props.filter,
            [props.field]: event.target.value,
          })
        }
      >
        <option value="">{props.anyLabel}</option>
        {props.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  )
}

function CopyButton(props: {
  label: string
  title?: string
  text: string
}): ReactElement {
  const [done, setDone] = useState(false)
  return (
    <button
      type="button"
      className="outbox-history__small-button"
      title={props.title}
      onClick={(event) => {
        event.stopPropagation()
        void copyDiagnostics(props.text).then(() => {
          setDone(true)
          setTimeout(() => setDone(false), 1500)
        })
      }}
    >
      {done ? 'Copied' : props.label}
    </button>
  )
}

function DetailRow(props: { row: HistoryRow; query: string }): ReactElement {
  const { row } = props
  return (
    <tr className="outbox-history__detail">
      <td colSpan={7}>
        {row.attempts.length === 0 ? (
          <div className="outbox-history__attempt">Not sent yet</div>
        ) : (
          row.attempts.map((attempt) => (
            <div
              className="outbox-history__attempt"
              key={`${attempt.atMs}-${attempt.outcome.tag}`}
            >
              {attemptLine(attempt)}
            </div>
          ))
        )}
        {row.retryInMs !== undefined && (
          <div className="outbox-history__attempt">
            Next try in {Math.ceil(Number(row.retryInMs) / 1000)} s
          </div>
        )}
        <div className="outbox-history__payload-head">
          <CopyButton label="Copy payload" text={row.payloadJson} />
        </div>
        <pre className="outbox-history__payload">
          <Highlighted text={row.payloadJson} query={props.query} />
        </pre>
      </td>
    </tr>
  )
}

function HistoryTableRow(props: {
  row: HistoryRow
  query: string
  open: boolean
  onToggle(): void
}): ReactElement {
  const { row } = props
  return (
    <>
      <tr
        className="outbox-history__row"
        data-testid={`outbox-history-row-${row.messageId}`}
        data-status={row.status}
        onClick={props.onToggle}
      >
        <td>
          <button
            type="button"
            className="outbox-history__toggle"
            aria-expanded={props.open}
            aria-label={`${props.open ? 'Hide' : 'Show'} details of ${KIND_LABEL[row.kind]} on ${row.fileName}`}
          >
            <span aria-hidden="true">{props.open ? '▾' : '▸'}</span>
            <span
              className={`outbox-history__pill outbox-history__pill--${row.status}`}
            >
              {STATUS_LABEL[row.status]}
            </span>
          </button>
        </td>
        <td className="outbox-history__file">
          <Highlighted text={row.fileName} query={props.query} />
        </td>
        <td>{KIND_LABEL[row.kind]}</td>
        <td className="outbox-history__number">{sizeLabel(row.sizeBytes)}</td>
        <td className="outbox-history__number">{row.attempts.length}</td>
        <td className="outbox-history__time">{timeLabel(row.atMs)}</td>
        <td className="outbox-history__actions">
          <CopyButton
            label="Copy report"
            title="Copy status and attempts, without the payload"
            text={buildRowReport(row)}
          />
        </td>
      </tr>
      {props.open && <DetailRow row={row} query={props.query} />}
    </>
  )
}

export function OutboxHistoryView(props: OutboxHistoryViewProps): ReactElement {
  const { filter, onFilterChange } = props
  const [opened, setOpened] = useState<ReadonlySet<string>>(new Set())
  const toggle = (messageId: string) =>
    setOpened((current) => {
      const next = new Set(current)
      if (!next.delete(messageId)) next.add(messageId)
      return next
    })
  const select = {
    filter,
    onFilterChange,
  }
  return (
    <div className="outbox-history">
      <div className="outbox-history__search">
        <input
          type="search"
          aria-label="Search file name or payload"
          placeholder="Search file name or payload"
          value={filter.query}
          onChange={(event) =>
            onFilterChange({ ...filter, query: event.target.value })
          }
        />
        {isFilterActive(filter) && (
          <button
            type="button"
            onClick={() => onFilterChange(EMPTY_FILTER_STATE)}
          >
            Clear filters
          </button>
        )}
      </div>
      <div className="outbox-history__filters">
        <FilterSelect
          {...select}
          label="Status"
          field="status"
          anyLabel="Any"
          options={STATUS_OPTIONS}
        />
        <FilterSelect
          {...select}
          label="Kind"
          field="kind"
          anyLabel="Any"
          options={KIND_OPTIONS}
        />
        <FilterSelect
          {...select}
          label="File"
          field="fileId"
          anyLabel="Any"
          options={props.files.map((file) => ({
            value: file.fileId,
            label: file.name,
          }))}
        />
        <FilterSelect
          {...select}
          label="When"
          field="age"
          anyLabel="Any time"
          options={AGE_OPTIONS}
        />
        <FilterSelect
          {...select}
          label="Size"
          field="size"
          anyLabel="Any"
          options={SIZE_OPTIONS}
        />
        <FilterSelect
          {...select}
          label="Tries"
          field="tries"
          anyLabel="Any"
          options={TRIES_OPTIONS}
        />
      </div>
      <div className="outbox-history__count" aria-live="polite">
        {props.rows.length} of {props.totalCount} messages
      </div>
      <div className="outbox-history__table-wrap">
        <table className="outbox-history__table">
          <thead>
            <tr>
              <th>Status</th>
              <th>File</th>
              <th>Kind</th>
              <th className="outbox-history__number">Size</th>
              <th className="outbox-history__number">Tries</th>
              <th>Time</th>
              <th>
                <span className="outbox-history__visually-hidden">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {props.rows.length === 0 && (
              <tr>
                <td colSpan={7} className="outbox-history__empty">
                  No messages match.
                </td>
              </tr>
            )}
            {props.rows.map((row) => (
              <HistoryTableRow
                key={row.messageId}
                row={row}
                query={filter.query}
                open={opened.has(row.messageId) || row.payloadMatchesQuery}
                onToggle={() => toggle(row.messageId)}
              />
            ))}
          </tbody>
        </table>
      </div>
      <p className="outbox-history__footnote">
        Sent messages are kept for 7 days, up to 200 messages or 5 MB.
      </p>
    </div>
  )
}
