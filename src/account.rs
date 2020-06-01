use std::collections::HashMap;
use std::net::TcpStream;
use std::process::Command;

use super::config::Config;
use super::mailbox::MailBox;

enum Password {
    Static(String),
    GPG(String),
}

pub struct Account {
    pub name: String,
    pub store: String,
    pub mailboxes: Vec<MailBox>,
    with_ssl: bool,
    host: String,
    port: u16,
    login: String,
    password: Password,
    session: Option<imap::Session<TcpStream>>,
}

fn mailboxes_from_hashmap(mb: HashMap<String, MailBox>) -> Vec<MailBox> {
    let mut mbx = Vec::with_capacity(mb.len());
    for (_, v) in mb {
        mbx.push(v);
    }

    mbx
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
            session: None,
            with_ssl: data.ssl_type,
            host: data.url,
            login: data.user,
            password: get_password(&data.pass_cmd)?,
            mailboxes: mailboxes_from_hashmap(data.mailboxes),
        })
    }

    pub fn connect(&mut self) -> imap::error::Result<()> {
        let stream = TcpStream::connect(&format!("{}:{}", self.host, self.port))?;

        let pass = match &self.password {
            Password::Static(p) => p,
            Password::GPG(p) => p,
        };

        let session = Some(
            imap::Client::new(stream)
                .login(&self.login, pass)
                .map_err(|e| e.0)?,
        );

        self.session = session;

        Ok(())
    }

    pub fn logout(&mut self) {
        if let Some(ref mut s) = self.session {
            s.logout().expect("logging out");
            self.session = None;
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.session.is_some()
    }

    pub fn sync(&mut self) {
        for mailbox in &self.mailboxes.clone() {
            if let Some(ref mut s) = self.session {
                mailbox.sync(s, &format!("{}/{}", self.store, self.name));
            }
        }
    }
}
