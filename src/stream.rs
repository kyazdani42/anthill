use imap::types::Flag;
use imap::{Client, Session};
use std::net;

pub type AnthillResult<T> = Result<T, String>;
pub type AnthillSession = Session<net::TcpStream>;

pub fn create_session(
    url: &str,
    login: &str,
    pass: &str,
    mailbox: &str,
) -> AnthillResult<AnthillSession> {
    let stream = std::net::TcpStream::connect(url)
        .map_err(|e| format!("could not connect to Tcp Stream: {}", e))?;

    let mut session = Client::new(stream)
        .login(login, pass)
        .map_err(|e| format!("could not login to imap session: {}", e.0))?;

    session
        .select(mailbox)
        .map_err(|e| format!("could not select mailbox {}: {}", mailbox, e))?;

    Ok(session)
}

pub fn logout(mut session: AnthillSession) -> AnthillResult<()> {
    session
        .logout()
        .map_err(|e| format!("could not logout from session: {}", e))?;
    Ok(())
}

// TODO: errors
pub fn fetch_body(session: &mut AnthillSession, uid: u32) -> imap::error::Result<Option<Vec<u8>>> {
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

#[derive(Clone)]
pub struct Message<'a> {
    pub uid: u32,
    pub msg_id: String,
    pub flags: Vec<&'a str>,
}

// TODO: Errors
pub fn get_remote_messages(
    session: &mut AnthillSession,
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
