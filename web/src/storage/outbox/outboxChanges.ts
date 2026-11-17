import type { OutboxChangeChannel } from './outboxLooper'

/** Tabs of one account tell each other when they change the stored queue,
 * so the leader tab delivers messages queued from a follower tab. Without
 * `BroadcastChannel` (or in tests) there is nobody to tell. */
export function broadcastChanges(
  account: string,
): OutboxChangeChannel | undefined {
  if (typeof BroadcastChannel === 'undefined') return undefined
  const name = `poentaro-outbox:${account}`
  // Channels of the same name in one tab hear each other, so every message
  // carries its sender and a tab ignores its own.
  const tabId = crypto.randomUUID()
  const sender = new BroadcastChannel(name)
  return {
    announce: () => sender.postMessage(tabId),
    listen: (onChange) => {
      const receiver = new BroadcastChannel(name)
      receiver.onmessage = (event) => {
        if (event.data !== tabId) onChange()
      }
      return () => receiver.close()
    },
  }
}
