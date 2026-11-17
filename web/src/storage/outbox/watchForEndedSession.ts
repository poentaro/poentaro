import type { OutboxLooper, OutboxSnapshot } from './outboxLooper'

const needsSignIn = (snapshot: OutboxSnapshot) =>
  snapshot.summary.lanes.some((lane) => lane.reason === 'needs-sign-in')

/** Calls `onSessionEnded` each time a lane stops for sign-in, i.e. the
 * worker refused a queued change because this session has ended. Lanes
 * already stopped when watching starts are an earlier session's, which this
 * one picks up, so they don't count. Returns the unsubscribe. */
export function watchForEndedSession(
  looper: OutboxLooper,
  onSessionEnded: () => void,
): () => void {
  let ended = needsSignIn(looper.snapshot())
  return looper.subscribe((snapshot) => {
    const nowEnded = needsSignIn(snapshot)
    if (nowEnded && !ended) onSessionEnded()
    ended = nowEnded
  })
}
