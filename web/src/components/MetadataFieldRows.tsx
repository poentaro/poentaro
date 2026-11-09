import { ResetIcon } from '@radix-ui/react-icons'
import { FieldLabel } from './FieldHelpModal'

export const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}

export const inputStyle: React.CSSProperties = {
  fontSize: '12px',
  fontFamily: 'var(--mono, monospace)',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  padding: '2px 6px',
  width: '100%',
  boxSizing: 'border-box',
}

const numberStepperWrapperStyle: React.CSSProperties = {
  display: 'flex',
  width: '100%',
}

const numberStepperInputStyle: React.CSSProperties = {
  ...inputStyle,
  minWidth: 0,
  flex: 1,
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
}

const numberStepperButtonsStyle: React.CSSProperties = {
  display: 'flex',
  flexShrink: 0,
}

const numberStepperButtonStyle: React.CSSProperties = {
  width: '18px',
  height: '20px',
  lineHeight: 1,
  fontSize: '12px',
  padding: 0,
  border: '1px solid #cbd5e0',
  borderLeft: 'none',
  color: '#444',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}

interface NumberStepperProps {
  value: number | ''
  defaultValue: number | null | undefined
  min: number
  step?: number
  'aria-label': string
  placeholder?: string
  onChange: (value: string) => void
}

/** `[input][-][+]` numeric field: always-visible stepper buttons (native
 * spinner arrows are hover-only in some browsers), and stepping from an
 * empty field applies the field's default rather than starting from `0`
 * (native `<input type="number">` always steps from `0` when empty). See
 * `HANDOFF-text-style-metadata.md`-adjacent context: `defaultValue` is the
 * same value shown today as the input's greyed-out `placeholder`. */
export function NumberStepper({
  value,
  defaultValue,
  min,
  step = 1,
  'aria-label': ariaLabel,
  placeholder,
  onChange,
}: NumberStepperProps) {
  const stepBy = (delta: number) => {
    const base = value === '' ? (defaultValue ?? min) : value
    onChange(String(Math.max(min, base + delta)))
  }
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      stepBy(step)
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      stepBy(-step)
    }
  }
  return (
    <div style={numberStepperWrapperStyle}>
      <input
        type="number"
        min={min}
        aria-label={ariaLabel}
        placeholder={placeholder}
        style={numberStepperInputStyle}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
      />
      <div style={numberStepperButtonsStyle}>
        <button
          type="button"
          className="number-stepper-button"
          aria-label={`${ariaLabel} decrease`}
          style={numberStepperButtonStyle}
          onClick={() => stepBy(-step)}
        >
          −
        </button>
        <button
          type="button"
          className="number-stepper-button"
          aria-label={`${ariaLabel} increase`}
          style={{ ...numberStepperButtonStyle, borderLeft: 'none' }}
          onClick={() => stepBy(step)}
        >
          +
        </button>
      </div>
    </div>
  )
}

export interface FieldRowProps {
  label: string
  help: string
  onShowHelp: (label: string, help: string) => void
  /** Set while a preset overrides this field: clears the override, so the
   * field falls back to the song's `metadata`. */
  onReset?: () => void
}

const valueCellStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '4px',
}

const resetButtonStyle: React.CSSProperties = {
  flexShrink: 0,
  width: '20px',
  height: '20px',
  padding: 0,
  border: 'none',
  background: 'none',
  color: '#666',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}

/** Inherited from `metadata` rather than set by the preset. */
export const inheritedStyle: React.CSSProperties = { opacity: 0.45 }

export function ResetOverrideButton({
  label,
  onReset,
}: {
  label: string
  onReset?: () => void
}) {
  if (!onReset) return null
  return (
    <button
      type="button"
      aria-label={`Reset ${label}`}
      title={`Reset ${label}`}
      style={resetButtonStyle}
      onClick={onReset}
    >
      <ResetIcon />
    </button>
  )
}

export function TextFieldRow({
  label,
  help,
  onShowHelp,
  onReset,
  value,
  placeholder,
  onChange,
}: FieldRowProps & {
  value: string
  placeholder?: string
  onChange: (value: string) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <div style={valueCellStyle}>
          <input
            type="text"
            aria-label={label}
            placeholder={placeholder}
            style={inputStyle}
            value={value}
            onChange={(e) => onChange(e.target.value)}
          />
          <ResetOverrideButton label={label} onReset={onReset} />
        </div>
      </td>
    </tr>
  )
}

export function NumberFieldRow({
  label,
  help,
  onShowHelp,
  onReset,
  value,
  placeholder,
  onChange,
}: FieldRowProps & {
  value: number | ''
  placeholder?: string
  onChange: (value: string) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <div style={valueCellStyle}>
          <NumberStepper
            value={value}
            defaultValue={placeholder != null ? Number(placeholder) : null}
            min={1}
            aria-label={label}
            placeholder={placeholder}
            onChange={onChange}
          />
          <ResetOverrideButton label={label} onReset={onReset} />
        </div>
      </td>
    </tr>
  )
}

export function CheckboxFieldRow({
  label,
  help,
  onShowHelp,
  onReset,
  checked,
  inherited = false,
  onChange,
}: FieldRowProps & {
  checked: boolean
  inherited?: boolean
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <div style={valueCellStyle}>
          <input
            type="checkbox"
            aria-label={label}
            checked={checked}
            style={inherited ? inheritedStyle : undefined}
            onChange={onChange}
          />
          <ResetOverrideButton label={label} onReset={onReset} />
        </div>
      </td>
    </tr>
  )
}
