#[macro_use]
extern crate lazy_static;

use std::fs::create_dir_all;
use std::process::Command;
use std::thread;

mod config;
mod mailbox;
mod stream;

use config::Config;
use mailbox::MailBox;

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

    let state = State::new(accounts);
    if let Err(e) = state.mkdir_all() {
        return eprintln!("{}", e);
    };

    state.sync();
}

struct State {
    accounts: Vec<Account>,
}

impl State {
    pub fn new(accounts: Vec<Account>) -> Self {
        Self { accounts }
    }

    pub fn sync(self) {
        for account in self.accounts {
            account.sync();
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

#[derive(Clone)]
pub enum Password {
    Static(String),
    GPG(String),
}

pub struct Account {
    pub name: String,
    pub store: String,
    pub mailboxes: Vec<MailBox>,
}

impl Account {
    pub fn new(name: String, data: Config) -> Result<Self, String> {
        let home = std::env::var("HOME").expect("$HOME variable not found");
        let store = data.folder.replace("~", &home).replace("$HOME", &home);
        let password = get_password(&data.pass_cmd)?;
        Ok(Self {
            store,
            name,
            mailboxes: data
                .mailboxes
                .iter()
                .map(|(_, d)| {
                    MailBox::new(
                        d.local.to_string(),
                        d.remote.to_string(),
                        password.clone(),
                        data.user.to_string(),
                        format!("{}:{}", data.url, data.port),
                        data.with_tls,
                    )
                })
                .collect(),
        })
    }

    pub fn sync(self) {
        let mut threads = vec![];
        for mailbox in self.mailboxes {
            let mut mailbox = mailbox;
            let store = self.store.clone();
            let name = self.name.clone();
            threads.push(thread::spawn(move || {
                if let Err(e) = mailbox.sync(&format!("{}/{}", store, name)) {
                    eprintln!("{}", e);
                }
            }));
        }

        for tx in threads {
            tx.join().expect("waiting for thread to finish syncing");
        }
    }
}

fn get_password(cmd: &str) -> Result<Password, String> {
    let cmd = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("could not execute password command: {}", e))?;
    Ok(Password::Static(
        std::str::from_utf8(&cmd.stdout[0..cmd.stdout.len() - 1])
            .map_err(|_| "Could not read password output".to_string())?
            .to_string(),
    ))
}
