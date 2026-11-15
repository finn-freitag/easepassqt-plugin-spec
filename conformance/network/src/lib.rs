//! Conformance plugin for the `network` permission. It is an importer whose source id is
//! a command, so a host test can drive any request. Behaviour: see ../README.md.

wit_bindgen::generate!({
    path: "../../wit",
    world: "easepass:plugin/importer-plugin",
    generate_all,
});

use easepass::plugin::network::{
    http_request, tcp_close, tcp_connect, tcp_read, tcp_start_tls, tcp_write, HttpHeader, Request, TcpOptions,
};
use easepass::plugin::types::{Error, ErrorCode, PasswordItem};
use exports::easepass::plugin::importer::{Guest, ImportSource};

struct Plugin;

fn error(code: ErrorCode, message: &str) -> Error {
    Error { code, message: message.into() }
}

fn item(display_name: String, user_name: String, notes: String) -> PasswordItem {
    PasswordItem {
        display_name,
        user_name,
        email: String::new(),
        website: String::new(),
        password: String::new(),
        notes,
        tags: Vec::new(),
        totp: None,
    }
}

/// `http <METHOD> <url> [body]`
fn http(arguments: &str) -> Result<Vec<PasswordItem>, Error> {
    let mut parts = arguments.splitn(3, ' ');
    let method = parts.next().filter(|m| !m.is_empty());
    let url = parts.next();
    let (Some(method), Some(url)) = (method, url) else {
        return Err(error(ErrorCode::InvalidArgument, "usage: http <METHOD> <url> [body]"));
    };
    let response = http_request(&Request {
        method: method.into(),
        url: url.into(),
        headers: vec![HttpHeader { name: "x-conformance".into(), value: "1".into() }],
        body: parts.next().map(|body| body.as_bytes().to_vec()),
        timeout_ms: Some(5000),
    })?;
    let reply = response
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("x-conformance-reply"))
        .map(|header| header.value.clone())
        .unwrap_or_default();
    Ok(vec![item(
        response.status.to_string(),
        reply,
        String::from_utf8_lossy(&response.body).into_owned(),
    )])
}

/// `tcp <host> <port> <plain|tls|starttls> <text>`: sends `text` and a line feed, returns
/// the first line of the answer.
fn tcp(arguments: &str) -> Result<Vec<PasswordItem>, Error> {
    let usage = || error(ErrorCode::InvalidArgument, "usage: tcp <host> <port> <plain|tls|starttls> <text>");
    let mut parts = arguments.splitn(4, ' ');
    let host = parts.next().ok_or_else(usage)?;
    let port: u16 = parts.next().and_then(|port| port.parse().ok()).ok_or_else(usage)?;
    let mode = parts.next().ok_or_else(usage)?;
    let text = parts.next().unwrap_or("");
    if !matches!(mode, "plain" | "tls" | "starttls") {
        return Err(usage());
    }

    let connection = tcp_connect(host, port, TcpOptions { tls: mode == "tls", timeout_ms: Some(5000) })?;
    let exchange = || -> Result<String, Error> {
        if mode == "starttls" {
            tcp_start_tls(connection)?;
        }
        let mut pending = format!("{text}\n").into_bytes();
        while !pending.is_empty() {
            let written = tcp_write(connection, &pending)? as usize;
            pending.drain(..written.min(pending.len()));
        }
        let mut line = Vec::new();
        loop {
            let chunk = tcp_read(connection, 4096)?;
            if chunk.is_empty() {
                break;
            }
            if let Some(end) = chunk.iter().position(|byte| *byte == b'\n') {
                line.extend_from_slice(&chunk[..end]);
                break;
            }
            line.extend_from_slice(&chunk);
        }
        Ok(String::from_utf8_lossy(&line).into_owned())
    };
    let result = exchange();
    tcp_close(connection);
    Ok(vec![item("tcp".into(), String::new(), result?)])
}

impl Guest for Plugin {
    fn sources() -> Vec<ImportSource> {
        vec![
            ImportSource {
                id: "http".into(),
                name: "HTTP request".into(),
                description: "Source id: http <METHOD> <url> [body]".into(),
            },
            ImportSource {
                id: "tcp".into(),
                name: "TCP exchange".into(),
                description: "Source id: tcp <host> <port> <plain|tls|starttls> <text>".into(),
            },
        ]
    }

    fn import_passwords(source_id: String) -> Result<Vec<PasswordItem>, Error> {
        match source_id.split_once(' ') {
            Some(("http", arguments)) => http(arguments),
            Some(("tcp", arguments)) => tcp(arguments),
            _ => Err(error(ErrorCode::NotFound, "unknown source")),
        }
    }
}

export!(Plugin);
