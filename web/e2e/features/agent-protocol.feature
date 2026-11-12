Feature: An AI agent edits the owner's scores with an API key

  Background:
    Given an account is signed in as "e2e-agent@poentaro.test"

  Scenario: Creating a key shows the prompt to paste into the agent
    When the app loads for the account chip flow
    And I click the account chip
    And I click "AI agents" in the profile popover
    And I click "Create key" in the AI agents dialog
    Then the AI agents dialog shows a prompt that fetches the agent guide with the new key
    And the AI agents dialog lists the new key

  Scenario: An agent's save shows up in the open score
    Given a cloud file named "agent-live.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    And I open "agent-live"
    When the agent writes "agent-live.poentaro" at the revision it first saw with the last line "5 6 7 1"
    Then the agent's last result is not an error
    And the editor now shows the remote content "5 6 7 1"

  Scenario: A write at an old revision is a conflict and changes nothing
    Given a cloud file named "agent-conflict.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent writes "agent-conflict.poentaro" at the revision it first saw with the last line "5 6 7 1"
    And the agent writes "agent-conflict.poentaro" at the revision it first saw with the last line "2 2 2 2"
    Then the agent's last result starts with "ERROR conflict at revision"
    And the cloud file "agent-conflict.poentaro" contains "5 6 7 1"

  Scenario: A score with errors is rejected and not saved
    Given a cloud file named "agent-rejected.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent writes "agent-rejected.poentaro" at the revision it first saw with the content "parts [oops"
    Then the agent's last result starts with "ERROR rejected"
    And the cloud file "agent-rejected.poentaro" contains "1 2 3 4"

  Scenario: The key never reaches another account's files
    Given a cloud file named "agent-private.poentaro" is seeded for "e2e-agent-other@poentaro.test"
    And the signed-in account has an API key
    When the agent sends "read \"agent-private.poentaro\""
    Then the agent's last result starts with "ERROR missing"

  Scenario: A revoked key is refused
    Given the signed-in account has an API key
    When the key is revoked
    And the agent sends "list files"
    Then the agent is refused for its key
