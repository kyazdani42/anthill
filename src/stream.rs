use imap::types::Flag;
use imap::{Client, Session};
use std::io::{Read, Write};
use std::net;

pub enum Stream {
    TLS(native_tls::TlsStream<net::TcpStream>),
    NOTLS(net::TcpStream),
}

impl Stream {
    pub fn new(with_tls: bool, url: &str) -> AnthillResult<Self> {
        let tcp_stream = net::TcpStream::connect(url)
            .map_err(|e| format!("could not connect to Tcp Stream: {}", e))?;
        let s = if with_tls {
            let tls = native_tls::TlsConnector::builder()
                .build()
                .map_err(|e| format!("could not initialize tls connexion: {}", e))?;
            let tls = tls
                .connect(url, tcp_stream)
                .map_err(|e| format!("failed to connect with tls: {}", e))?;
            Stream::TLS(tls)
        } else {
            Stream::NOTLS(tcp_stream)
        };

        Ok(s)
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            Stream::TLS(s) => s.read(buf),
            Stream::NOTLS(s) => s.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Stream::TLS(s) => s.write(buf),
            Stream::NOTLS(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self {
            Stream::TLS(s) => s.flush(),
            Stream::NOTLS(s) => s.flush(),
        }
    }
}

pub type AnthillResult<T> = Result<T, String>;
pub type AnthillSession = Session<Stream>;

pub fn create_session(
    url: &str,
    login: &str,
    pass: &str,
    mailbox: &str,
    with_tls: bool,
) -> AnthillResult<AnthillSession> {
    let stream = Stream::new(with_tls, url)?;

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
pub fn get_remote_messages<'a>(
    session: &mut AnthillSession,
) -> imap::error::Result<Vec<Message<'a>>> {
    let messages = session.fetch("1:*", "(UID ENVELOPE FLAGS)")?;

    let mut all_messages = vec![];
    for msg in messages.iter() {
        let uid = if let Some(uid) = msg.uid {
            uid
        } else {
            eprintln!("message does not have an uid");
            continue;
        };

        let envelope = msg.envelope();
        if let None = envelope {
            eprintln!("message {} does not have an envelope", uid);
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
            eprintln!("message id for message {} is not valid utf-8", uid);
        }
    }

    Ok(all_messages)
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
