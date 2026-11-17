Feature: Moving to the new origin keeps saved data and old links

  The app moved from jianpu-generator.pages.dev to poentaro.pages.dev. The
  old origin serves a bridge page that hands the browser's saved data over
  and redirects to the same link on the new origin.

  Scenario: Local files saved on the old origin are there after the redirect
    Given the old origin has a saved local file "moved.jianpu"
    When I open the old origin at "/?ref=old-link#keep"
    Then I land on the new origin at "/?ref=old-link#keep"
    And the new origin has the saved local files "moved.poentaro"
    And the new origin has no data saved under a legacy name

  Scenario: Files already on the new origin are kept and the carried ones go in a folder
    Given the old origin has a saved local file "moved.jianpu"
    And the new origin has a saved local file "mine.poentaro"
    When I open the old origin at "/?ref=old-link"
    Then I land on the new origin at "/?ref=old-link"
    And the new origin has the saved local files "From jianpu-generator.pages.dev/moved.poentaro, mine.poentaro"

  Scenario: An old origin with nothing saved redirects straight away, keeping the link
    When I open the old origin at "/?ref=old-link#fragment"
    Then I land on the new origin at "/?ref=old-link#fragment"

  Scenario: Files saved before the rename to Poentaro are renamed in place
    Given the new origin has a local file "Hymns/old.jianpu" saved before the rename
    When I open the new origin
    Then the new origin has the saved local files "Hymns/old.poentaro"
    And the new origin has no data saved under a legacy name
