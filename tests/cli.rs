//! Drives the built binary against an in-process mock of the Apify API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-prefix, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v2/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("");
                    let path = raw_path.trim_start_matches("/v2/").to_string();
                    let path_without_query = path.split('?').next().unwrap_or(&path).to_string();

                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap_or(Value::Null));
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| {
                            *m == method && (*p == path || *p == path_without_query || path.starts_with(p))
                        })
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"error": {"type": "not_found", "message": "no route"}})));

                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "twitter-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_twitter"))
            .args(args)
            .env("TWITTER_CONFIG_DIR", &self.dir)
            .env("TWITTER_SECRET_STORE", "plaintext")
            .env("TWITTER_ALLOW_PLAINTEXT_STORE", "1")
            .env("APIFY_API_URL", &self.api)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let filtered = s
                .lines()
                .filter(|l| {
                    !l.starts_with("warning:")
                        && !l.starts_with("Searching")
                        && !l.starts_with("Scraping")
                        && !l.starts_with("Successfully")
                })
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::from_str(&filtered).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap_or(-1), parse(&out.stdout), parse(&out.stderr))
    }

    /// Adds account `work` with token `apify_test_token`, verified against mock `users/me`.
    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "apify_test_token"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn me() -> Route {
    (
        "GET",
        "users/me",
        200,
        json!({
            "data": {
                "id": "user_123",
                "username": "devuser",
                "email": "dev@example.com",
                "plan": "developer"
            }
        }),
    )
}

fn actor_search_route() -> Route {
    (
        "POST",
        "acts/apidojo~tweet-scraper/run-sync-get-dataset-items",
        200,
        json!([
            {
                "id": "tweet_1",
                "text": "Agentic coding in Rust is blazing fast!",
                "userName": "rustacean",
                "likes": 42
            },
            {
                "id": "tweet_2",
                "text": "SpaceCorps delivers production-grade CLI tools.",
                "userName": "spacecorps",
                "likes": 128
            }
        ]),
    )
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![me()]);
    let env = Env::new(&mock).with_account();

    assert_eq!(
        mock.last("GET").headers.iter().find(|(k, _)| k == "authorization").unwrap().1,
        "Bearer apify_test_token"
    );

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "work");
    assert_eq!(out["accounts"][0]["identity"], "devuser");
    assert_eq!(out["accounts"][0]["plan"], "developer");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (code, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(code, 0);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    let (code, out, _) = env.json(&["accounts", "test", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["identity"], "devuser");
    assert_eq!(out["keyStatus"], "valid");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 0);
}

#[test]
fn login_command() {
    let mock = Mock::start(vec![me()]);
    let env = Env::new(&mock);

    let (code, out, err) = env.json(&["login", "primary", "--api-key", "apify_primary_token", "--no-browser"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "logged_in");
    assert_eq!(out["name"], "primary");
    assert_eq!(out["identity"], "devuser");

    let (code, out, _) = env.json(&["me", "-a", "primary"]);
    assert_eq!(code, 0);
    assert_eq!(out["data"]["username"], "devuser");
}

#[test]
fn search_tweets() {
    let mock = Mock::start(vec![me(), actor_search_route()]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "search",
        "agentic coding",
        "-a",
        "work",
        "--max",
        "10",
        "--sort",
        "Top",
        "--lang",
        "en",
        "--verified",
        "--images",
        "--videos",
        "--since",
        "2025-01-01",
        "--until",
        "2025-12-31",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());
    assert_eq!(out.as_array().unwrap().len(), 2);
    assert_eq!(out[0]["id"], "tweet_1");
    assert_eq!(out[1]["userName"], "spacecorps");

    let last_post = mock.last("POST");
    let payload = last_post.body.expect("payload should exist");
    assert_eq!(payload["searchTerms"][0], "agentic coding");
    assert_eq!(payload["maxItems"], 10);
    assert_eq!(payload["sort"], "Top");
    assert_eq!(payload["tweetLanguage"], "en");
    assert_eq!(payload["onlyVerifiedUsers"], true);
    assert_eq!(payload["onlyImage"], true);
    assert_eq!(payload["onlyVideo"], true);
    assert_eq!(payload["start"], "2025-01-01");
    assert_eq!(payload["end"], "2025-12-31");
}

#[test]
fn scrape_profile_and_thread() {
    let mock = Mock::start(vec![me(), actor_search_route()]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["scrape", "https://x.com/rustlang", "-a", "work", "--max", "50", "--followers"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());

    let last_post = mock.last("POST");
    let payload = last_post.body.expect("payload should exist");
    assert_eq!(payload["startUrls"][0], "https://x.com/rustlang");
    assert_eq!(payload["maxItems"], 50);
    assert_eq!(payload["includeFollowers"], true);

    // Also test scraping handle directly
    let (code, out, _) = env.json(&["scrape", "@spacecorps", "-a", "work"]);
    assert_eq!(code, 0);
    assert!(out.is_array());
    let payload2 = mock.last("POST").body.unwrap();
    assert_eq!(payload2["startUrls"][0], "https://x.com/spacecorps");
    assert_eq!(payload2["twitterHandles"][0], "spacecorps");
}

#[test]
fn ad_hoc_api_key_and_env_token() {
    let mock = Mock::start(vec![actor_search_route()]);
    let env = Env::new(&mock);

    // Direct --api-key
    let (code, out, _) = env.json(&["search", "rust", "--api-key", "token_adhoc"]);
    assert_eq!(code, 0);
    assert!(out.is_array());
    assert_eq!(mock.last("POST").headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer token_adhoc");

    // APIFY_TOKEN environment variable
    let out = Command::new(env!("CARGO_BIN_EXE_twitter"))
        .args(["search", "rust", "--json"])
        .env("TWITTER_CONFIG_DIR", &env.dir)
        .env("TWITTER_SECRET_STORE", "plaintext")
        .env("TWITTER_ALLOW_PLAINTEXT_STORE", "1")
        .env("APIFY_API_URL", &env.api)
        .env("APIFY_TOKEN", "token_from_env")
        .output()
        .unwrap();
    assert_eq!(out.status.code().unwrap(), 0);
    assert_eq!(
        mock.last("POST").headers.iter().find(|(k, _)| k == "authorization").unwrap().1,
        "Bearer token_from_env"
    );
}

#[test]
fn agent_readme_text_and_json() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "twitter");
    assert!(out["rules"].is_array());
    assert_eq!(out["exitCodes"]["0"], "ok");
    assert_eq!(out["exitCodes"]["3"], "auth_required - stop, surface the remediation to a human");

    let out = env.run(&["agent-readme"]);
    assert_eq!(out.status.code().unwrap(), 0);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("# twitter - agent operating manual"));
}

#[test]
fn error_mapping_and_envelopes() {
    let routes = vec![
        me(),
        (
            "POST",
            "acts/apidojo~tweet-scraper/run-sync-get-dataset-items",
            401,
            json!({
                "error": {
                    "type": "user-or-token-not-found",
                    "message": "User was not found or authentication token is not valid"
                }
            }),
        ),
    ];
    let mock = Mock::start(routes);
    let env = Env::new(&mock).with_account();

    // 401 Unauthorized -> code 3 auth_required
    let (code, _, err) = env.json(&["search", "fail", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");
    assert!(err["remediation"].as_str().unwrap().contains("twitter login"));

    // Missing account and no env var -> code 7 no_account
    let (code, _, err) = env.json(&["search", "no_account"]);
    assert_eq!(code, 7);
    assert_eq!(err["code"], "no_account");

    // Invalid CLI arguments -> code 6 invalid_input
    let (code, _, err) = env.json(&["search"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
}
