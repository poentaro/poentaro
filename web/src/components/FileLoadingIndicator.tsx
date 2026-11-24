import { SpinnerLabel } from './SpinnerLabel'

export function FileLoadingIndicator() {
  return (
    <div className="pane-file-loading" role="status">
      <SpinnerLabel pending label="Loading file…" />
    </div>
  )
}
