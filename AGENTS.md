# AGENTS.md

Notes for whoever extends this next.

`twitter` is a native Rust CLI for searching and scraping Twitter/X via Apify (actor `apidojo~tweet-scraper`),
built to be driven by an LLM agent. It replaced a .NET global tool (`Twitter.Console`) of the same lineage
and retains its interface: the same commands and flags, YAML-first output, error envelope and exit codes,
while introducing deterministic multi-account safety and hardware-backed OS keystores.

For the manual the *agent* reads, run `twitter agent-readme` — that text lives in `src/readme.rs`
and is the tool's actual interface for its main audience. This file is for the human editing the source.

## Commands

```bash
cargo build --release              # target/release/twitter
cargo test                         # unit tests + tests/cli.rs against a mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # put it on PATH
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export TWITTER_CONFIG_DIR=$(mktemp -d) TWITTER_SECRET_STORE=plaintext TWITTER_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `TWITTER_CONFIG_DIR` | Overrides the config/secrets location |
| `TWITTER_SECRET_STORE` | Forces a backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `TWITTER_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext fallback where no keystore exists |
| `APIFY_API_URL` | Overrides the API base URL — how `tests/cli.rs` points at its mock |
| `APIFY_TOKEN` / `TWITTER_API_TOKEN` | Ephemeral API token for CI/CD or one-off runs |

## Layout

```
src/
  main.rs          arg parsing, the --json pre-scan, clap errors -> invalid_input envelopes
  cli.rs           the whole command tree (clap derive); help text lives here
  commands/
    mod.rs         dispatch, print helper, and me command
    accounts.rs    accounts add|list|test|remove
    login.rs       interactive/headless login
    search.rs      search tweets with filters
    scrape.rs      scrape profiles, tweets, and threads
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode mapping
  error.rs         ErrorCode (= exit code) and Error {code, message, detail, remediation}
  output.rs        YAML by default, JSON with --json, the error envelope, the obj! macro
  account.rs       --account -> Resolved (name, config, key); identity probing of users/me
  config.rs        config.yaml, paths, atomic writes, 0600, cross-process lock
  secrets.rs       Keychain (security), libsecret (secret-tool), DPAPI, plaintext
  readme.rs        agent-readme text, and the API spec/actor version
tests/cli.rs       drives the binary against an in-process mock of the API
```

## Why it is built this way

**Blocking HTTP, no async runtime.** A CLI makes one to a few requests. Tokio would cost more in
startup than it saves; multi-account checks (`accounts list --check`) fan out with scoped threads.

**The Keychain goes through `/usr/bin/security`, not the Security framework.**
Reading items through `/usr/bin/security` never prompts or causes code-signing verification loops
after unsigned binary rebuilds.

**Responses stay `serde_json::Value`.** The Apify actor and Twitter API schemas evolve frequently.
Untyped JSON ensures the CLI passes new fields transparently to agents and users rather than failing.

## How a command is wired

1. Add the variant (with doc comments as help text) to the right enum in `src/cli.rs`.
2. Handle it in `src/commands/` — `client(&a)?` resolves the account and returns a `Client`.
   Print with `print(v)`.
3. Fail by returning `Err(Error::invalid(..))` (or another code) with `.fix(..)` when a specific
   command fixes it. Never `process::exit`, never print an error yourself.
4. Add the row to `README.md` **and** to `src/readme.rs`. A command an agent cannot discover in
   `agent-readme` effectively does not exist.
5. Add a case to `tests/cli.rs` that asserts on the request body the mock received.

## Invariants — do not casually revert these

**Deterministic Multi-Account Safety.** `--account <name>` (`-a <name>`) allows safe multi-account
workspaces without accidental token pollution. Ad-hoc tokens via `--api-key` or `APIFY_TOKEN` env
vars are supported for scripts.

**Secrets never touch `config.yaml`.** Config holds the account name, identity, plan, and date.
The API token goes through `secrets::Store` into the OS keystore. When no keystore is available
the tool refuses to start rather than silently writing a file — `TWITTER_ALLOW_PLAINTEXT_STORE=1`
is the explicit opt-out.
