Feature: An AI agent edits the owner's scores with an API key

  Background:
    Given an account is signed in as "e2e-agent@poentaro.test"

  Scenario: Creating a key shows the prompt to paste into the agent
    When the app loads for the account chip flow
    And I click the account chip
    And I click "AI agents" in the profile popover
    And I click "Create key" in the AI agents dialog
    Then the AI agents dialog shows a prompt that saves the new key and installs the Poentaro skill
    And the AI agents dialog lists the new key

  Scenario: The agent learns the protocol and the score syntax from its requests
    Given the signed-in account has an API key
    When the agent sends "SCHEMA"
    Then the agent's last result contains "PROTOCOL poentaro VERSION 1"
    And the agent's last result contains "METHOD syntax RETURNS"
    When the agent sends "score/syntax"
    Then the agent's last result is not an error
    And the agent's last result contains "Syntax Reference"

  Scenario: An agent's save shows up in the open score
    Given a cloud file named "agent-live.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    And I open "agent-live"
    When the agent writes "agent-live.poentaro" at the revision it first saw with the last line "5 6 7 1"
    Then the agent's last result is not an error
    And the editor now shows the remote content "5 6 7 1"

  Scenario: A save comes back with the score's warnings
    Given a cloud file named "agent-warned.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent writes "agent-warned.poentaro" at the revision it first saw with the last line "1 2 (3) 4"
    Then the agent's last result is not an error
    And the agent's last result contains "at least 2 notes"
    And the cloud file "agent-warned.poentaro" contains "1 2 (3) 4"

  Scenario: The agent exports a score as SVG and MIDI to inspect it
    Given a cloud file named "agent-export.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    And the agent writes "agent-export.poentaro" at the revision it first saw with the last line "1 2 3 4"
    When the agent sends "files/export/svg \"agent-export.poentaro\""
    Then the agent's last result is not an error
    And the agent's last result contains "<svg"
    When the agent sends "files/export/midi \"agent-export.poentaro\""
    Then the agent's last result starts with "files/export/midi \"agent-export.poentaro\" RETURNS \"TVRoZA"

  Scenario: A write at an old revision is a conflict and changes nothing
    Given a cloud file named "agent-conflict.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent writes "agent-conflict.poentaro" at the revision it first saw with the last line "5 6 7 1"
    And the agent writes "agent-conflict.poentaro" at the revision it first saw with the last line "2 2 2 2"
    Then the agent's last result starts with "files/write \"agent-conflict.poentaro\" at revision"
    And the agent's last result contains " ERROR conflict at revision"
    And the cloud file "agent-conflict.poentaro" contains "5 6 7 1"

  Scenario: A score with errors is rejected and not saved
    Given a cloud file named "agent-rejected.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent writes "agent-rejected.poentaro" at the revision it first saw with the content "parts [oops"
    Then the agent's last result starts with "files/write \"agent-rejected.poentaro\" at revision"
    And the agent's last result contains " ERROR rejected"
    And the cloud file "agent-rejected.poentaro" contains "1 2 3 4"

  Scenario: The key never reaches another account's files
    Given a cloud file named "agent-private.poentaro" is seeded for "e2e-agent-other@poentaro.test"
    And the signed-in account has an API key
    When the agent sends "files/read \"agent-private.poentaro\""
    Then the agent's last result starts with "files/read \"agent-private.poentaro\" ERROR missing"

  Scenario: A revoked key is refused
    Given the signed-in account has an API key
    When the key is revoked
    And the agent sends "files/list"
    Then the agent is refused for its key
