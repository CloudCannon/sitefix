Feature: Link Tests
    Background:
        Given I have the environment variables:
            | SITEFIX_SOURCE | public |

    Scenario: Sitefix accepts valid links
        Given I have a "public/beets/index.html" file with the body:
            """
            <p>Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/">Beets</a>
            """
        When I run my program
        Then I should see "All ok!" in stdout

    Scenario: Sitefix calls out broken links
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Dead Link: <a> links to /beets/, but that page does not exist" in stderr
