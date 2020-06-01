#[macro_use]
extern crate lazy_static;

use std::fs::create_dir_all;

mod account;
use account::Account;

mod config;
mod mailbox;

lazy_static! {
    pub static ref HOSTNAME: String = gethostname::gethostname()
        .into_string()
        .expect("could not get hostname");
}

fn main() {
    let config = match config::get_config() {
        Ok(c) => c,
        Err(e) => return eprint!("{}", e),
    };

    let mut accounts = vec![];
    for (k, v) in config {
        match Account::new(k, v) {
            Ok(v) => accounts.push(v),
            Err(e) => eprintln!("{}", e),
        }
    }

    let mut state = State::new(accounts);
    if let Err(e) = state.mkdir_all() {
        return eprintln!("{}", e);
    };

    state.connect();
    state.sync();
    state.logout();
}

struct State {
    accounts: Vec<Account>,
}

impl State {
    pub fn new(accounts: Vec<Account>) -> Self {
        Self { accounts }
    }

    pub fn connect(&mut self) {
        for account in &mut self.accounts {
            if let Err(e) = account.connect() {
                eprintln!("could not connect to {}: {}", account.name, e);
            }
        }
    }

    pub fn logout(&mut self) {
        for account in &mut self.accounts {
            account.logout();
        }
    }

    pub fn sync(&mut self) {
        for account in &mut self.accounts {
            if account.is_logged_in() {
                account.sync();
            }
        }
    }

    pub fn mkdir_all(&self) -> Result<(), std::io::Error> {
        for account in &self.accounts {
            for mailbox in &account.mailboxes {
                create_dir_all(format!(
                    "{}/{}/{}/cur",
                    account.store, account.name, mailbox.local
                ))?;
                create_dir_all(format!(
                    "{}/{}/{}/new",
                    account.store, account.name, mailbox.local
                ))?;
                create_dir_all(format!(
                    "{}/{}/{}/tmp",
                    account.store, account.name, mailbox.local
                ))?;
            }
        }

        Ok(())
    }
}
