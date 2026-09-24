//! HTTP to the Apify API, and the translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    pub api_key: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("twitter-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("APIFY_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, api_key: api_key.trim().to_string() }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    pub fn me(&self) -> Result<Value> {
        self.get("users/me")
    }

    pub fn search(&self, input: &Value) -> Result<Value> {
        let endpoint = format!("acts/apidojo~tweet-scraper/run-sync-get-dataset-items?token={}", seg(&self.api_key));
        self.post(&endpoint, input)
    }

    pub fn scrape(&self, input: &Value) -> Result<Value> {
        let endpoint = format!("acts/apidojo~tweet-scraper/run-sync-get-dataset-items?token={}", seg(&self.api_key));
        self.post(&endpoint, input)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base, path);

        let auth_header = format!("Bearer {}", self.api_key);
        macro_rules! headers {
            ($req:expr) => {{ $req.header("Authorization", &auth_header).header("Accept", "application/json") }};
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => headers!(self.agent.post(&url)).send_empty(),
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Array(Vec::new()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let parsed_err = serde_json::from_str::<Value>(body).ok();
    let api_type = parsed_err
        .as_ref()
        .and_then(|v| v.get("error").and_then(|e| e.get("type")))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let api_msg = parsed_err
        .as_ref()
        .and_then(|v| v.get("error").and_then(|e| e.get("message")))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let e = match status {
        401 => {
            let msg = if !api_msg.is_empty() {
                format!("The Apify API token was rejected: {api_msg}")
            } else {
                "The Apify API token was rejected or expired.".into()
            };
            Error::new(ErrorCode::AuthRequired, msg)
                .fix("Run 'twitter login <name> --force' or verify your APIFY_TOKEN.")
        }
        403 => {
            let msg = if !api_msg.is_empty() {
                format!("Apify permission denied: {api_msg}")
            } else {
                "Permission denied or Apify usage limit reached.".into()
            };
            Error::new(ErrorCode::AuthRequired, msg)
                .fix("Check your Apify account permissions and credits at https://console.apify.com")
        }
        404 => {
            let msg = if api_type == "user-or-token-not-found" {
                "User was not found or authentication token is not valid.".into()
            } else if !api_msg.is_empty() {
                format!("The resource was not found: {api_msg}")
            } else {
                "The requested resource was not found.".into()
            };
            if api_type == "user-or-token-not-found" {
                Error::new(ErrorCode::AuthRequired, msg)
                    .fix("Run 'twitter login <name> --force' or verify your APIFY_TOKEN.")
            } else {
                Error::new(ErrorCode::NotFound, msg)
            }
        }
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the Apify API.").fix("Back off before retrying."),
        400 | 422 => {
            let msg = if !api_msg.is_empty() {
                format!("The Apify API refused the request: {api_msg}")
            } else {
                "The Apify API refused the request.".into()
            };
            Error::new(ErrorCode::InvalidInput, msg)
        }
        s if s >= 500 => Error::new(ErrorCode::Network, "The Apify API returned a server error.")
            .fix("Retry; if it persists Apify is having trouble."),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

/// Percent-encodes one path segment, so an id or token can never escape its place in the URL.
pub fn seg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seg_escapes_reserved() {
        assert_eq!(seg("token_123"), "token_123");
        assert_eq!(seg("A B/C"), "A%20B%2FC");
    }

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(503, "").code, ErrorCode::Network);
    }
}
