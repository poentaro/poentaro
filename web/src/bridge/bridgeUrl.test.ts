import { describe, expect, it } from 'vitest'
import { bridgeRedirectUrl } from './bridgeUrl'

const TARGET = 'https://poentaro.pages.dev'

describe('bridgeRedirectUrl', () => {
  it('keeps the path, query and fragment of a share link', () => {
    const current = new URL(
      'https://jianpu-generator.pages.dev/?s=abcdefghijk#payload=xyz',
    )

    expect(bridgeRedirectUrl(current, '/', TARGET, null)).toBe(
      'https://poentaro.pages.dev/?s=abcdefghijk#payload=xyz',
    )
  })

  it('adds the handoff id next to the existing query', () => {
    const current = new URL('https://jianpu-generator.pages.dev/?file=a#h')

    expect(bridgeRedirectUrl(current, '/', TARGET, 'id_1')).toBe(
      'https://poentaro.pages.dev/?file=a&migrate=id_1#h',
    )
  })

  it("drops GitHub Pages' base path", () => {
    const current = new URL(
      'https://wongjiahau.github.io/jianpu-generator/?s=abc',
    )

    expect(bridgeRedirectUrl(current, '/jianpu-generator/', TARGET, null)).toBe(
      'https://poentaro.pages.dev/?s=abc',
    )
  })

  it('keeps a path outside the base path as it is', () => {
    const current = new URL('https://wongjiahau.github.io/elsewhere')

    expect(bridgeRedirectUrl(current, '/jianpu-generator/', TARGET, null)).toBe(
      'https://poentaro.pages.dev/elsewhere',
    )
  })
})
