import { describe, expect, it } from 'vitest'
import { shouldWarnBeforeUnload } from './outboxSaveBadge'
import { shouldScheduleAutosave } from './useStorageBackend'

describe('shouldScheduleAutosave', () => {
  it('never schedules a save for the local backend', () => {
    expect(
      shouldScheduleAutosave(
        'local',
        { active: 'a.poentaro', content: 'old' },
        { active: 'a.poentaro', content: 'new' },
      ),
    ).toBe(false)
  })

  it('does not schedule on the very first render (no previous snapshot)', () => {
    expect(
      shouldScheduleAutosave('cloud', null, {
        active: 'a.poentaro',
        content: 'new',
      }),
    ).toBe(false)
  })

  it('does not schedule when switching the active file, even though content differs', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.poentaro', content: 'content of a' },
        { active: 'b.poentaro', content: 'content of b' },
      ),
    ).toBe(false)
  })

  it('does not schedule when the same active file has unchanged content', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.poentaro', content: 'same' },
        { active: 'a.poentaro', content: 'same' },
      ),
    ).toBe(false)
  })

  it('schedules a save when the same active file has edited content on the cloud backend', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.poentaro', content: 'old' },
        { active: 'a.poentaro', content: 'new' },
      ),
    ).toBe(true)
  })
})

describe('shouldWarnBeforeUnload', () => {
  it('never warns for the local backend', () => {
    expect(shouldWarnBeforeUnload('local', true, true)).toBe(false)
  })

  it('does not warn on the cloud backend when nothing is pending or unsynced', () => {
    expect(shouldWarnBeforeUnload('cloud', false, false)).toBe(false)
  })

  it('warns on the cloud backend while a debounced save is still armed', () => {
    expect(shouldWarnBeforeUnload('cloud', true, false)).toBe(true)
  })

  it('warns on the cloud backend while the outbox has unsynced work', () => {
    expect(shouldWarnBeforeUnload('cloud', false, true)).toBe(true)
  })
})
