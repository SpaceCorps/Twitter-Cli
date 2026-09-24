//! The command tree. One variant per command; `commands` does the work.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "twitter",
    version,
    about = "CLI for searching and scraping Twitter/X via Apify — YAML-first output optimized for LLM agent consumption",
    after_help = "An LLM agent should start with: twitter agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Credential options for commands that call the API.
#[derive(Args, Clone, Debug)]
pub struct Account {
    /// Account to run against (see 'twitter accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Direct Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Search Twitter/X for tweets matching a query
    Search(Search),

    /// Scrape a Twitter/X URL (profile, tweet, or thread)
    Scrape(Scrape),

    /// Get current identity and token context from Apify
    Me(Me),

    /// Log in with an Apify API token (opens browser to copy token)
    Login(Login),

    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,

    /// Manage Twitter CLI accounts and their Apify API tokens
    #[command(subcommand)]
    Accounts(Accounts),
}

#[derive(Args, Clone, Debug)]
pub struct Search {
    /// Search query (keywords, hashtags, or @mentions)
    pub query: String,

    /// Maximum tweets to return
    #[arg(long, default_value_t = 20, value_name = "N")]
    pub max: i32,

    /// Sort by: Latest, Top
    #[arg(long, default_value = "Latest", value_name = "SORT")]
    pub sort: String,

    /// Filter by tweet language (e.g. en, sv, de)
    #[arg(long, value_name = "LANG")]
    pub lang: Option<String>,

    /// Only tweets from verified users
    #[arg(long)]
    pub verified: bool,

    /// Only tweets with images
    #[arg(long)]
    pub images: bool,

    /// Only tweets with videos
    #[arg(long)]
    pub videos: bool,

    /// Tweets after this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Tweets before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub until: Option<String>,

    #[command(flatten)]
    pub account: Account,
}

#[derive(Args, Clone, Debug)]
pub struct Scrape {
    /// Twitter/X URL to scrape (profile, tweet, or thread)
    pub url: String,

    /// Maximum tweets to return
    #[arg(long, default_value_t = 20, value_name = "N")]
    pub max: i32,

    /// Include followers list (profile URLs only)
    #[arg(long)]
    pub followers: bool,

    /// Include following list (profile URLs only)
    #[arg(long)]
    pub following: bool,

    /// Include retweeters (tweet URLs only)
    #[arg(long)]
    pub retweeters: bool,

    #[command(flatten)]
    pub account: Account,
}

#[derive(Args, Clone, Debug)]
pub struct Me {
    #[command(flatten)]
    pub account: Account,
}

#[derive(Args, Clone, Debug)]
pub struct Login {
    /// Account name to store under (defaults to 'default')
    #[arg(default_value = "default")]
    pub name: String,

    /// Apify API token (prompts securely if omitted)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,

    /// Read the API token from stdin rather than prompting
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not attempt to open the Apify token dashboard in a browser
    #[arg(long)]
    pub no_browser: bool,

    /// Overwrite an existing account of the same name
    #[arg(long)]
    pub force: bool,

    /// Store the key without checking that it works
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Accounts {
    /// Add an account and store its API token in the OS keystore
    Add {
        /// Account name
        name: String,

        /// Apify API token (prompts securely if omitted)
        #[arg(long, value_name = "KEY")]
        api_key: Option<String>,

        /// Read the API token from stdin rather than prompting
        #[arg(long)]
        api_key_stdin: bool,

        /// Overwrite an existing account of the same name
        #[arg(long)]
        force: bool,

        /// Store the key without checking that it works
        #[arg(long)]
        no_verify: bool,
    },

    /// List configured accounts
    List {
        /// Verify each stored key against the Apify API
        #[arg(long)]
        check: bool,
    },

    /// Test an account's API token against the Apify API
    Test {
        /// Account name
        name: String,
    },

    /// Remove an account and delete its API token from the OS keystore
    Remove {
        /// Account name
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}
