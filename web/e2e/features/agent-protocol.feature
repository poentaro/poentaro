Feature: An AI agent edits the owner's scores with an API key

  Background:
    Given an account is signed in as "e2e-agent@poentaro.test"

  Scenario: Creating a key shows the prompt to paste into the agent
    When the app loads for the account chip flow
    And I click the account chip
    And I click "AI agents" in the profile popover
    And I click "Create key" in the AI agents dialog
    Then the AI agents dialog shows a prompt that saves the new key and installs the Poentaro skill
    And the AI agents dialog lists the new key, expiring in 30 days

  Scenario: The agent learns the protocol and the score syntax from its requests
    Given the signed-in account has an API key
    When the agent sends "SCHEMA"
    Then the agent's last result contains "PROTOCOL poentaro VERSION 1"
    And the agent's last result contains "READ ONLY METHOD score syntax"
    And the agent's last result contains "METHOD move file"
    When the agent sends "score syntax"
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
    When the agent sends "export svg \"agent-export.poentaro\""
    Then the agent's last result is not an error
    And the agent's last result contains "<svg"
    When the agent sends "export midi \"agent-export.poentaro\""
    Then the agent's last result starts with "RETURNS \"TVRoZA"

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

  Scenario: A renamed score keeps its identity and content
    Given a cloud file named "agent-rename.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    And the agent writes "agent-rename.poentaro" at the revision it first saw with the last line "5 6 7 1"
    When the agent sends "move file \"agent-rename.poentaro\" to \"Agent Choir/Renamed.poentaro\""
    Then the agent's last result is not an error
    And the file first seen as "agent-rename.poentaro" is now at "Agent Choir/Renamed.poentaro"
    And the cloud file "Agent Choir/Renamed.poentaro" contains "5 6 7 1"

  Scenario: A rename onto an existing score is refused and overwrites nothing
    Given a cloud file named "agent-rename-onto.poentaro" is seeded for the signed-in account
    And a cloud file named "agent-rename-from.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    And the agent writes "agent-rename-from.poentaro" at the revision it first saw with the last line "5 6 7 1"
    When the agent sends "move file \"agent-rename-from.poentaro\" to \"agent-rename-onto.poentaro\""
    Then the agent's last result starts with "ERROR taken"
    And the cloud file "agent-rename-onto.poentaro" contains "1 2 3 4"
    And the cloud file "agent-rename-from.poentaro" contains "5 6 7 1"

  Scenario: A rename from a missing score or to an invalid path is refused
    Given a cloud file named "agent-rename-invalid.poentaro" is seeded for the signed-in account
    And the signed-in account has an API key
    When the agent sends "move file \"agent-no-such-score.poentaro\" to \"agent-anything.poentaro\""
    Then the agent's last result starts with "ERROR missing"
    When the agent sends "move file \"agent-rename-invalid.poentaro\" to \"../agent-escaped.poentaro\""
    Then the agent's last result starts with "ERROR invalid path"
    When the agent sends "move file \"agent-rename-invalid.poentaro\" to \"agent-renamed.txt\""
    Then the agent's last result starts with "ERROR invalid path"
    And the cloud file "agent-rename-invalid.poentaro" contains "1 2 3 4"

  Scenario: The key never reaches another account's files
    Given a cloud file named "agent-private.poentaro" is seeded for "e2e-agent-other@poentaro.test"
    And the signed-in account has an API key
    When the agent sends "read file \"agent-private.poentaro\""
    Then the agent's last result starts with "ERROR missing"

  Scenario: A key revoked in the dialog leaves the list and is refused
    Given the signed-in account has an API key
    When the app loads for the account chip flow
    And I click the account chip
    And I click "AI agents" in the profile popover
    And I click Revoke on the agent's key in the AI agents dialog
    And I confirm revoking the key
    Then the AI agents dialog no longer lists the agent's key
    When the agent sends "list files"
    Then the agent is refused for its key

  Scenario: Cancelling the revoke keeps the key working
    Given the signed-in account has an API key
    When the app loads for the account chip flow
    And I click the account chip
    And I click "AI agents" in the profile popover
    And I click Revoke on the agent's key in the AI agents dialog
    And I cancel revoking the key
    Then the AI agents dialog lists the agent's key
    When the agent sends "list files"
    Then the agent's last result starts with "RETURNS"
