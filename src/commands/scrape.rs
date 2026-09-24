//! `twitter scrape <url>`.

use std::io::IsTerminal;

use serde_json::json;

use crate::cli::Scrape;
use crate::commands::{client, print};
use crate::error::Result;

pub fn run(args: Scrape) -> Result<()> {
    let client = client(&args.account)?;

    if std::io::stderr().is_terminal() {
        eprintln!("Scraping Twitter/X (this may take 30–60s)...");
    }

    let trimmed = args.url.trim();
    let is_url = trimmed.starts_with("http://") || trimmed.starts_with("https://");

    let (start_urls, twitter_handles) = if is_url {
        (vec![trimmed.to_string()], Vec::new())
    } else {
        let handle = trimmed.trim_start_matches('@').to_string();
        (vec![format!("https://x.com/{handle}")], vec![handle])
    };

    let mut payload = json!({
        "startUrls": start_urls,
        "maxItems": args.max,
    });

    if !twitter_handles.is_empty() {
        payload["twitterHandles"] = json!(twitter_handles);
    }
    if args.followers {
        payload["includeFollowers"] = json!(true);
    }
    if args.following {
        payload["includeFollowing"] = json!(true);
    }
    if args.retweeters {
        payload["includeRetweeters"] = json!(true);
    }

    let doc = client.scrape(&payload)?;
    print(doc)
}
