Feature: Link Tests
    Background:
        Given I have the environment variables:
            | SITEFIX_SOURCE | public |

    @skip
    Scenario: Sitefix can fix non-trailing slashes
        Given I have a "public/beets/index.html" file with the body:
            """
            <p>Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets">Beets</a>
            """
        When I run my program with the flags:
            | --internal-urls trailing |
            | --autofix                |
            | --verbose                |
        Then I should see "* public/index.html: Non-trailing: Rewrote link from /beets to /beets/" in stdout
        Then I should see a selector 'a' in "public/index.html" with the attributes:
            | href | /beets/ |

    @skip
    Scenario: Sitefix can fix trailing slashes
        Given I have a "public/beets/index.html" file with the body:
            """
            <p>Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/">Beets</a>
            """
        When I run my program with the flags:
            | --internal-urls nontrailing |
            | --autofix                   |
            | --verbose                   |
        Then I should see "* public/index.html: Trailing: Rewrote link from /beets/ to /beets" in stdout
        Then I should see a selector 'a' in "public/index.html" with the attributes:
            | href | /beets |
