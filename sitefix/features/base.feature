Feature: Base Tests
    Background:
        Given I have the environment variables:
            | SITEFIX_SOURCE | public |

    Scenario: Checks run
        Given I have a "public/index.html" file with the body:
            """
            <p>Hello!</p>
            """
        When I run my program
        Then I should see "Checked 1 file" in stdout

