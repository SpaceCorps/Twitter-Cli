//! `twitter search <query>`.

use std::io::IsTerminal;

use serde_json::json;

use crate::cli::Search;
use crate::commands::{client, print};
use crate::error::Result;

pub fn run(args: Search) -> Result<()> {
    let client = client(&args.account)?;

    if std::io::stderr().is_terminal() {
        eprintln!("Searching Twitter/X (this may take 30–60s)...");
    }

    let mut payload = json!({
        "searchTerms": [args.query],
        "maxItems": args.max,
        "sort": args.sort,
        "onlyVerifiedUsers": args.verified,
        "onlyImage": args.images,
        "onlyVideo": args.videos,
        "onlyQuote": false,
    });

    if let Some(lang) = args.lang.filter(|s| !s.trim().is_empty()) {
        payload["tweetLanguage"] = json!(lang);
    }
    if let Some(since) = args.since.filter(|s| !s.trim().is_empty()) {
        payload["start"] = json!(since);
    }
    if let Some(until) = args.until.filter(|s| !s.trim().is_empty()) {
        payload["end"] = json!(until);
    }

    let doc = client.search(&payload)?;
    print(doc)
}
