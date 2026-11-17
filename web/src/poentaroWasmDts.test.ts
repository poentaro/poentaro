/// <reference types="node" />
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

// jco names each variant case's interface `<Variant><Case>` — the same name
// a WIT `record <variant>-<case>` gets. When both exist, TypeScript merges
// the two declarations into one type that demands the record's fields *and*
// `{ tag, val }`, which no plain value satisfies, so consumers could only
// build that variant through a cast. Every interface in the generated
// `.d.ts` must therefore be declared exactly once.
describe('jco-generated poentaro_wasm.d.ts', () => {
  it('declares every interface exactly once', () => {
    const dts = readFileSync(
      fileURLToPath(
        new URL(
          '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.d.ts',
          import.meta.url,
        ),
      ),
      'utf-8',
    )
    const names = [...dts.matchAll(/^export interface (\w+)/gm)].map(
      (match) => match[1],
    )
    const duplicates = names.filter(
      (name, index) => names.indexOf(name) !== index,
    )
    expect(names.length).toBeGreaterThan(0)
    expect(duplicates).toEqual([])
  })
})
