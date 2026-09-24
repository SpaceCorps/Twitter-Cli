# Twitter CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Twitter-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Twitter-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Twitter-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Twitter-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Twitter-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for searching and scraping Twitter/X via Apify (actor `apidojo~tweet-scraper`). Built in Rust 2024 for developers and autonomous AI workflows.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Executes in ~1–3 ms with no interpreter startup delays.
- 🔐 **OS Keystore Integration**: `twitter login` securely saves your Apify token to native OS vaults (macOS Keychain, Linux Secret Service / Keyutils, Windows DPAPI).
- 🔍 **Full Twitter Search & Scrape**: Search tweets by keyword, hashtag, or mention with filters (language, verified, media, date ranges). Scrape profiles, user timelines, and full tweet threads.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, human-friendly YAML default, standardized error envelopes with stable exit codes, and embedded `agent-readme` documentation.
- 🛡️ **Multi-Account Safety**: Strict account scoping protects multi-tenant setups and prevents cross-account token pollution.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Twitter-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Twitter-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`twitter-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Twitter-Cli/releases/download/v1.0.0/twitter-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`twitter-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Twitter-Cli/releases/download/v1.0.0/twitter-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`twitter-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Twitter-Cli/releases/download/v1.0.0/twitter-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`twitter-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Twitter-Cli/releases/download/v1.0.0/twitter-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Authenticate

Run `twitter login` to launch your browser, copy your Apify token from `https://console.apify.com/account/integrations`, verify it against `GET /v2/users/me`, and store it in your OS keystore:

```bash
# Interactive browser login (saved under account 'default')
twitter login

# Log in with a specific account name
twitter login work

# Headless / CI pipeline login (reads token from stdin with no shell history trace)
echo "$APIFY_TOKEN" | twitter login ci --api-key-stdin

# Or pass token directly via environment variable
export APIFY_TOKEN=your-apify-token
```

### 2. Search Tweets

```bash
# Basic query search (YAML output)
twitter search "agentic coding"

# Search with filters: top tweets, max results, language, date window
twitter search "Claude Code" --sort Top --max 10
twitter search "AI coding" --lang en --since 2025-01-01 --until 2025-12-31
twitter search "developer tools" --verified --images

# Pipe structured JSON into jq or agent tools
twitter search "rust lang" --json | jq '.[0].text'
```

### 3. Scrape Profiles & Threads

```bash
# Scrape a user's recent tweets
twitter scrape "https://x.com/username" --max 50

# Scrape using a Twitter handle directly
twitter scrape "@username" --max 20

# Scrape an entire conversation thread
twitter scrape "https://x.com/username/status/123456789"
```

---

## Command Reference

### Commands

| Command | Description | Primary Options |
|:---|:---|:---|
| `twitter search <query>` | Search Twitter/X for matching tweets | `--max`, `--sort`, `--lang`, `--verified`, `--images`, `--videos`, `--since`, `--until` |
| `twitter scrape <url>` | Scrape a user profile, tweet, or thread | `--max`, `--followers`, `--following`, `--retweeters` |
| `twitter me` | Query token identity, username, and plan | `-a, --account`, `--api-key` |
| `twitter login [name]` | Authenticate interactively and store token in keystore | `--api-key`, `--api-key-stdin`, `--no-browser`, `--force` |
| `twitter accounts list` | List configured accounts (pass `--check` to verify) | `--check` |
| `twitter accounts test <name>` | Verify account credentials and test connectivity | none |
| `twitter accounts add <name>` | Manually add an account and API token | `--api-key`, `--api-key-stdin`, `--force` |
| `twitter accounts remove <name>` | Remove an account and delete token from keystore | `--yes` |
| `twitter agent-readme` | Print embedded LLM agent operating manual | `--json` |

---

## AI Agent Integration

`twitter` is designed from the ground up for LLM agent integration:

- **Agent Manual**: Run `twitter agent-readme` (or `twitter agent-readme --json`) for the complete operating manual.
- **Machine-Readable Outputs**: Pass `--json` anywhere on the command line for raw JSON arrays and objects.
- **Structured Error Envelopes**: Errors return JSON envelopes on `stderr` with stable numeric exit codes:
  - `0`: Success (`ok`)
  - `1`: Unclassified error (`error`)
  - `2`: Network / transport error (`network`)
  - `3`: Authentication failure / expired token (`auth_required`)
  - `4`: Not found (`not_found`)
  - `5`: Rate limit reached (`rate_limited`)
  - `6`: Invalid arguments / parameters (`invalid_input`)
  - `7`: No account or token provided (`no_account`)

---

## Documentation & Trust Links

- **Documentation Site**: [https://spacecorps.github.io/Twitter-Cli/](https://spacecorps.github.io/Twitter-Cli/)
- **LLM Manifest (`llms.txt`)**: [https://spacecorps.github.io/Twitter-Cli/llms.txt](https://spacecorps.github.io/Twitter-Cli/llms.txt)
- **Full Agent Manual**: [https://spacecorps.github.io/Twitter-Cli/llms-full.txt](https://spacecorps.github.io/Twitter-Cli/llms-full.txt)
- **Authentication Guide**: [auth.md](docs/auth.md)
- **Pricing & Licensing**: [pricing.md](docs/pricing.md)
- **About Project**: [about.html](docs/about.html)
- **Contact & Support**: [contact.html](docs/contact.html)
- **Privacy Policy**: [privacy.html](docs/privacy.html)

---

## License

MIT License. Copyright (c) 2026 SpaceCorps. See [LICENSE](LICENSE) for details.
