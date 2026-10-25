//! A local HTTP/1.1 server that answers scripted responses and records what
//! it was asked, so transport tests need no network.
#![allow(
    dead_code,
    reason = "shared by several test binaries; not every binary uses every helper"
)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// One recorded request: the request line and the headers, lower-cased names.
#[derive(Debug, Clone, PartialEq)]
pub struct Recorded {
    pub line: String,
    pub headers: Vec<(String, String)>,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// A scripted response: status line, extra header lines, body.
pub type Scripted = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
);

/// Starts a server that answers `responses` in order (the last one repeats)
/// and returns its root URL plus the requests it received.
pub fn server(responses: Vec<Scripted>) -> (String, Arc<Mutex<Vec<Recorded>>>) {
    server_bytes(
        responses
            .into_iter()
            .map(|(status, extra, body)| (status, extra, body.as_bytes().to_vec()))
            .collect(),
    )
}

/// A scripted response with a byte body, for responses that are not text.
pub type ScriptedBytes = (
    &'static str,
    &'static [(&'static str, &'static str)],
    Vec<u8>,
);

/// [`server`] with byte bodies.
pub fn server_bytes(responses: Vec<ScriptedBytes>) -> (String, Arc<Mutex<Vec<Recorded>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let recorded = seen.clone();
    std::thread::spawn(move || {
        let mut i = 0;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                continue;
            }
            let mut headers = Vec::new();
            loop {
                let mut h = String::new();
                reader.read_line(&mut h).unwrap();
                let h = h.trim_end();
                if h.is_empty() {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    headers.push((k.trim().to_ascii_lowercase(), v.trim().to_owned()));
                }
            }
            recorded.lock().unwrap().push(Recorded {
                line: line.trim_end().to_owned(),
                headers,
            });
            let (status, extra, body) = &responses[i.min(responses.len() - 1)];
            i += 1;
            let mut head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                body.len()
            );
            for (k, v) in extra.iter() {
                head.push_str(&format!("{k}: {v}\r\n"));
            }
            let _ = write!(stream, "{head}\r\n");
            let _ = stream.write_all(body);
            let _ = stream.flush();
        }
    });
    (url, seen)
}

/// `{"services":{"clickhouse":true,"postgres":true,"redis":true}}`, gzip-compressed.
pub const HEALTH_GZIP: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 171, 86, 42, 78, 45, 42, 203, 76, 78, 45, 86, 178, 170, 86,
    74, 206, 201, 76, 206, 206, 200, 47, 45, 78, 85, 178, 42, 41, 42, 77, 213, 81, 42, 200, 47, 46,
    73, 47, 2, 201, 66, 248, 69, 169, 41, 153, 80, 78, 109, 45, 0, 1, 224, 187, 104, 61, 0, 0, 0,
];

/// A fresh temporary directory path for one test.
pub fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("deadlock-api-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}
