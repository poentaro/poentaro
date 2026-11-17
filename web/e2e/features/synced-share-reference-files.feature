Feature: Synced shares point at the owner's cloud file

  # A live link is a pointer to one cloud `files` row: viewers read that
  # file's saved content, and the owner's ordinary autosave is the only
  # thing that ever changes it. Regression coverage for a bug where the
  # owner's page pushed its own copy of the content to the share, one render
  # behind the file switch -- so switching files and editing sent the other
  # file's score to the first file's link. (That a second session signed in
  # as the same account sees the same live link is covered in
  # synced-share-idempotent-create.feature.)

  Background:
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score

  Scenario: Switching to another file and editing it never changes what the first file's viewers see
    Given another cloud file titled "Other Score" is seeded
    And the clock is under test control
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page
    Then the viewer's preview contains "Synced Score"
    When the owner switches to the other cloud file
    And the owner edits the other file's title to "Edited Other Score"
    And the owner's autosave debounce interval elapses
    And the viewer reloads the page
    Then the viewer's preview contains "Synced Score"
    And the viewer's preview no longer contains "Other Score"

  Scenario: Trashing the shared file ends the link, and restoring it brings the score back
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page
    Then the viewer's preview contains "Synced Score"
    When the shared cloud file is moved to the bin
    And the viewer reloads the page
    Then the viewer sees "This synced share has ended."
    And the viewer's preview no longer contains "Synced Score"
    When the shared cloud file is restored from the bin
    And the viewer reloads the page
    Then the viewer's preview contains "Synced Score"

  Scenario: A stopped share's document carries no score content at all
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When the owner clicks the stop-sync button
    Then the share modal shows the start-sync button
    And the raw share document for the copied link has an empty filename and content
