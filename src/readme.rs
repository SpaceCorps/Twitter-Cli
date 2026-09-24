//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "twitter",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run twitter accounts list or twitter login",
            },
        });
        return;
    }
    println!("{README}");
}

pub const API_VERSION: &str = "Apify v2 (apidojo/tweet-scraper)";

const RULES: &[&str] = &[
    "Pass --account <name> to use stored credentials, or pass --api-key <key> / set APIFY_TOKEN.",
    "Run 'twitter accounts list' first if you do not know which accounts exist.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "Search and scrape commands invoke Apify actors synchronously and may take 30–60s.",
    "Use --json when you are going to parse or pipe the output.",
    "Empty responses return an empty JSON array '[]' rather than blank output.",
];

const README: &str = r#"# twitter - agent operating manual

A native CLI for searching and scraping Twitter/X via Apify (actor `apidojo~tweet-scraper`).
Results are YAML on stdout by default, errors are YAML on stderr, and `--json` switches both to JSON.
Progress notices are sent only to interactive stderr, so stdout is always clean and safe to parse.

## Authentication & Accounts

Twitter CLI supports deterministic multi-account credential management via native OS keystores
(Keychain on macOS, Windows DPAPI, libsecret on Linux) as well as direct API tokens.

    twitter login [<name>]                      # opens Apify dashboard to copy token
    twitter accounts add <name> --api-key <key> # store token in OS keystore
    printf %s "$KEY" | twitter accounts add <name> --api-key-stdin
    twitter accounts list [--check]             # inspect configured accounts
    twitter accounts test <name>                # test account credentials
    twitter accounts remove <name> --yes        # remove stored account

When running commands:
- Pass `--account <name>` (short `-a <name>`) to select a stored account.
- Pass `--api-key <key>` for ad-hoc script runs.
- Set `APIFY_TOKEN` (or `TWITTER_API_TOKEN`) in the environment as a fallback.

## Search Tweets

Search Twitter/X for tweets matching keywords, hashtags, or mentions:

    twitter search "agentic coding"
    twitter search "Claude Code" --sort Top --max 10
    twitter search "AI coding" --lang en --since 2025-01-01
    twitter search "developer tools" --verified
    twitter search "rust lang" --images --max 15
    twitter search "release announcement" --videos

Options:
- `--max <N>`: Maximum tweets to return (default: 20)
- `--sort <SORT>`: Sort order: Latest, Top (default: Latest)
- `--lang <LANG>`: Filter by tweet language (e.g. en, sv, de, ja)
- `--verified`: Only tweets from verified users
- `--images`: Only tweets containing images
- `--videos`: Only tweets containing videos
- `--since <DATE>`: Tweets after this date (YYYY-MM-DD)
- `--until <DATE>`: Tweets before this date (YYYY-MM-DD)

## Scrape Profiles and Threads

Scrape a user profile, tweet, or conversation thread:

    # Scrape a profile
    twitter scrape "https://x.com/username" --max 50
    twitter scrape "@username" --max 50

    # Scrape a specific tweet or thread
    twitter scrape "https://x.com/username/status/123456789"

Options:
- `--max <N>`: Maximum tweets to return (default: 20)
- `--followers`: Include followers list (profile URLs only)
- `--following`: Include following list (profile URLs only)
- `--retweeters`: Include retweeters (tweet URLs only)

## Identity Verification

Check your current token identity and account metadata:

    twitter me
    twitter me -a work

## Agent Automation & JSON Mode

Add `--json` anywhere on the command line for structured, machine-parseable output:

    twitter search "agentic coding" --json | jq '.[0].text'
    twitter agent-readme --json

## Error Codes & Envelopes

Failures print YAML (or JSON with `--json`) on stderr with a stable `code` and exit status:

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  stop; give the human the remediation string verbatim
    4  not_found      the requested actor or resource was not found
    5  rate_limited   back off before trying again
    6  invalid_input  fix the arguments or parameters
    7  no_account     run `twitter login <name>` or provide `--account <name>`
"#;
