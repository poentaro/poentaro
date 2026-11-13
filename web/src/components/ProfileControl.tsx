import {
  ExitIcon,
  LockClosedIcon,
  MagicWandIcon,
  PersonIcon,
} from '@radix-ui/react-icons'
import * as Popover from '@radix-ui/react-popover'
import { useCallback, useState } from 'react'
import './ProfileControl.css'

/**
 * The header's persistent account control (see `HANDOFF-account-chip.md`)
 * -- the only glanceable indicator anywhere in the app that the user is
 * signed in, and the only place that signs out or changes the password.
 * `ShareModal`'s Synced-link tab and `StorageSettingsModal` still offer
 * "Sign in" as a shortcut (opening the same account dialog as this
 * component's own button), but neither renders a sign-out control -- both
 * just show a plain status line once signed in.
 */
export interface ProfileControlProps {
  /** Whether this browser has an account session -- see
   * `useSyncedShareOwner.ts`. The same account backs both cloud storage
   * and Synced Share, so this one chip covers both. */
  isSignedIn: boolean
  /** The signed-in account's email; `null` while signed out. */
  accountEmail: string | null
  /** Drives the chip's small live dot -- true while the active file is
   * currently syncing via Synced Share. */
  isSynced: boolean
  /** Opens the account dialog (`AccountAuthModal`) -- the same one
   * `ShareModal` and `StorageSettingsModal` open as their sign-in
   * shortcut. */
  onSignIn: () => void
  /** Opens the change password dialog. */
  onChangePassword: () => void
  /** Opens the dialog that connects an AI agent with an API key. */
  onOpenApiKeys: () => void
  /** The only sign-out action in the app -- stops any active sync, ends
   * this browser's session, and falls back to local storage if cloud
   * storage was active (see `App.tsx`'s `handleAccountSignOut`). */
  onSignOut: () => void
}

export function ProfileControl({
  isSignedIn,
  accountEmail,
  isSynced,
  onSignIn,
  onChangePassword,
  onOpenApiKeys,
  onSignOut,
}: ProfileControlProps) {
  const [popoverOpen, setPopoverOpen] = useState(false)

  const handleChangePassword = useCallback(() => {
    setPopoverOpen(false)
    onChangePassword()
  }, [onChangePassword])

  const handleOpenApiKeys = useCallback(() => {
    setPopoverOpen(false)
    onOpenApiKeys()
  }, [onOpenApiKeys])

  const handleSignOut = useCallback(() => {
    setPopoverOpen(false)
    onSignOut()
  }, [onSignOut])

  if (!isSignedIn) {
    return (
      <button
        type="button"
        className="preview-export-btn"
        data-testid="account-chip-sign-in"
        onClick={onSignIn}
      >
        <PersonIcon aria-hidden="true" />
        Sign in
      </button>
    )
  }

  return (
    <Popover.Root open={popoverOpen} onOpenChange={setPopoverOpen}>
      <Popover.Trigger asChild>
        <button
          type="button"
          className="account-chip"
          data-testid="account-chip"
        >
          <PersonIcon aria-hidden="true" />
          {accountEmail}
          {isSynced && (
            <span
              className="account-chip-live-dot"
              title="Currently syncing"
              data-testid="account-chip-syncing-indicator"
            />
          )}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className="account-popover"
          data-testid="account-popover"
          align="end"
          sideOffset={8}
        >
          <div className="account-popover-head">
            <span className="account-popover-login">{accountEmail}</span>
            <span className="account-popover-sub">
              {isSynced ? 'Signed in · syncing this file' : 'Signed in'}
            </span>
          </div>
          <button
            type="button"
            className="account-popover-link"
            onClick={handleChangePassword}
          >
            <LockClosedIcon aria-hidden="true" />
            Change password
          </button>
          <button
            type="button"
            className="account-popover-link"
            onClick={handleOpenApiKeys}
          >
            <MagicWandIcon aria-hidden="true" />
            AI agents
          </button>
          <button
            type="button"
            className="account-popover-signout"
            onClick={handleSignOut}
          >
            <ExitIcon aria-hidden="true" />
            Sign out
          </button>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  )
}
