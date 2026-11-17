Feature: File-op failure handling for the cloud storage backend

  Background:
    # Dedicated login, not the shared default "e2e-test-user@poentaro.test" -- this
    # scenario asserts no file named exactly "untitled" exists after a
    # failed create, but "untitled" is also the literal name
    # `files-cloud-backend.feature`'s "Creating a file persists it to the
    # cloud backend" scenario's own "New" click produces. Sharing an
    # account would make that scenario's genuine "untitled" file, created
    # concurrently in another `fullyParallel` worker, leak into this one's
    # assertion. See `e2eAccounts.ts`'s comment on this email.
    Given an account is signed in as "e2e-test-user-create-error@poentaro.test"

  Scenario: A create that hits a server error still appears at once and raises no error modal
    Given a cloud file named "existing.poentaro" is seeded for the signed-in account
    And the first file-create request will fail with a 500 error
    When the app loads the cloud-backed file list for a failing create
    And I click the "New" button to create a file that will fail
    Then no error modal is shown
    And an "untitled" tab exists
    And the new-file button spinner clears and its label resets to "New"
