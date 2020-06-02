use rand::{thread_rng, Rng};
use regex::Regex;

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;

use super::stream::{
    create_session, fetch_body, get_remote_messages, logout, AnthillResult, Message,
};
use super::{Password, HOSTNAME};

const EPOCH: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct MailBox {
    pub local: String,
    remote: String,
    pass: String,
    login: String,
    url: String,
}

impl MailBox {
    pub fn new(
        local: String,
        remote: String,
        password: Password,
        login: String,
        url: String,
    ) -> Self {
        let pass = match password {
            Password::Static(p) => p.to_string(),
            Password::GPG(p) => p.to_string(),
        };
        Self {
            local,
            remote,
            pass,
            login,
            url,
        }
    }

    fn fetch_messages<'a>(&self) -> AnthillResult<Vec<Message<'a>>> {
        let mut session = create_session(&self.url, &self.login, &self.pass, &self.remote)?;

        let messages = get_remote_messages(&mut session).map_err(|e| {
            format!(
                "Error when fetching message info for mailbox {}: {}",
                self.remote, e
            )
        })?;

        logout(session)?;

        Ok(messages)
    }

    pub fn sync(&mut self, store: &str) -> AnthillResult<()> {
        let messages = self.fetch_messages()?;
        let mail_folder = format!("{}/{}", store, self.local);
        let local_uids = get_local_uids(&mail_folder);

        let mut threads = vec![];
        for data in messages {
            if let Some(_) = local_uids.get(&data.uid) {
                continue;
            }
            println!("Fetching message `{}` in `{}`", data.msg_id, &self.remote);
            let mail_folder = mail_folder.clone();
            let mut session = match create_session(&self.url, &self.login, &self.pass, &self.remote)
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };
            threads.push(thread::spawn(move || {
                let result = fetch_body(&mut session, data.uid);
                if result.is_err() {
                    return;
                }
                let body = if let Some(v) = result.unwrap() {
                    v
                } else {
                    return;
                };
                sync_msg(&body, &data, &mail_folder);
                if let Err(e) = logout(session) {
                    eprintln!("{}", e);
                }
            }));
        }

        for tx in threads {
            if let Err(_) = tx.join() {
                eprintln!("waiting for thread syncing to finish");
            }
        }

        Ok(())
    }
}

fn sync_msg(body: &[u8], data: &Message, mail_folder: &str) {
    let rnd = thread_rng().gen::<u16>();
    let now = std::time::SystemTime::now()
        .duration_since(EPOCH)
        .unwrap()
        .as_secs();
    let flags = transform_flags(&data.flags);
    let subfolder = if flags.contains("S") { "cur" } else { "new" };

    let write_to = format!(
        "{}/{}/{}.{}.{},U={}:2,{}",
        mail_folder,
        subfolder,
        now,
        rnd,
        HOSTNAME.as_str(),
        data.uid,
        flags
    );

    let mut handle = match fs::File::create(&write_to) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Could not create file {}: {}", &write_to, e);
            return;
        }
    };

    if let Err(e) = handle.write_all(&body) {
        eprintln!("could not write to file: {}", e);
        let _ = fs::remove_file(write_to);
    }
}

fn get_local_uids(mail_folder: &str) -> HashSet<u32> {
    let mut uids = HashSet::new();

    let folder = Path::new(mail_folder);
    let cur_folder = folder.join("cur");
    let tmp_folder = folder.join("tmp");
    let new_folder = folder.join("new");
    let re = Regex::new(r"U=[0-9]+:").unwrap();
    for f in &[cur_folder, tmp_folder, new_folder] {
        for entry in fs::read_dir(f).unwrap() {
            let mail = entry
                .expect("reading a dir entry")
                .file_name()
                .into_string()
                .expect("converting filename");
            let matches = if let Some(m) = re.find(&mail) {
                m
            } else {
                continue;
            };
            let (a, b) = (matches.start(), matches.end());
            let st = std::str::from_utf8(&mail.as_bytes()[a + 2..b - 1])
                .expect("convert matching regex to utf8");
            uids.insert(st.parse::<u32>().unwrap());
        }
    }

    uids
}

fn transform_flags(flags: &Vec<&str>) -> String {
    flags
        .iter()
        .fold("".to_string(), |a, f| format!("{}{}", a, f))
}
