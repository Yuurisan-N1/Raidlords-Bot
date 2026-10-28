use crate::support::error::Error;

const G: &str = "\x1b[1;32m";
const Y: &str = "\x1b[1;33m";
const R: &str = "\x1b[1;31m";
const C: &str = "\x1b[1;36m";
const Z: &str = "\x1b[0m";

pub const LINE_MAX: usize = 96;

static LINES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[allow(dead_code)]
pub fn lines_written() -> usize {
    LINES.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn install() {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "chcp 65001 >nul"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

fn line(color: &str, msg: &str) {
    LINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("{}{}{}", color, clamp(msg), Z);
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn lg(msg: &str) {
    line(G, msg);
}

pub fn ly(msg: &str) {
    line(Y, msg);
}

pub fn lr(msg: &str) {
    line(R, msg);
}

#[allow(dead_code)]
pub fn lc(msg: &str) {
    line(C, msg);
}

static LIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static ASKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn blank() {
    let live = LIVE.swap(false, std::sync::atomic::Ordering::Relaxed);
    let asking = ASKING.load(std::sync::atomic::Ordering::Relaxed);
    if live {
        print!("\r\x1b[2K\r");
    } else if asking {
        println!();
    }
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn ask(msg: &str) -> String {
    blank();
    print!("{}{}{} ", Y, squeeze(msg), Z);
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let mut answer = String::new();
    ASKING.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = std::io::stdin().read_line(&mut answer);
    ASKING.store(false, std::sync::atomic::Ordering::Relaxed);
    {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() {
            println!();
        }
    }
    answer.trim().to_string()
}

#[allow(dead_code)]
pub fn ask_secret(msg: &str) -> String {
    blank();
    let label = squeeze(msg);
    let echo = EchoGuard::off();
    print!("{}{}{} ", Y, label, Z);
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let mut answer = String::new();
    ASKING.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = std::io::stdin().read_line(&mut answer);
    ASKING.store(false, std::sync::atomic::Ordering::Relaxed);
    drop(echo);

    let trimmed = answer.trim().to_string();
    println!("{}{}{}", G, "*".repeat(10), Z);
    let _ = std::io::Write::flush(&mut std::io::stdout());
    trimmed
}

struct EchoGuard {
    suppressed: bool,
}

impl EchoGuard {
    fn off() -> EchoGuard {
        EchoGuard {
            suppressed: set_echo(false),
        }
    }
}

impl Drop for EchoGuard {
    fn drop(&mut self) {
        if self.suppressed {
            set_echo(true);
        }
    }
}

fn set_echo(on: bool) -> bool {
    let flags: &[&str] = if on {
        &["echo", "echonl"]
    } else {
        &["-echo", "-echonl"]
    };

    let direct = std::process::Command::new("stty")
        .args(flags)
        .stdin(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::null())
        .status();
    if let Ok(status) = direct {
        if status.success() {
            return true;
        }
    }

    let script = format!("stty {} < /dev/tty", flags.join(" "));
    std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn restore_echo() {
    set_echo(true);
}

pub fn newline() {
    println!();
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

#[allow(dead_code)]
pub fn gap() {
    LINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!();
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn squeeze(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut sp = false;
    for c in input.chars() {
        if c.is_control() || c == '\u{feff}' {
            if c == '\n' || c == '\r' || c == '\t' {
                sp = true;
            }
            continue;
        }
        if c.is_whitespace() {
            sp = true;
            continue;
        }
        if sp && !out.is_empty() {
            out.push(' ');
        }
        sp = false;
        out.push(c);
    }
    out
}

pub fn sanitize(input: &str) -> String {
    let mut out = squeeze(input);
    for key in [
        "accessToken",
        "access_token",
        "refreshToken",
        "initData",
        "init_data",
        "Authorization",
        "authorization",
        "idToken",
        "id_token",
        "password",
        "PRIVATEKEY",
        "privatekey",
        "privateKey",
        "signature",
        "token",
    ] {
        out = redact_after(&out, key);
    }
    out
}

pub fn reason(input: &str) -> &'static str {
    let text = sanitize(input).to_ascii_lowercase();
    if text.contains("timed out") || text.contains("timeout") {
        "the request timed out"
    } else if text.contains("connect") {
        "the connection could not be established"
    } else if text.contains("dns") || text.contains("resolve") || text.contains("name or service") {
        "the address could not be resolved"
    } else if text.contains("body") || text.contains("decode") || text.contains("json") {
        "the reply could not be read"
    } else if text.contains("config") || text.contains(".env") || text.contains("proxy.txt") {
        "the local files are not ready"
    } else {
        "the request failed"
    }
}

pub fn explain(e: &Error) -> String {
    match e {
        Error::Cfg(_) => "the local files are not ready".to_string(),
        Error::Auth(msg) | Error::State(msg) | Error::Sim(msg) | Error::Wallet(msg) => {
            sanitize(msg)
        }
        Error::Api { status, message } => {
            let text = sanitize(message);
            if text.is_empty() {
                format!("the server answered {}", status)
            } else {
                text
            }
        }
        Error::Http(msg) => sanitize(msg),
        other => reason(&other.to_string()).to_string(),
    }
}

fn redact_after(input: &str, key: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find(key) {
        out.push_str(&rest[..pos]);
        out.push_str(key);
        let tail = &rest[pos + key.len()..];
        let mut idx = 0;
        let bytes = tail.as_bytes();
        while idx < bytes.len()
            && (bytes[idx] == b'"'
                || bytes[idx] == b':'
                || bytes[idx] == b'='
                || bytes[idx] == b' ')
        {
            out.push(bytes[idx] as char);
            idx += 1;
        }
        while idx < bytes.len() {
            let c = bytes[idx] as char;
            if c == '"' || c == '&' || c == ' ' || c == ',' || c == '}' || c == '\n' {
                break;
            }
            idx += 1;
        }
        out.push_str("***");
        rest = &tail[idx..];
    }
    out.push_str(rest);
    out
}

pub fn money(value: &str) -> String {
    format!("{}{}{}", G, value, Z)
}

pub fn mask_host(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() == 4 {
        return format!("{}*****{}", parts[0], parts[3]);
    }
    let chars: Vec<char> = host.chars().collect();
    if chars.len() > 4 {
        let head: String = chars.iter().take(2).collect();
        let tail: String = chars
            .iter()
            .rev()
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{}*****{}", head, tail)
    } else {
        "***".to_string()
    }
}

pub fn mask_proxy(line: &str) -> String {
    let mut rest = line.trim();
    if let Some(pos) = rest.find("://") {
        rest = &rest[pos + 3..];
    }
    if let Some(at) = rest.rfind('@') {
        rest = &rest[at + 1..];
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) && !port.is_empty() => {
            (host, port)
        }
        _ => (rest, ""),
    };
    let shown = mask_host(host);
    if port.is_empty() {
        format!("http://user:pass@{}", shown)
    } else {
        format!("http://user:pass@{}:{}", shown, port)
    }
}

pub fn mask_wallet(address: &str) -> String {
    let body = address.trim_start_matches("0x");
    if body.len() < 8 {
        return "0x*****".to_string();
    }
    let head: String = body.chars().take(4).collect();
    let tail: String = body
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("0x{}*****{}", head, tail)
}

fn short(input: &str, max: usize) -> String {
    let s = squeeze(input);
    if s.chars().count() <= max {
        return s;
    }
    let cut: String = s.chars().take(max + 1).collect();
    match cut.rfind(' ') {
        Some(i) if i > 0 => cut[..i].trim_end().to_string(),
        _ => s.chars().take(max).collect(),
    }
}

pub fn clamp(input: &str) -> String {
    short(input, LINE_MAX)
}

pub fn unit_word<'a>(value: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if value == 1 {
        singular
    } else {
        plural
    }
}

fn left_text(left: u64) -> String {
    let days = left / 86400;
    let hours = (left % 86400) / 3600;
    let minutes = (left % 3600) / 60;
    let seconds = left % 60;
    if days > 0 {
        return format!("{:02}:{:02}:{:02}:{:02}", days, hours, minutes, seconds);
    }
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn stamp(left: u64) -> String {
    left_text(left)
}

pub fn countdown(seconds: u64, label: &str) {
    let text = squeeze(label);
    let mut left = seconds;
    LIVE.store(true, std::sync::atomic::Ordering::Relaxed);
    while left > 0 {
        print!("\r{}{} {}{}", Y, text, left_text(left), Z);
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::thread::sleep(std::time::Duration::from_secs(1));
        left -= 1;
    }
    blank();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_text_carries_days() {
        assert_eq!(left_text(45), "00:00:45");
        assert_eq!(left_text(3661), "01:01:01");
        assert_eq!(
            left_text(2 * 86400 + 3 * 3600 + 47 * 60 + 33),
            "02:03:47:33"
        );
    }

    #[test]
    fn masked_proxy_hides_the_middle_and_the_password() {
        let shown = mask_proxy("socks5://user:secret@192.0.2.10:10043");
        assert_eq!(shown, "http://user:pass@192*****10:10043");
        assert!(!shown.contains("secret"), "password survived: {}", shown);
        assert_eq!(
            mask_proxy("http://user:secret@198.51.100.81:8000"),
            "http://user:pass@198*****81:8000"
        );
    }

    #[test]
    fn a_wallet_keeps_only_the_outer_groups() {
        assert_eq!(
            mask_wallet("0x1111111111111111111111111111111111111111"),
            "0x1111*****1111"
        );
    }

    #[test]
    fn clamp_keeps_a_long_line_at_the_limit() {
        let long = "word ".repeat(80);
        assert!(clamp(&long).chars().count() <= LINE_MAX);
    }
}
