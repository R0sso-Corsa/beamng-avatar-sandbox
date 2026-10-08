use avatar_bridge::{Config, Relay, MAX_BYTES};
use serde_json::json;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn http(mut stream: TcpStream, relay: &Mutex<Relay>) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_millis(500);
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        if let Some(i) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            if i + 4 > 8192 {
                return Err(io::Error::other("headers too large"));
            }
            break i + 4;
        }
        if bytes.len() >= 8192 {
            return Err(io::Error::other("headers too large"));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::other("request deadline"));
        }
        stream.set_read_timeout(Some(remaining))?;
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(io::Error::other("incomplete request"));
        }
        bytes.extend_from_slice(&chunk[..n]);
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(io::Error::other)?;
    let mut lines = headers.split("\r\n");
    let first = lines.next().unwrap_or("");
    let mut allowed = first == "POST /exchange HTTP/1.1" || first == "POST /exchange HTTP/1.0";
    let mut length = None;
    for line in lines.filter(|line| !line.is_empty()) {
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("content-length") {
                if length.is_some() {
                    allowed = false;
                }
                length = value.trim().parse::<usize>().ok();
            }
            if key.eq_ignore_ascii_case("transfer-encoding") || key.eq_ignore_ascii_case("origin") {
                allowed = false;
            }
        } else {
            allowed = false;
        }
    }
    let result = if length.is_some_and(|n| n <= MAX_BYTES) {
        let total = header_end + length.unwrap();
        while bytes.len() < total {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::Error::other("request deadline"));
            }
            stream.set_read_timeout(Some(remaining))?;
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                return Err(io::Error::other("incomplete body"));
            }
            bytes.extend_from_slice(&chunk[..n]);
        }
        if !allowed {
            Err("unsupported HTTP request")
        } else if bytes.len() != total {
            Err("extra request bytes")
        } else {
            relay
                .lock()
                .map_err(|_| io::Error::other("relay unavailable"))?
                .exchange(&bytes[header_end..], "guest", Instant::now())
        }
    } else {
        Err("unsupported HTTP request")
    };
    let status = if result.is_ok() {
        "200 OK"
    } else {
        "400 Bad Request"
    };
    let body = serde_json::to_vec(&result.unwrap_or_else(|e| json!({"ok":false,"error":e})))?;
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len())?;
    stream.write_all(&body)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--config") {
        return Err("usage: avatar-bridge --config <private session.json>".into());
    }
    let path = args.next().ok_or("missing config path")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let config: Config = serde_json::from_slice(&std::fs::read(path)?)?;
    config.validate()?;
    let http_listener = TcpListener::bind(("127.0.0.1", config.http_port))?;
    let udp = UdpSocket::bind(("127.0.0.1", config.udp_port))?;
    let relay = Arc::new(Mutex::new(Relay::new(config.clone())?));
    let http_relay = relay.clone();
    std::thread::spawn(move || {
        // ponytail: one bounded HTTP worker; one local guest is supported.
        for stream in http_listener.incoming().flatten() {
            let _ = http(stream, &http_relay);
        }
    });
    println!(
        "Avatar bridge ready: HTTP 127.0.0.1:{}, UDP 127.0.0.1:{}; local snapshots only",
        config.http_port, config.udp_port
    );
    let mut buffer = vec![0; MAX_BYTES + 1];
    loop {
        let (n, peer) = udp.recv_from(&mut buffer)?;
        if !peer.ip().is_loopback() {
            continue;
        }
        let result = relay.lock().map_err(|_| "relay unavailable")?.exchange(
            &buffer[..n],
            "host",
            Instant::now(),
        );
        let response = result.unwrap_or_else(|e| json!({"ok":false,"error":e}));
        let body = serde_json::to_vec(&response)?;
        udp.send_to(&body, peer)?;
    }
}
