use crate::support::log;

#[derive(Debug, Clone)]
pub struct ProxySpec {
    raw: String,
    url: String,
    user: Option<String>,
    pass: Option<String>,
}

impl ProxySpec {
    pub fn parse(line: &str) -> Option<ProxySpec> {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            return None;
        }

        let (scheme, rest) = match text.split_once("://") {
            Some((scheme, rest)) => (scheme.to_ascii_lowercase(), rest),
            None => ("http".to_string(), text),
        };

        let (auth, hostport) = match rest.rsplit_once('@') {
            Some((auth, hostport)) => (Some(auth), hostport),
            None => (None, rest),
        };

        let (user, pass) = match auth {
            Some(pair) => match pair.split_once(':') {
                Some((user, pass)) => (Some(percent_decode(user)), Some(percent_decode(pass))),
                None => (Some(percent_decode(pair)), None),
            },
            None => (None, None),
        };

        let hostport = hostport.trim_end_matches('/');
        if hostport.is_empty() {
            return None;
        }

        let url = match scheme.as_str() {
            "socks5" | "socks5h" => format!("socks5h://{}", hostport),
            _ => format!("http://{}", hostport),
        };

        Some(ProxySpec {
            raw: text.to_string(),
            url,
            user,
            pass,
        })
    }

    pub fn masked(&self) -> String {
        log::mask_proxy(&self.raw)
    }

    pub fn apply(
        &self,
        builder: reqwest::blocking::ClientBuilder,
    ) -> reqwest::blocking::ClientBuilder {
        let mut proxy = reqwest::Proxy::all(&self.url)
            .unwrap_or_else(|_| reqwest::Proxy::all("http://127.0.0.1:1").unwrap());
        if let Some(user) = &self.user {
            proxy = proxy.basic_auth(user, self.pass.as_deref().unwrap_or(""));
        }
        builder.proxy(proxy)
    }
}

pub fn load() -> Vec<ProxySpec> {
    let text = std::fs::read_to_string(crate::support::consts::app::PROXY_FILE).unwrap_or_default();
    let mut out: Vec<ProxySpec> = Vec::new();
    for line in text.lines() {
        if let Some(spec) = ProxySpec::parse(line) {
            if out.iter().any(|seen| seen.url == spec.url) {
                log::lr("proxy.txt repeats an entry so two lanes share one ip");
                continue;
            }
            out.push(spec);
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            let hi = (bytes[idx + 1] as char).to_digit(16);
            let lo = (bytes[idx + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                idx += 3;
                continue;
            }
        }
        out.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_with_credentials_keeps_them_out_of_the_log() {
        let spec = ProxySpec::parse("http://user:p%40ss@192.0.2.10:10043").unwrap();
        assert_eq!(spec.masked(), "http://user:pass@192*****10:10043");
        assert_eq!(spec.pass.as_deref(), Some("p@ss"));
        assert!(!spec.masked().contains("p@ss"));
    }

    #[test]
    fn a_blank_line_is_not_a_proxy() {
        assert!(ProxySpec::parse("").is_none());
        assert!(ProxySpec::parse("   ").is_none());
        assert!(ProxySpec::parse("# note").is_none());
    }

    #[test]
    fn a_host_without_a_scheme_still_works() {
        let spec = ProxySpec::parse("198.51.100.81:8000").unwrap();
        assert_eq!(spec.url, "http://198.51.100.81:8000");
        assert_eq!(spec.masked(), "http://user:pass@198*****81:8000");
    }
}
