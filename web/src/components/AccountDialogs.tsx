import type { AccountDialog } from '../hooks/useSyncedShareOwner'
import { AccountAuthModal } from './AccountAuthModal'
import { ApiKeysModal } from './ApiKeysModal'
import { ChangePasswordModal } from './ChangePasswordModal'

/** The account dialog `useSyncedShareOwner` has open, if any. */
export function AccountDialogs({
  open,
  onClose,
}: {
  open: AccountDialog | null
  onClose: () => void
}) {
  const closeWhenHidden = (visible: boolean) => {
    if (!visible) onClose()
  }
  return (
    <>
      <AccountAuthModal
        open={open === 'sign-in'}
        onOpenChange={closeWhenHidden}
      />
      <ChangePasswordModal
        open={open === 'change-password'}
        onOpenChange={closeWhenHidden}
      />
      <ApiKeysModal open={open === 'api-keys'} onOpenChange={closeWhenHidden} />
    </>
  )
}
