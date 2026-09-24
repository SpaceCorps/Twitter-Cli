//! Multi-account management and token resolution for Twitter CLI.
//!
//! Explicit account flags (`--account <name>` / `-a <name>`) protect against multi-tenant mistakes.
//! For automation and one-off usage, `--api-key <key>` or the `APIFY_TOKEN` environment variable
//! are also supported.

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub config: AccountConfig,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(requested_account: Option<&str>, requested_key: Option<&str>) -> Result<Resolved> {
    if let Some(key) = requested_key.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(Resolved { name: "api-key".into(), config: AccountConfig::default(), api_key: key.to_string() });
    }

    let config = config::load()?;

    if let Some(requested) = requested_account.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("twitter accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;

        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API token."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("twitter accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: account.clone(), api_key });
    }

    if let Some(env_key) = std::env::var("APIFY_TOKEN")
        .or_else(|_| std::env::var("TWITTER_API_TOKEN"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return Ok(Resolved { name: "env".into(), config: AccountConfig::default(), api_key: env_key });
    }

    Err(Error::new(
        ErrorCode::NoAccount,
        "No account or API token specified. Pass --account <name>, --api-key <key>, or set APIFY_TOKEN.",
    )
    .detail(describe(&config))
    .fix("twitter login <name>"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'twitter login <name>' or 'twitter accounts add <name>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}

pub mod identity {
    use super::Value;

    const CANDIDATES: &[&str] =
        &["username", "userName", "user_name", "email", "userEmail", "user_email", "name", "login", "id"];

    /// A short human label - username or email.
    pub fn describe(me: &Value) -> String {
        if !me.is_object() {
            return String::new();
        }
        first_string(me, CANDIDATES)
            .or_else(|| nested(me, "data", CANDIDATES))
            .or_else(|| nested(me, "user", CANDIDATES))
            .unwrap_or_default()
    }

    /// The plan or subscription type observed.
    pub fn plan(me: &Value) -> String {
        if !me.is_object() {
            return String::new();
        }
        const PLAN_CANDIDATES: &[&str] = &["plan", "planType", "pricingTier", "tier", "subscriptionType"];
        first_string(me, PLAN_CANDIDATES)
            .or_else(|| nested(me, "data", PLAN_CANDIDATES))
            .or_else(|| nested(me, "plan", &["id", "name", "type"]))
            .unwrap_or_default()
    }

    fn nested(root: &Value, property: &str, names: &[&str]) -> Option<String> {
        root.get(property).filter(|c| c.is_object()).and_then(|c| first_string(c, names))
    }

    fn first_string(v: &Value, names: &[&str]) -> Option<String> {
        names
            .iter()
            .filter_map(|n| v.get(*n).and_then(Value::as_str))
            .find(|s| !s.trim().is_empty())
            .map(str::to_string)
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        #[test]
        fn reads_apify_users_me_shape() {
            let me = json!({
                "data": {
                    "id": "wRsJZpLoEGmCfqkmm",
                    "username": "janedoe",
                    "email": "jane@example.com",
                    "plan": "personal"
                }
            });
            assert_eq!(super::describe(&me), "janedoe");
            assert_eq!(super::plan(&me), "personal");
        }

        #[test]
        fn degrades_to_empty() {
            assert_eq!(super::describe(&json!([1])), "");
            assert_eq!(super::plan(&json!({"x": 1})), "");
        }
    }
}
