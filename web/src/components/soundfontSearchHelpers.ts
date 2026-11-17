import type { SoundChoice } from '../poentaroWasm'
import type {
  InstrumentArticulation,
  InstrumentCategory,
  InstrumentEntry,
  InstrumentRole,
  InstrumentSource,
} from '../utils/gmInstruments'

export type ActiveTag =
  | { kind: 'category'; value: InstrumentCategory }
  | { kind: 'source'; value: InstrumentSource }
  | { kind: 'role'; value: InstrumentRole }
  | { kind: 'articulation'; value: InstrumentArticulation }

export function tagKey(tag: ActiveTag): string {
  return `${tag.kind}:${tag.value}`
}

export function fuzzyScore(query: string, target: string): number {
  const q = query.toLowerCase()
  const t = target.toLowerCase()
  if (t.includes(q)) return 1000
  let score = 0
  let qi = 0
  let consecutive = 0
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      score += 1 + consecutive * 2
      consecutive++
      qi++
    } else {
      consecutive = 0
    }
  }
  return qi === q.length ? score : 0
}

export function instrumentFuzzyScore(
  query: string,
  instrument: InstrumentEntry,
  label: string,
): number {
  return Math.max(
    fuzzyScore(query, label),
    fuzzyScore(query, instrument.category),
    fuzzyScore(query, instrument.source),
    fuzzyScore(query, instrument.role),
    fuzzyScore(query, instrument.articulation),
  )
}

export function percussionFuzzyScore(
  query: string,
  choice: SoundChoice,
): number {
  return fuzzyScore(query, choice.label)
}
