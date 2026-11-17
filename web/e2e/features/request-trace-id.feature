Feature: Trace IDs on failed requests

  # Every failed worker request answers with a trace ID (the `X-Trace-Id`
  # header and the body's `traceId`), the same one its `app_log` row holds,
  # and the app shows it next to the error so it can be looked up.

  Scenario: A refused sign-in shows the request's trace ID
    Given the account "e2e-trace-id@poentaro.test" is reset
    And an account exists for "e2e-trace-id@poentaro.test"
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I enter the email "e2e-trace-id@poentaro.test" in the account dialog
    And I sign in with the password "not-the-right-password" and record the worker's trace ID
    Then the account dialog shows the error "Incorrect email or password."
    And the account dialog shows the recorded trace ID
