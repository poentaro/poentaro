Feature: Idempotent Synced Share creation for cloud-backed files

  # A share is a pointer to one D1 `files` row (crates/live-share-worker's
  # `shares.file_id`, unique), and only that file's owner can start it --
  # so re-sharing the same file always resolves to the same link, even from
  # a different browser context, and the live/stopped state is read from
  # the server rather than a per-browser cache. Only cloud files can be
  # shared live at all. Each seeded file name gets a per-scenario suffix
  # (see `idempotentCreateState.actualNames`), so parallel scenarios never
  # share, or rename, each other's file.

  Background:
    Given clipboard permissions are granted

  Scenario: A second browser context signed in as the same account already shows the file's live link
    Given a cloud file named "idempotent.poentaro" is seeded for idempotent sharing
    And the owner is signed in as "e2e-test-user@poentaro.test"
    When the app loads the cloud-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a separate browser context loads the same cloud-backed file, signed in as the same account "e2e-test-user@poentaro.test"
    Then the separate browser context's share modal already shows the stop-sync button
    When the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context is identical to the original link

  Scenario: A different account cannot start a share on someone else's cloud file
    Given a cloud file named "idempotent.poentaro" is seeded for idempotent sharing
    And the owner is signed in as "e2e-test-user@poentaro.test"
    When the app loads the cloud-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a separate browser context loads the same cloud-backed file, signed in as a different account "e2e-test-user-two@poentaro.test"
    And the other account clicks "Start Sync" in that separate browser context
    Then the separate browser context shows the synced share error dialog

  Scenario: Sharing two different cloud-backed files as the same account produces two different links
    Given a cloud file named "idempotent-a.poentaro" is seeded for idempotent sharing
    And a cloud file named "idempotent-b.poentaro" is seeded for idempotent sharing
    And the owner is signed in as "e2e-test-user@poentaro.test"
    When the app loads the cloud-backed file list for idempotent sharing
    And I select the "idempotent-a" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When I select the "idempotent-b" tab to test idempotent sharing
    And the owner clicks "Sync" again
    Then the synced link is copied
    And the copied sync link is different from the original link

  Scenario: A file not stored in the cloud offers no live link
    Given local storage is cleared
    And the owner is signed in as "e2e-test-user@poentaro.test"
    When the owner loads the app and clicks "Sync"
    Then the share modal shows the cloud-only message

  Scenario: Renaming a cloud-backed file after sharing keeps the same link, including from a fresh session elsewhere
    Given a cloud file named "idempotent.poentaro" is seeded for idempotent sharing
    And the owner is signed in as "e2e-test-user@poentaro.test"
    When the app loads the cloud-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When the owner renames the active file to "renamed"
    Then the share modal still shows the stop-sync button
    And the synced link is unchanged after the rename
    When a separate browser context loads the cloud-backed file at its renamed path, signed in as the same account "e2e-test-user@poentaro.test"
    And the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context has the same share id as the original link
