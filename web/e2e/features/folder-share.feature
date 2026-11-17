Feature: Sharing a whole folder from its row menu

  # A folder link is a synced link to a path prefix (or a static snapshot of
  # the folder). Every scenario uses its own folder names because the signed-in
  # account is shared by the whole suite run.
  Background:
    Given clipboard permissions are granted
    And an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Sharing a folder shows a link, a marker on the folder and a via tag on what is inside
    Given a cloud file named "ShareOne/hymn.poentaro" is seeded for the signed-in account
    And a cloud file named "ShareOne/inner/carol.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareOne"
    Then the share dialog is titled 'Share folder "ShareOne"'
    When I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When I close the share dialog
    Then the folder "ShareOne" shows the shared marker
    When I open the file tree list
    And I click the file tree folder "ShareOne"
    Then the file "hymn" is tagged "via ShareOne"
    And the folder "inner" is tagged "via ShareOne"

  Scenario: Stopping a folder link asks for confirmation first
    Given a cloud file named "ShareTwo/hymn.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareTwo"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When I click stop sync in the share dialog
    Then the share dialog asks to confirm stopping
    When I cancel stopping sync
    Then the share dialog shows a synced link
    When I click stop sync in the share dialog
    And I confirm stopping sync
    Then the share dialog offers to start sync
    When I close the share dialog
    Then the folder "ShareTwo" shows no shared marker

  Scenario: A folder inside a shared folder says it is shared via the outer one
    Given a cloud file named "ShareThree/inner/carol.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareThree"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When I close the share dialog
    And I open the file tree list
    And I click the file tree folder "ShareThree"
    Then the menu of the folder "inner" offers "Shared via ShareThree"

  Scenario: Renaming a shared folder keeps its link
    Given a cloud file named "ShareFour/hymn.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareFour"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When I remember the synced link in the share dialog
    And I close the share dialog
    And I open the file tree list
    And I choose "Rename" from the actions menu of the folder "ShareFour"
    And I type "ShareFour2" in the rename dialog and confirm
    Then the folder "ShareFour2" shows the shared marker
    When I open the sharing settings of the folder "ShareFour2"
    Then the synced link in the share dialog is the remembered one

  Scenario: Deleting a shared folder warns that viewers will see it empty
    Given a cloud file named "ShareFive/hymn.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareFive"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When I close the share dialog
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "ShareFive"
    Then the delete folder dialog warns "This folder is shared with a link"

  Scenario: Sharing a folder warns about files inside that already have their own link
    Given a cloud file named "ShareSix/hymn.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the file tree list
    And I click the file tree folder "ShareSix"
    And I click the file tree file "hymn"
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When I close the share dialog
    And I open the share dialog of the folder "ShareSix"
    And I open the synced tab of the share dialog
    Then the share dialog shows the file warning "already has its own synced link"

  Scenario: A small folder gets a static link
    Given a cloud file named "ShareSeven/hymn.poentaro" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareSeven"
    Then the share dialog shows a static link
    And the share dialog describes the snapshot as "1 file in 0 folders"

  Scenario: A folder too big for a static link offers a synced link instead
    Given the cloud folder "ShareEight" holds 3 large unrelated files for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ShareEight"
    Then the share dialog warns the folder is too big
    When I choose the synced link from the size warning
    And I start sync in the share dialog
    Then the share dialog shows a synced link

  Scenario: A folder on this device cannot get a live link
    Given the file tree is seeded with files "Local/a.poentaro", "Local/b.poentaro" and "c.poentaro" and an empty folder "Spare" with "c.poentaro" active
    When the file tree app loads
    And I open the share dialog of the folder "Local"
    And I open the synced tab of the share dialog
    Then the share dialog explains links need cloud storage
