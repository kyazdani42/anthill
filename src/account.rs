use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use super::config::Config;
use super::mailbox::MailBox;

enum Password {
    Static(String),
    GPG(String),
}

pub struct Account {
    pub name: String,
    pub store: String,
    pub mailboxes: Vec<Arc<Mutex<MailBox>>>,
    with_ssl: bool,
    host: String,
    port: u16,
    login: String,
    password: Password,
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

impl Account {
    pub fn new(name: String, data: Config) -> Result<Self, String> {
        let home = std::env::var("HOME").expect("$HOME variable not found");
        let store = data.folder.replace("~", &home).replace("$HOME", &home);
        Ok(Self {
            store,
            name,
            port: data.port,
            with_ssl: data.ssl_type,
            host: data.url,
            login: data.user,
            password: get_password(&data.pass_cmd)?,
            mailboxes: data
                .mailboxes
                .iter()
                .map(|(_, d)| Arc::new(Mutex::new(MailBox::new(&d.local, &d.remote))))
                .collect(),
        })
    }

    pub fn connect(&mut self) {
        let pass = match &self.password {
            Password::Static(p) => p.to_string(),
            Password::GPG(p) => p.to_string(),
        };

        let mut threads = vec![];
        for mb in &self.mailboxes {
            let mailbox = mb.clone();
            let login = self.login.to_string();
            let v = format!("{}:{}", self.host.clone(), self.port);
            let pass = pass.to_string();
            threads.push(thread::spawn(move || {
                let stream = match TcpStream::connect(v) {
                    Ok(s) => s,
                    Err(e) => return eprintln!("could not connect to Tcp Stream: {}", e),
                };
                let session = match imap::Client::new(stream).login(login, pass.to_string()) {
                    Ok(s) => s,
                    Err(e) => return eprintln!("could not login to imap session: {}", e.0),
                };

                mailbox.lock().expect("acquiring lock").set_session(session);
            }));
        }

        for tx in threads {
            tx.join().expect("waiting for thread to finish connecting");
        }
    }

    pub fn logout(&mut self) {
        for mb in &mut self.mailboxes {
            mb.lock().expect("acquiring lock").logout();
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.mailboxes
            .iter()
            .any(|v| v.lock().expect("acquiring lock").session.is_some())
    }

    pub fn sync(&mut self) {
        let mut threads = vec![];
        for mailbox in &self.mailboxes {
            let mailbox = mailbox.clone();
            let store = self.store.clone();
            let name = self.name.clone();
            threads.push(thread::spawn(move || {
                mailbox
                    .lock()
                    .expect("acquiring lock")
                    .sync(&format!("{}/{}", store, name));
            }));
        }

        for tx in threads {
            tx.join().expect("waiting for thread to finish syncing");
        }
    }
}
