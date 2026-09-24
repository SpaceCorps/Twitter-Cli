---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Twitter CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Twitter CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Twitter CLI.

## Overview
The Twitter CLI interfaces with Twitter/X through Apify's REST API (`https://api.apify.com/v2/`). Authentication requires an Apify API token. Tokens can be stored in the host operating system's native keychain, supplied via environment variables (`APIFY_TOKEN`), or passed directly with `--api-key`.

## Prerequisites
- An Apify account ([apify.com](https://apify.com))
- A valid API token from the Apify console integrations page (`https://console.apify.com/account/integrations`)
- Twitter CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Twitter-Cli --locked`)

## Authentication Flow

### Interactive Browser Login (`twitter login`)
The recommended flow for local developer machines:
```bash
twitter login [account_name]
```
1. The CLI launches your system browser to `https://console.apify.com/account/integrations`.
2. You copy your personal Apify API token.
3. Paste the token into the CLI prompt (input characters are masked).
4. The CLI validates the key with a live request to `GET /v2/users/me`.
5. Upon confirmation, the key is securely saved to the native OS keyring under the account name (defaults to `default`).

### Non-Interactive / Headless Login
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
echo "$APIFY_TOKEN" | twitter login [account_name] --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
twitter login [account_name] --api-key "$APIFY_TOKEN"
```

## Environment Variables
The CLI checks the environment for credentials when no keychain account is specified:
- `APIFY_TOKEN`: Fallback API token used if no keystore account is explicitly selected.
- `TWITTER_API_TOKEN`: Alternative alias for `APIFY_TOKEN`.
- `TWITTER_CONFIG_DIR`: Custom directory path for configuration metadata.
- `TWITTER_ALLOW_PLAINTEXT_STORE=1`: Explicit opt-in for plaintext token fallback when no OS keystore is present.

## Multi-Account Management
Switch or verify accounts using:
```bash
twitter accounts list --check
twitter accounts test [account_name]
twitter accounts remove [account_name] --yes
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error payloads:
- `auth_required` (code 3): Token missing, invalid, or expired.
- `no_account` (code 7): Specified account does not exist in keystore.
- `invalid_input` (code 6): Token or argument format rejected.
- `rate_limited` (code 5): Apify API rate limits reached.
