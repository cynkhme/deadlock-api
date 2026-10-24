//! A local HTTP/1.1 server that answers scripted responses and records what
//! it was asked, so transport tests need no network.
#![allow(dead_code)] // shared by several test binaries; not every binary uses every helper

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
            let (status, extra, body) = responses[i.min(responses.len() - 1)];
            i += 1;
            let mut head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                body.len()
            );
            for (k, v) in extra {
                head.push_str(&format!("{k}: {v}\r\n"));
            }
            let _ = write!(stream, "{head}\r\n{body}");
            let _ = stream.flush();
        }
    });
    (url, seen)
}

/// A fresh temporary directory path for one test.
pub fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("deadlock-api-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}
