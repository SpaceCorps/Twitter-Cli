//! Dispatch for all CLI subcommands.

mod accounts;
mod login;
mod scrape;
mod search;

use serde_json::Value;

use crate::account::{self, Resolved};
use crate::cli::*;
use crate::client::Client;
use crate::error::Result;
use crate::output;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Search(args) => search::run(args),
        Command::Scrape(args) => scrape::run(args),
        Command::Me(args) => me(args),
        Command::Login(args) => login::run(args),
        Command::AgentReadme => {
            crate::readme::print();
            Ok(())
        }
        Command::Accounts(args) => accounts::run(args),
    }
}

pub(crate) fn resolve(a: &Account) -> Result<Resolved> {
    account::resolve(a.account.as_deref(), a.api_key.as_deref())
}

pub(crate) fn client(a: &Account) -> Result<Client> {
    Ok(resolve(a)?.client())
}

pub(crate) fn print(v: Value) -> Result<()> {
    output::write(&v);
    Ok(())
}

fn me(args: Me) -> Result<()> {
    let client = client(&args.account)?;
    let user_info = client.me()?;
    print(user_info)
}
