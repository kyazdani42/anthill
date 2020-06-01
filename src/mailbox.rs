use imap::types::Flag;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::Deserialize;

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

use super::HOSTNAME;

const EPOCH: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH;

#[derive(Clone)]
struct Message<'a> {
    uid: u32,
    msg_id: String,
    flags: Vec<&'a str>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MailBox {
    pub local: String,
    remote: String,
}

impl MailBox {
    pub fn new(local: &str, remote: &str) -> Self {
        Self {
            local: local.to_string(),
            remote: remote.to_string(),
        }
    }

    pub fn sync(&self, session: &mut imap::Session<TcpStream>, store: &str) {
        if let Err(e) = session.select(&self.remote) {
            return eprintln!(
                "Error when accessing data for mailbox {}: {}",
                self.remote, e
            );
        }

        let mut messages = vec![];
        if let Err(e) = self.get_remote_messages(session, &mut messages) {
            return eprintln!(
                "Error when fetching message info for mailbox {}: {}",
                self.remote, e
            );
        }

        let mail_folder = format!("{}/{}", store, self.local);
        let local_uids = self.get_local_uids(&mail_folder);

        for data in &messages {
            self.sync_msg(session, data, &mail_folder, &local_uids);
        }
    }

    fn sync_msg(
        &self,
        session: &mut imap::Session<TcpStream>,
        data: &Message,
        mail_folder: &str,
        local_uids: &HashSet<u32>,
    ) {
        if let Some(_) = local_uids.get(&data.uid) {
            return;
        }
        println!("Fetching body for message {}", data.uid);
        let body = if let Ok(v) = self.fetch_body(session, data.uid) {
            if let Some(v) = v {
                v
            } else {
                return;
            }
        } else {
            return;
        };
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

    fn fetch_body(
        &self,
        session: &mut imap::Session<TcpStream>,
        uid: u32,
    ) -> imap::error::Result<Option<Vec<u8>>> {
        let message = session.uid_fetch(uid.to_string(), "(BODY[])")?;
        let body = message
            .iter()
            .next()
            .expect(&format!("message with uid {} should have a body", uid))
            .body();
        let body = if let Some(v) = body {
            v
        } else {
            eprintln!("message {} did not have a body", uid);
            return Ok(None);
        };

        Ok(Some(body.to_owned()))
    }

    fn get_local_uids(&self, mail_folder: &str) -> HashSet<u32> {
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

    fn get_remote_messages(
        &self,
        session: &mut imap::Session<TcpStream>,
        all_messages: &mut Vec<Message>,
    ) -> imap::error::Result<()> {
        let messages = session.fetch("1:*", "(UID ENVELOPE FLAGS)")?;

        *all_messages = Vec::with_capacity(messages.len());
        for (i, msg) in messages.iter().enumerate() {
            let uid = if let Some(uid) = msg.uid {
                uid
            } else {
                eprintln!("message {} does not have an uid", i + 1);
                continue;
            };

            let envelope = msg.envelope();
            if let None = envelope {
                eprintln!("message {} does not have an envelope", i + 1);
                continue;
            };

            let envelope = envelope.unwrap();
            let msg_id = if let Some(msg_id) = envelope.message_id {
                msg_id
            } else {
                eprintln!("message {} does not have an associated message id", uid);
                continue;
            };

            if let Ok(msg_id) = std::str::from_utf8(msg_id) {
                all_messages.push(Message {
                    msg_id: msg_id.to_string(),
                    flags: msg.flags().iter().map(format_flag).collect(),
                    uid,
                });
            } else {
                eprintln!("message id for message {} is not valid utf-8", i + 1);
            }
        }

        Ok(())
    }
}

fn transform_flags(flags: &Vec<&str>) -> String {
    flags
        .iter()
        .fold("".to_string(), |a, f| format!("{}{}", a, f))
}

fn format_flag<'a>(flag: &Flag) -> &'a str {
    match flag {
        Flag::Seen => "S",
        Flag::Flagged => "F",
        Flag::Answered => "R",
        Flag::Deleted => "D",
        // TODO
        Flag::Recent | Flag::MayCreate | Flag::Draft | Flag::Custom(_) => "",
    }
}
