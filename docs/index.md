---
title: "Twitter CLI"
description: "A blazing fast native command-line tool and agent interface for searching and scraping Twitter/X via Apify. Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Twitter-Cli/index.md"
---

# Twitter CLI

A blazing fast native command-line tool and agent interface for searching and scraping Twitter/X via Apify (`apidojo~tweet-scraper`). Built in Rust 2024 for developers and autonomous AI agents.

## Quickstart

```bash
# Authenticate interactively via browser token flow
twitter login

# Or non-interactively in headless CI/CD environments
echo "$APIFY_TOKEN" | twitter login --api-key-stdin

# Search tweets
twitter search "agentic coding"

# Search with filters
twitter search "Claude Code" --sort Top --max 10
twitter search "AI coding" --lang en --since 2025-01-01
twitter search "developer tools" --verified

# Scrape a profile
twitter scrape "https://x.com/username" --max 50

# Scrape a specific tweet/thread
twitter scrape "https://x.com/username/status/123456789"
```

## Features

- **Blazing Fast Native Rust**: Sub-millisecond startup times with zero runtime dependencies.
- **AI Agent Native**: Structured YAML output by default, JSON with `--json`, and standardized error envelopes.
- **Secure Keystore Integration**: Token storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Multi-Account Workspaces**: Isolate personal, client, and production accounts safely.
- **Apify Actor Powered**: Backed by `apidojo~tweet-scraper` for reliable, high-volume Twitter data extraction.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Twitter-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Twitter-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/Twitter-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Twitter-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Twitter-Cli)
