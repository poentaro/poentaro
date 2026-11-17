Feature: Storage settings error banner for cloud autosave failures

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: A network-failed autosave shows the offline banner, which clears once a save succeeds
    Given a cloud file named "banner.poentaro" is seeded for the signed-in account
    And the first content-save request will be aborted as a network failure
    And I open and edit "banner" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the autosave debounce interval for the error banner
    And I open the storage settings modal to check the error banner
    Then the status banner is visible and mentions "offline"
    When I close the storage settings modal
    And I append " 6" to the editor to trigger a recovering autosave
    Then the editor contains "1 2 3 4 5 6"
    When I fast-forward the clock past the autosave debounce interval for the error banner
    Then the content save lands for "banner.poentaro" containing "1 2 3 4 5 6"
    When I open the storage settings modal to check the error banner
    Then the status banner is gone
