Feature: Synced Share sign-in

  # Covers signing in from the share modal's prompt and the full-screen
  # session-failure dialog -- an account is required to create or own a
  # Synced Share. The rest of the Synced Share scenarios
  # (synced-share-button.feature) seed a signed-in session instead of
  # exercising this UI, to keep their focus on the sync/viewer behavior.
  # Every scenario opens the share modal's Synced-link tab first, via the
  # shared "the owner loads the app and clicks 'Sync'" step (see
  # synced-share-button.steps.ts's `openSyncedTab`). Scenarios that go on to
  # start a share seed a cloud file first, since only a cloud file can be
  # shared live.

  Scenario: Clicking Sync while signed out shows a sign-in prompt, and signing in from it lets a second click start syncing
    Given clipboard permissions are granted
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in" in the prompt
    And I enter the email "e2e-test-user@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the identity row shows signed in as "e2e-test-user@poentaro.test"
    When the owner reopens the app on the seeded synced file
    And the owner clicks "Sync" again
    Then the synced link is copied
    And the synced share identity row reads "Synced as e2e-test-user@poentaro.test"

  Scenario: Signing out disconnects the account and stops the sync
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When the owner closes the share modal
    And the owner clicks the account chip
    And the owner clicks "Sign out" in the profile popover
    When the owner reopens the share modal
    Then the sign-in prompt is shown

  Scenario: Signing out on one device keeps the owner signed in and syncing on their other devices
    Given the file store is seeded with the synced score for "e2e-test-user-multi-device@poentaro.test"
    And the owner signs in as "e2e-test-user-multi-device@poentaro.test" on device "B"
    And the owner starts syncing the seeded file on device "B"
    And the owner signs in as "e2e-test-user-multi-device@poentaro.test" on device "A"
    When the owner signs out on device "A"
    And the owner reopens the seeded file on device "B"
    Then device "B" is still signed in and syncing the seeded file

  Scenario: A rejected session while starting a share shows the full-screen error dialog, with no automatic retry
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    And the Synced Share worker rejects the next start-share request with a verification failure
    When the owner loads the app and clicks "Sync"
    Then the synced share error dialog is shown with a "file a GitHub issue" link
    And the synced share error dialog shows the reason "session expired"
    When the owner dismisses the synced share error dialog
    Then the synced share error dialog is gone
    When the owner reopens the share modal
    Then the sign-in prompt is shown

  Scenario: A rejected session clears the stale sign-in, so retrying prompts a fresh sign-in instead of repeating the same failure
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    And the Synced Share worker rejects the next start-share request with a verification failure
    When the owner loads the app and clicks "Sync"
    Then the synced share error dialog is shown with a "file a GitHub issue" link
    When the owner dismisses the synced share error dialog
    When the owner reopens the share modal
    Then the sign-in prompt is shown
    When the owner clicks "Sign in" in the prompt
    And I enter the email "e2e-test-user@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the identity row shows signed in as "e2e-test-user@poentaro.test"
