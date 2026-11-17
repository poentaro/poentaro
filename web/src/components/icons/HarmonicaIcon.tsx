/** A harmonica seen from the front: a long rounded body with a row
 * of hole slots. Drawn in lucide-react's style (24×24 viewBox,
 * `currentColor` 2px round strokes) since lucide has no harmonica icon —
 * used by the notation switcher's "Harmonica" option. */
export function HarmonicaIcon({ size = 24 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="2" y="8" width="20" height="8" rx="2" />
      <path d="M6 11v2M10 11v2M14 11v2M18 11v2" />
    </svg>
  )
}
