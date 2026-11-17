Feature: Synced share viewer response caching

  # Regression coverage for a bug where a viewer had to refresh the share
  # link several times before an owner's latest save appeared -- caused by
  # the browser being free to cache the GET /shares/:share_id response since
  # the worker sent no caching header on it at all. Pins the worker's
  # response to explicitly forbid caching so a single reload is always
  # enough, and so this can't silently regress if caching headers are added
  # back later (e.g. by a CDN rule or a well-meaning performance change).

  Background:
    Given clipboard permissions are granted

  Scenario: The share document response tells the browser never to cache it
    Given the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page, capturing the share response
    Then the captured share response has a "Cache-Control" header of "no-store"
