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
            <a href="#beets">Beets</a>
            <a href="/beets/">Beets</a>
            <a id="beets" href="https://beets.com">Beets</a>
            """
        When I run my program
        Then I should see "All ok!" in stdout

    Scenario: Sitefix calls out broken internal links
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Dead Link: <a> links to /beets/, but that page does not exist" in stderr

    @skip
    Scenario: Sitefix calls out broken hash links
        Given I have a "public/index.html" file with the body:
            """
            <a href="#beets">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Dead Link: <a> links to #beets, but no such element exists on the page" in stderr

    @skip
    Scenario: Sitefix accepts valid internal + hash links
        Given I have a "public/beets/index.html" file with the body:
            """
            <p id="beets">Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/#beets">Beets</a>
            """
        When I run my program
        Then I should see "All ok!" in stdout

    @skip
    Scenario: Sitefix calls out broken internal + hash links
        Given I have a "public/beets/index.html" file with the body:
            """
            <p id="beets">Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets/#not-beets">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Dead Link: <a> links to /beets/#not-beets, but no such element exists on that page" in stderr

    @skip
    Scenario: Sitefix calls out http links
        Given I have a "public/index.html" file with the body:
            """
            <a href="http://beets.com">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Insecure Link: <a> links to http://beets.com using http instead of https" in stderr

    @skip
    Scenario: Sitefix warns on non-trailing slashes
        Given I have a "public/beets/index.html" file with the body:
            """
            <p>Beets!</p>
            """
        Given I have a "public/index.html" file with the body:
            """
            <a href="/beets">Beets</a>
            """
        When I run my program
        Then I should see "* public/index.html: Non-trailing: <a> links to /beets instead of /beets/" in stdout
        Then I should see "All ok!" in stdout

    @skip
    Scenario: Sitefix can error on non-trailing slashes
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
        Then I should see "* public/index.html: Non-trailing: <a> links to /beets instead of /beets/" in stderr

    @skip
    Scenario: Sitefix can error on trailing slashes
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
        Then I should see "* public/index.html: Trailing: <a> links to /beets/ instead of /beets" in stderr
