import { withFileExtension } from '../../fileExtension'
import { fileNameWithoutFolder } from '../../hooks/workerHelpers'

export function downloadLocalCopy(fileName: string, content: string): void {
  const name = withFileExtension(fileNameWithoutFolder(fileName))
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = name
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
  URL.revokeObjectURL(url)
}
